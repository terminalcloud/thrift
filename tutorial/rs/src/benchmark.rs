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
use tutorial::CalculatorClient;

mod tutorial;
mod shared;

// Minimalistic performance test
// Start C++ TutorialServer with output redirected to /dev/null

pub fn main() {
    let iterations = match std::env::args().nth(1) {
        Some(s) => {
            let t = s.parse().unwrap();
            println!("arg: {}", t);
            t
        }
        _ => 1000000
    };

    let mut client = tutorial::CalculatorClient::new(
        BinaryProtocol, BufStream::new(TcpStream::connect("127.0.0.1:9090").unwrap()));

    println!("Rust Thrift benchmark");
    println!("Running {} iterations", iterations);

    for i in 0..iterations {
        let _ = client.add(1, 1).ok().unwrap();
        if i % 100 == 0 {
            print!(".");
        }
    }
    println!("");
    println!("DONE");
}
