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

use protocol::{self, MessageType, Protocol, Type};
use transport::Transport;
use {Result, Error};

use podio::{ReadPodExt, WritePodExt, BigEndian};

static BINARY_PROTOCOL_VERSION_1: u16 = 0x8001;

#[derive(Copy, Clone)]
pub struct BinaryProtocol;

impl BinaryProtocol {
    fn write_type<T: Transport>(&self, transport: &mut T, type_: Type) -> Result<()> {
        self.write_byte(transport, type_ as i8)
    }

    fn read_type<T: Transport>(&self, transport: &mut T) -> Result<Type> {
        let raw = try!(self.read_byte(transport));
        match Type::from_num(raw as u64) {
            Some(type_) => Ok(type_),
            None => Err(Error::from(protocol::Error::ProtocolViolation)),
        }
    }
}

impl Protocol for BinaryProtocol {
    fn write_message_begin<T: Transport>(
        &self,
        transport: &mut T,
        name: &str,
        message_type: MessageType,
        sequence_id: i32
    ) -> Result<()> {
        let version = ((BINARY_PROTOCOL_VERSION_1 as i32) << 16) | message_type as i32;
        try!(self.write_i32(transport, version));
        try!(self.write_str(transport, name));
        self.write_i32(transport, sequence_id)
    }

    fn write_message_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_struct_begin<T: Transport>(&self, _transport: &mut T, _name: &str) -> Result<()> {
        Ok(())
    }

    fn write_struct_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_field_begin<T: Transport>(
        &self,
        transport: &mut T,
        _name: &str,
        field_type: Type,
        field_id: i16
    ) -> Result<()> {
        try!(self.write_type(transport, field_type));
        self.write_i16(transport, field_id)
    }

    fn write_field_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_field_stop<T: Transport>(&self, transport: &mut T) -> Result<()> {
        self.write_byte(transport, protocol::Type::Stop as i8)
    }

    fn write_map_begin<T: Transport>(
        &self,
        transport: &mut T,
        key_type: Type,
        value_type: Type,
        size: usize
    ) -> Result<()> {
        try!(self.write_type(transport, key_type));
        try!(self.write_type(transport, value_type));
        self.write_i32(transport, size as i32)
    }

    fn write_map_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_list_begin<T: Transport>(&self, transport: &mut T, elem_type: Type, size: usize) -> Result<()> {
        try!(self.write_type(transport, elem_type));
        self.write_i32(transport, size as i32)
    }

    fn write_list_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_set_begin<T: Transport>(&self, transport: &mut T, elem_type: Type, size: usize) -> Result<()> {
        try!(self.write_type(transport, elem_type));
        self.write_i32(transport, size as i32)
    }

    fn write_set_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_bool<T: Transport>(&self, transport: &mut T, value: bool) -> Result<()> {
        self.write_byte(transport, value as i8)
    }

    fn write_byte<T: Transport>(&self, mut transport: &mut T, value: i8) -> Result<()> {
        Ok(try!(transport.write_i8(value)))
    }

    fn write_i16<T: Transport>(&self, mut transport: &mut T, value: i16) -> Result<()> {
        Ok(try!(transport.write_i16::<BigEndian>(value)))
    }

    fn write_i32<T: Transport>(&self, mut transport: &mut T, value: i32) -> Result<()> {
        Ok(try!(transport.write_i32::<BigEndian>(value)))
    }

    fn write_i64<T: Transport>(&self, mut transport: &mut T, value: i64) -> Result<()> {
        Ok(try!(transport.write_i64::<BigEndian>(value)))
    }

    fn write_double<T: Transport>(&self, mut transport: &mut T, value: f64) -> Result<()> {
        Ok(try!(transport.write_f64::<BigEndian>(value)))
    }

    fn write_str<T: Transport>(&self, transport: &mut T, value: &str) -> Result<()> {
        self.write_binary(transport, value.as_bytes())
    }

    fn write_string<T: Transport>(&self, transport: &mut T, value: &String) -> Result<()> {
        self.write_binary(transport, (&value[..]).as_bytes())
    }

    fn write_binary<T: Transport>(&self, transport: &mut T, value: &[u8]) -> Result<()> {
        try!(self.write_i32(transport, value.len() as i32));
        Ok(try!(transport.write_all(value)))
    }

    fn read_message_begin<T: Transport>(&self, transport: &mut T) -> Result<(String, MessageType, i32)> {
        let header = try!(self.read_i32(transport));
        let version = (header >> 16) as u16;
        if version != BINARY_PROTOCOL_VERSION_1 {
            return Err(Error::from(protocol::Error::BadVersion));
        };
        let name = try!(self.read_string(transport));
        let raw_type = header & 0xff;
        let message_type = match MessageType::from_num(raw_type as u64) {
            Some(t) => t,
            None => return Err(Error::from(protocol::Error::ProtocolViolation)),
        };
        let sequence_id = try!(self.read_i32(transport));
        Ok((name, message_type, sequence_id))
    }

    fn read_message_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_struct_begin<T: Transport>(&self, _transport: &mut T) -> Result<String> {
        Ok(String::new())
    }

    fn read_struct_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_field_begin<T: Transport>(&self, transport: &mut T) -> Result<(String, Type, i16)> {
        let field_type = try!(self.read_type(transport));
        let field_id = match field_type {
            protocol::Type::Stop => 0,
            _ => try!(self.read_i16(transport)),
        };
        Ok((String::new(), field_type, field_id))
    }

    fn read_field_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_map_begin<T: Transport>(&self, transport: &mut T) -> Result<(Type, Type, i32)> {
        let key_type = try!(self.read_type(transport));
        let value_type = try!(self.read_type(transport));
        let size = try!(self.read_i32(transport));
        Ok((key_type, value_type, size))
    }

    fn read_map_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_list_begin<T: Transport>(&self, transport: &mut T) -> Result<(Type, i32)> {
        let elem_type = try!(self.read_type(transport));
        let size = try!(self.read_i32(transport));
        Ok((elem_type, size))
    }

    fn read_list_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_set_begin<T: Transport>(&self, transport: &mut T) -> Result<(Type, i32)> {
        let elem_type = try!(self.read_type(transport));
        let size = try!(self.read_i32(transport));
        Ok((elem_type, size))
    }

    fn read_set_end<T: Transport>(&self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_bool<T: Transport>(&self, transport: &mut T) -> Result<bool> {
        match try!(self.read_byte(transport)) {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    fn read_byte<T: Transport>(&self, transport: &mut T) -> Result<i8> {
        Ok(try!(transport.read_i8()))
    }

    fn read_i16<T: Transport>(&self, transport: &mut T) -> Result<i16> {
        Ok(try!(transport.read_i16::<BigEndian>()))
    }

    fn read_i32<T: Transport>(&self, transport: &mut T) -> Result<i32> {
        Ok(try!(transport.read_i32::<BigEndian>()))
    }

    fn read_i64<T: Transport>(&self, transport: &mut T) -> Result<i64> {
        Ok(try!(transport.read_i64::<BigEndian>()))
    }

    fn read_double<T: Transport>(&self, transport: &mut T) -> Result<f64> {
        Ok(try!(transport.read_f64::<BigEndian>()))
    }

    fn read_string<T: Transport>(&self, transport: &mut T) -> Result<String> {
        let bytes = try!(self.read_binary(transport));
        Ok(try!(String::from_utf8(bytes).map_err(|e| protocol::Error::from(e.utf8_error()))))
    }

    fn read_binary<T: Transport>(&self, transport: &mut T) -> Result<Vec<u8>> {
        let len = try!(self.read_i32(transport)) as usize;
        Ok(try!(transport.read_exact(len)))
    }

    fn skip<T: Transport>(&self, transport: &mut T, type_: Type) -> Result<()> {
        match type_ {
            Type::Bool => { try!(self.read_bool(transport)); }
            Type::Byte => { try!(self.read_byte(transport)); }
            Type::I16 => { try!(self.read_i16(transport)); }
            Type::I32 => { try!(self.read_i32(transport)); }
            Type::I64 => { try!(self.read_i64(transport)); }
            Type::Double => { try!(self.read_double(transport)); }
            Type::String => { try!(self.read_binary(transport)); }
            Type::Struct => {
                try!(self.read_struct_begin(transport));
                loop {
                    let (_, field_type, _) = try!(self.read_field_begin(transport));
                    if field_type == Type::Stop {
                        break;
                    }
                    try!(self.skip(transport, field_type));
                    try!(self.read_field_end(transport));
                }
                try!(self.read_struct_end(transport));
            }
            Type::Map => {
                let (key_type, value_type, size) = try!(self.read_map_begin(transport));
                for _ in 0..size {
                    try!(self.skip(transport, key_type));
                    try!(self.skip(transport, value_type));
                }
                try!(self.read_map_end(transport));
            }
            Type::Set => {
                let (elem_type, size) = try!(self.read_set_begin(transport));
                for _ in 0..size {
                    try!(self.skip(transport, elem_type));
                }
                try!(self.read_set_end(transport));
            }
            Type::List => {
                let (elem_type, size) = try!(self.read_list_begin(transport));
                for _ in 0..size {
                    try!(self.skip(transport, elem_type));
                }
                try!(self.read_list_end(transport));
            }
            Type::Void => { }
            Type::Stop => { }
        };

        Ok(())
    }
}

#[cfg(test)]
pub mod test;
