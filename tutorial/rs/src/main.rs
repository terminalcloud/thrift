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

#![crate_name="calculator"]
#![crate_type="bin"]

extern crate thrift;

use std::str::FromStr;
use std::io::net::ip;
use thrift::TResult;
use thrift::ThriftErr;
use thrift::ThriftErr::*;
use thrift::protocol::{MessageType, Type};
use thrift::transport::Transport;
use thrift::protocol::Protocol;
use thrift::protocol::ProtocolHelpers;
use thrift::protocol::{Readable, Writeable};
use thrift::protocol::binary_protocol::BinaryProtocol;

mod tutorial;
mod shared;

fn runClient(client: &mut tutorial::CalculatorClient) {
    // Ping
    client.ping().unwrap();
    println!("ping()");

    // Add
    println!("1 + 1 = {}", client.add(1, 1).unwrap());

    // Work: divide
    let work = tutorial::Work { 
      op: tutorial::Operation::DIVIDE, 
      num1: 1, 
      num2: 0, 
      comment: None };

    match client.calculate(1, work) {
      Ok(_) => {
        println!("Whoa? We can divide by zero!");
      }
      Err(_) => {
        // FIXME: use thrift exceptions
        println!("Invalid operation")
      }
    }

    // Work: subtract
    let work = tutorial::Work { 
        op: tutorial::Operation::SUBTRACT, 
        num1: 15, 
        num2: 10, 
        comment: None };
    println!("15 - 10 = {}", client.calculate(2, work).unwrap());

    let ss = client.getStruct(1).unwrap();
    println!("Received log: {:?}", ss);

    println!("PASS");
}

pub fn main() {
    let addr: ip::SocketAddr = FromStr::from_str("127.0.0.1:9090")
        .expect("bad server address");
    let tcp = std::io::TcpStream::connect(addr).unwrap();
    // FIXME: do we want tutorial::build_calculator_client(BinaryProtocol, tcp) here?
    let mut client = tutorial::CalculatorClientImpl::new(BinaryProtocol, tcp);

    runClient(&mut client);
}