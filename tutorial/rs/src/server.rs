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

#[macro_use]
extern crate thrift;
extern crate bufstream;

mod tutorial;
mod shared;

use std::io;
use std::net::{TcpListener, TcpStream};
use std::cell::RefCell;
use std::collections::HashMap;

use thrift::protocol::binary_protocol::BinaryProtocol;
use thrift::server::SimpleServer;
use thrift::transport::server::TransportServer;

use tutorial::*;
use shared::*;

use bufstream::BufStream;

struct CalculatorHandler {
    log: RefCell<HashMap<i32, SharedStruct>>
}

impl<'a> Calculator for &'a CalculatorHandler {
    fn ping(&self) {
        println!("ping()");
    }

    fn add(&self, n1: i32, n2: i32) -> i32 {
        println!("add({}, {})", n1, n2);
        n1 + n2
    }

    fn calculate(&self, log_id: i32, work: Work) -> Result<i32, CalculatorCalculateError> {
        println!("calculate({}, {:?})", log_id, work);

        let num1 = work.num1;
        let num2 = work.num2;

        let val = match work.op {
            Operation::ADD => num1 + num2,
            Operation::SUBTRACT => num1 - num2,
            Operation::MULTIPLY => num1 * num2,
            Operation::DIVIDE => {
                if num2 == 0 {
                    return Err(CalculatorCalculateError::Ouch(InvalidOperation {
                        what_op: work.op as i32,
                        why: "Cannot divide by 0".into()
                    }))
                }

                num1 / num2
            }
        };

        let ss = SharedStruct { key: log_id, value: val.to_string() };
        self.log.borrow_mut().insert(log_id, ss);

        Ok(val)
    }

    fn zip(&self) {
        println!("zip");
    }
}

impl<'a> SharedService for &'a CalculatorHandler {
    fn getStruct(&self, log_id: i32) -> SharedStruct {
        println!("getStruct({})", log_id);
        self.log.borrow()[&log_id].clone()
    }
}

struct BufferServer(TcpListener);

impl TransportServer for BufferServer {
     type Transport = BufStream<TcpStream>;

     fn accept(&self) -> io::Result<BufStream<TcpStream>> {
        self.0.accept().map(|res| BufStream::new(res.0))
     }
}

pub fn main() {
    let handler = CalculatorHandler { log: RefCell::new(HashMap::new()) };
    let processor = CalculatorProcessor::new(&handler, &handler);

    let server_transport = BufferServer(TcpListener::bind("127.0.0.1:9090").unwrap());
    let mut server = SimpleServer::new(processor, server_transport, || BinaryProtocol);

    println!("Starting the server...");
    server.serve();
    println!("Done.");
}
