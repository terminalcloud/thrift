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

use super::BinaryProtocol;

use mock::MockTransport;
use protocol::{self, Protocol};
use Error;

#[test]
fn read_bool() {
    let transport = &mut MockTransport::new(vec!(0x00, 0x01, 0xff));
    let mut protocol = BinaryProtocol;
    assert_eq!(protocol.read_bool(transport).unwrap(), false);
    assert_eq!(protocol.read_bool(transport).unwrap(), true);
    assert_eq!(protocol.read_bool(transport).unwrap(), true);
}

#[test]
fn read_byte() {
    let transport = &mut MockTransport::new(vec!(0xa4, 0x27));
    let mut protocol = BinaryProtocol;
    assert_eq!(protocol.read_byte(transport).unwrap(), -0x5c);
    assert_eq!(protocol.read_byte(transport).unwrap(), 0x27);
}

#[test]
fn read_i16() {
    let transport = &mut MockTransport::new(vec!(0xf2, 0xf8, 0xa1, 0x40));
    let mut protocol = BinaryProtocol;
    assert_eq!(protocol.read_i16(transport).unwrap(), -0x0d08);
    assert_eq!(protocol.read_i16(transport).unwrap(), -0x5ec0);
}

#[test]
fn read_i32() {
    let transport = &mut MockTransport::new(vec!(0x27, 0xd0, 0x39, 0x49, 0xe5, 0xd8, 0xfe, 0x8b));
    let mut protocol = BinaryProtocol;
    assert_eq!(protocol.read_i32(transport).unwrap(), 0x27d03949);
    assert_eq!(protocol.read_i32(transport).unwrap(), -0x1a270175);
}

#[test]
fn read_i64() {
    let transport = &mut MockTransport::new(vec!(
        0x27, 0xd0, 0x39, 0x49, 0xe5, 0xd8, 0xfe, 0x8b,
        0xa7, 0x2e, 0x82, 0xea, 0xd1, 0x28, 0x0b, 0xe2,
    ));
    let mut protocol = BinaryProtocol;
    assert_eq!(protocol.read_i64(transport).unwrap(), 0x27d03949e5d8fe8b);
    assert_eq!(protocol.read_i64(transport).unwrap(), -0x58d17d152ed7f41e);
}

#[test]
fn read_double() {
    let transport = &mut MockTransport::new(vec!(
        0x40, 0xa9, 0x5e, 0xaf, 0x39, 0x4b, 0x7b, 0x29,
        0xbf, 0xe9, 0x3a, 0xe4, 0x21, 0xd3, 0x0e, 0x85,
    ));
    let mut protocol = BinaryProtocol;
    assert_eq!(protocol.read_double(transport).unwrap(), 3247.342234);
    assert_eq!(protocol.read_double(transport).unwrap(), -0.78843886);
}

#[test]
fn read_string() {
    let transport = &mut MockTransport::new(vec!(
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x04, 0x41, 0x73, 0x64, 0x66,
        0x00, 0x00, 0x00, 0x0d, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21,
    ));
    let mut protocol = BinaryProtocol;
    assert_eq!(&protocol.read_string(transport).unwrap(), "");
    assert_eq!(&protocol.read_string(transport).unwrap(), "Asdf");
    assert_eq!(&protocol.read_string(transport).unwrap(), "Hello, World!");
}

#[test]
fn read_binary() {
    let transport = &mut MockTransport::new(vec!(
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x04, 0x41, 0x73, 0x64, 0x66,
        0x00, 0x00, 0x00, 0x0d, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21,
    ));
    let mut protocol = BinaryProtocol;
    assert_eq!(&protocol.read_binary(transport).unwrap(), &[]);
    assert_eq!(&protocol.read_binary(transport).unwrap(), &[0x41, 0x73, 0x64, 0x66]);
    assert_eq!(&protocol.read_binary(transport).unwrap(), &[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x2c, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21]);
}

#[test]
fn read_set_begin() {
    let transport = &mut MockTransport::new(vec!(0x0b, 0x00, 0x00, 0x01, 0x0f));
    let mut protocol = BinaryProtocol;
    assert_eq!(
        protocol.read_set_begin(transport).unwrap(),
        (protocol::Type::String, 0x0000010f)
    );
}

#[test]
fn read_list_begin() {
    let transport = &mut MockTransport::new(vec!(0x0b, 0x00, 0x00, 0x01, 0x0f));
    let mut protocol = BinaryProtocol;
    assert_eq!(
        protocol.read_list_begin(transport).unwrap(),
        (protocol::Type::String, 0x0000010f)
    );
}

#[test]
fn read_map_begin() {
    let transport = &mut MockTransport::new(vec!(0x0b, 0x08, 0x00, 0x00, 0x01, 0x0f));
    let mut protocol = BinaryProtocol;
    assert_eq!(
        protocol.read_map_begin(transport).unwrap(),
        (protocol::Type::String, protocol::Type::I32, 0x0000010f)
    );
}

#[test]
fn read_field_begin() {
    let transport = &mut MockTransport::new(vec!(0x00, 0x0d, 0x14, 0x0e));
    let mut protocol = BinaryProtocol;
    assert_eq!(
        protocol.read_field_begin(transport).unwrap(),
        ("".to_string(), protocol::Type::Stop, 0)
    );
    assert_eq!(
        protocol.read_field_begin(transport).unwrap(),
        ("".to_string(), protocol::Type::Map, 0x140e)
  );
}

#[test]
fn read_message_begin() {
    let transport = &mut MockTransport::new(vec!(
        0x80, 0x01, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x03, 0x66, 0x6f, 0x6f,
        0x00, 0x02, 0x47, 0x1e
    ));
    let mut protocol = BinaryProtocol;
    assert_eq!(
        protocol.read_message_begin(transport).unwrap(),
        ("foo".to_string(), protocol::MessageType::Call, 0x0002471e)
    );
}

#[test]
fn read_message_begin_bad_version() {
    let transport = &mut MockTransport::new(vec!(
        0x80, 0x22, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x03, 0x66, 0x6f, 0x6f,
        0x00, 0x02, 0x47, 0x1e
    ));
    let mut protocol = BinaryProtocol;
    let err = protocol.read_message_begin(transport).unwrap_err();
    match err {
        Error::ProtocolError(e) => assert_eq!(e, protocol::Error::BadVersion),
        e => panic!("Expected a protocol error, got {:?}", e)
    }
}

#[test]
fn read_message_begin_invalid_message_type() {
    let transport = &mut MockTransport::new(vec!(
        0x80, 0x01, 0x00, 0x0f,
        0x00, 0x00, 0x00, 0x03, 0x66, 0x6f, 0x6f,
        0x00, 0x02, 0x47, 0x1e
    ));
    let mut protocol = BinaryProtocol;
    let err = protocol.read_message_begin(transport).unwrap_err();
    match err {
        Error::ProtocolError(e) => assert_eq!(e, protocol::Error::ProtocolViolation),
        e => panic!("Expected a protocol error, got {:?}", e)
    }
}

