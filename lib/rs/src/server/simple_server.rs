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

use transport::server::TransportServer;
use transport::Transport;
use protocol::ProtocolFactory;
use processor::Processor;

pub struct SimpleServer<P, PF, TS> {
    processor: P,
    protocol_factory: PF,
    transport_server: TS,
}

impl<P, PF: ProtocolFactory, TS: TransportServer> SimpleServer<P, PF, TS>
where P: Processor<PF::Protocol, TS::Transport>,
      TS::Transport: Transport {

    pub fn new(processor: P, transport_server: TS, pf: PF) -> Self {
        SimpleServer {
            processor: processor,
            protocol_factory: pf,
            transport_server: transport_server
        }
    }

    pub fn serve(&mut self) {
        loop {
            let mut transport = self.transport_server.accept().unwrap();
            let mut protocol = self.protocol_factory.new_protocol();
            while let Ok(_) = self.processor.process(&mut protocol, &mut transport) { }
        }
    }
}
