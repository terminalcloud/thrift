/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements. See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership. The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License. You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::io::{self, Read, Write};

use byteorder::{ByteOrder, BigEndian};

pub mod server;

pub trait Transport: Write + Read { }

impl<'t, T> Transport for &'t mut T where T: Transport {}
impl<'t> Transport for &'t mut Transport {}

pub struct RwTransport<T>(pub T);

impl<T: Read> Read for RwTransport<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<T: Write> Write for RwTransport<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<T: Read + Write> Transport for RwTransport<T> { }

pub struct FramedTransport<T> {
    pub transport: T,
    tx_buffer: Vec<u8>,
    flushed: bool,
}

impl<T> FramedTransport<T> {
    pub fn new(transport: T) -> FramedTransport<T> {
        FramedTransport {
            transport: transport,
            tx_buffer: Vec::<u8>::new(),
            flushed: true,
        }
    }

    fn flush(&mut self) -> &Self {
        self.tx_buffer.clear();
        self.flushed = true;
        self
    }
}

impl<T: Read> Read for FramedTransport<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

        loop {
            if self.flushed {
                // read the size of the frame
                let mut b = [0; 4];
                if let Ok(_) = self.transport.read_exact(&mut b) {
                    self.flushed = false;
                }
            } else {
                break;
            }
        }

        self.transport.read(buf)
    }
}

impl<T: Write> Write for FramedTransport<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tx_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // write all buffered data to sink
        let mut size = [0; 4];
        BigEndian::write_i32(&mut size, self.tx_buffer.len() as i32);
        try!(self.transport.write(&size));
        try!(self.transport.write(&self.tx_buffer));

        // flush self
        self.flush();

        // flush the sink
        self.transport.flush()
    }
}

impl<T: Read + Write> Transport for FramedTransport<T> { }
