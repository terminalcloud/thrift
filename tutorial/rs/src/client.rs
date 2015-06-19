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

use std::net::TcpStream;
use bufstream::BufStream;
use thrift::protocol::binary_protocol::BinaryProtocol;

mod tutorial;
mod shared;

pub fn main() {
    let stream = BufStream::new(TcpStream::connect("127.0.0.1:9090").unwrap());
    let mut client = tutorial::CalculatorClient::new(BinaryProtocol, stream);

    // Ping
    client.ping().unwrap();
    println!("ping()");

    // Add
    println!("1 + 1 = {}", client.add(1, 1).unwrap());

    // TODO: Add this back in when exceptions are implemented
    // Work: divide
    // let work = tutorial::Work {
    //   op: tutorial::Operation::DIVIDE,
    //   num1: 1,
    //   num2: 0,
    //   comment: None };
    //
    // match client.calculate(1, work) {
    //   Ok(_) => {
    //     println!("Whoa? We can divide by zero!");
    //   }
    //   Err(_) => {
    //     // FIXME: use thrift exceptions
    //     println!("Invalid operation")
    //   }
    // }

    // Work: subtract
    let work = tutorial::Work {
        op: tutorial::Operation::SUBTRACT,
        num1: 15,
        num2: 10,
        comment: None };
    println!("15 - 10 = {}", client.calculate(1, work).ok().unwrap());

    let ss = client.getStruct(1).ok().unwrap();
    println!("Received log: {}", ss.value);

    println!("PASS");
}
