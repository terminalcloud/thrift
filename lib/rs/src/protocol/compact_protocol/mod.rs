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

#![allow(overflowing_literals)]

use protocol::{MessageType, Protocol, Type, Error};
use transport::Transport;
use Result;

use podio::{ReadPodExt, WritePodExt, LittleEndian};

const PROTOCOL_ID: i8 = 0x82;
const VERSION_N: i8 = 1;
const VERSION_MASK: i8 = 0x1f;
const TYPE_MASK: i8 = 0xE0;
const TYPE_BITS: i8 = 0x07;
const TYPE_SHIFT_AMOUNT: i32 = 5;

#[derive(PartialEq)]
enum CType {
    Stop = 0x0,
    BooleanTrue = 0x01,
    BooleanFalse = 0x02,
    Byte = 0x03,
    I16 = 0x04,
    I32 = 0x05,
    I64 = 0x06,
    Double = 0x07,
    Binary = 0x08,
    List = 0x09,
    Set = 0x0A,
    Map = 0x0B,
    Struct = 0x0C,
}

impl CType {
    fn from_type(type_: Type) -> CType {
        match type_ {
            Type::Stop => CType::Stop,
            Type::Bool => CType::BooleanTrue,
            Type::Byte => CType::Byte,
            Type::Double => CType::Double,
            Type::I16 => CType::I16,
            Type::I32 => CType::I32,
            Type::I64 => CType::I64,
            Type::String => CType::Binary,
            Type::Struct => CType::Struct,
            Type::Map => CType::Map,
            Type::Set => CType::Set,
            Type::List => CType::List,
            Type::Void => unreachable!(),
        }
    }

    fn from_u8(v: u8) -> Option<CType> {
        match v {
            0x00 => Some(CType::Stop),
            0x01 => Some(CType::BooleanTrue),
            0x02 => Some(CType::BooleanFalse),
            0x03 => Some(CType::Byte),
            0x04 => Some(CType::I16),
            0x05 => Some(CType::I32),
            0x06 => Some(CType::I64),
            0x07 => Some(CType::Double),
            0x08 => Some(CType::Binary),
            0x09 => Some(CType::List),
            0x0A => Some(CType::Set),
            0x0B => Some(CType::Map),
            0x0C => Some(CType::Struct),
            _ => None,
        }
    }

    fn to_type(&self) -> Type {
        match *self {
            CType::Stop => Type::Stop,
            CType::BooleanTrue | CType::BooleanFalse => Type::Bool,
            CType::Byte => Type::Byte,
            CType::I16 => Type::I16,
            CType::I32 => Type::I32,
            CType::I64 => Type::I64,
            CType::Double => Type::Double,
            CType::Binary => Type::String,
            CType::List => Type::List,
            CType::Set => Type::Set,
            CType::Map => Type::Map,
            CType::Struct => Type::Struct,
        }
    }
}

#[derive(Debug, Clone)]
struct Field {
    name: String,
    field_type: Type,
    field_id: i16,
}

#[derive(Debug, Clone)]
pub struct CompactProtocol {
    bool_field: Option<Field>,
    bool_value: Option<bool>,
    last_field: Vec<i16>,
    last_field_id: i16,
}

fn i32_to_zigzag(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

fn i64_to_zigzag(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

fn zigzag_to_i32(n: u32) -> i32 {
    ((n >> 1) ^ (-((n & 1) as i32)) as u32) as i32
}

fn zigzag_to_i64(n: u64) -> i64 {
    ((n >> 1) ^ (-((n & 1) as i64)) as u64) as i64
}

impl CompactProtocol {
    pub fn new() -> CompactProtocol {
        CompactProtocol {
            bool_field: None,
            bool_value: None,
            last_field: vec![],
            last_field_id: 0,
        }
    }

    fn write_field_begin_internal<T: Transport>(
        &mut self,
        transport: &mut T,
        _name: &str,
        field_type: Type,
        field_id: i16,
        type_override: Option<CType>
    ) -> Result<()> {
        let type_to_write = if let Some(type_) = type_override {
            type_ as i8
        } else {
            CType::from_type(field_type) as i8
        };

        if field_id > self.last_field_id && field_id - self.last_field_id <= 15 {
            let last_field_id = self.last_field_id;
            try!(self.write_byte(transport, ((field_id - last_field_id) as i8) << 4 | type_to_write));
        } else {
            try!(self.write_byte(transport, type_to_write));
            try!(self.write_i16(transport, field_id));
        }

        self.last_field_id = field_id;
        Ok(())
    }

    fn write_collection_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        elem_type: Type,
        size: usize
    ) -> Result<()> {
        let elem_type = CType::from_type(elem_type) as i8;
        if size < 14 {
            self.write_byte(transport, (size as i8) << 4 | elem_type)
        } else if size <= i32::max_value() as usize {
            self.write_byte(transport, 0xf0 | elem_type)
        } else {
            Err(Error::ProtocolViolation.into())
        }
    }

    fn write_var_i32<T: Transport>(
        &mut self,
        transport: &mut T,
        n: i32
    ) -> Result<()> {
        let mut n = n as u32;
        let mut buf = [0; 5];
        let mut wsize = 0;

        loop {
            if n & !0x7F == 0 {
                buf[wsize] = n as u8;
                wsize += 1;
                break;
            } else {
                buf[wsize] = (n as u8 & 0x7F) | 0x80;
                wsize += 1;
                n >>= 7;
            }
        }
        Ok(try!(transport.write_all(&buf[..wsize])))
    }

    fn write_var_i64<T: Transport>(
        &mut self,
        transport: &mut T,
        n: i64
    ) -> Result<()> {
        let mut n = n as u64;
        let mut buf = [0; 10];
        let mut wsize = 0;

        loop {
            if n & !0x7F == 0 {
                buf[wsize] = n as u8;
                wsize += 1;
                break;
            } else {
                buf[wsize] = (n as u8 & 0x7F) | 0x80;
                wsize += 1;
                n >>= 7;
            }
        }
        Ok(try!(transport.write_all(&buf[..wsize])))
    }

    fn read_var_i32<T: Transport>(
        &mut self,
        transport: &mut T
    ) -> Result<i32> {
        self.read_var_i64(transport).map(|v| v as i32)
    }

    fn read_var_i64<T: Transport>(
        &mut self,
        transport: &mut T
    ) -> Result<i64> {
        let mut val = 0;
        let mut shift = 0;
        loop {
            let byte = try!(transport.read_u8());
            val |= ((byte & 0x7f) as i64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                return Ok(val);
            }
        }
    }
}

impl Protocol for CompactProtocol {
    fn write_message_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        name: &str,
        message_type: MessageType,
        sequence_id: i32
    ) -> Result<()> {
        try!(self.write_byte(transport, PROTOCOL_ID));
        try!(self.write_byte(transport,
                             (VERSION_N & VERSION_MASK) |
                                (((message_type as i8) << TYPE_SHIFT_AMOUNT) & TYPE_MASK)));
        try!(self.write_var_i32(transport, sequence_id));
        self.write_str(transport, name)
    }

    fn write_message_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_struct_begin<T: Transport>(&mut self, _transport: &mut T, _name: &str) -> Result<()> {
        self.last_field.push(self.last_field_id);
        self.last_field_id = 0;
        Ok(())
    }

    fn write_struct_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        self.last_field_id = self.last_field.pop().unwrap();
        Ok(())
    }

    fn write_field_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        name: &str,
        field_type: Type,
        field_id: i16
    ) -> Result<()> {
        if field_type == Type::Bool {
            self.bool_field = Some(Field {
                name: name.into(),
                field_type: field_type,
                field_id: field_id,
            });
            Ok(())
        } else {
            self.write_field_begin_internal(transport, name, field_type, field_id, None)
        }
    }

    fn write_field_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_field_stop<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        self.write_byte(transport, Type::Stop as i8)
    }

    fn write_map_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        key_type: Type,
        value_type: Type,
        size: usize
    ) -> Result<()> {
        if size == 0 {
            self.write_byte(transport, 0)
        } else if size > i32::max_value() as usize {
            Err(Error::ProtocolViolation.into())
        } else {
            try!(self.write_var_i32(transport, size as i32));
            self.write_byte(transport, (CType::from_type(key_type) as i8) << 4 |
                                CType::from_type(value_type) as i8)
        }
    }

    fn write_map_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_list_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        elem_type: Type,
        size: usize
    ) -> Result<()> {
        self.write_collection_begin(transport, elem_type, size)
    }

    fn write_list_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_set_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        elem_type: Type,
        size: usize
    ) -> Result<()> {
        self.write_collection_begin(transport, elem_type, size)
    }

    fn write_set_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn write_bool<T: Transport>(
        &mut self,
        transport: &mut T,
        value: bool
    ) -> Result<()> {
        let value = if value {
            CType::BooleanTrue
        } else {
            CType::BooleanFalse
        };

        match self.bool_field.take() {
            Some(field) => {
                self.write_field_begin_internal(transport,
                                                &field.name,
                                                field.field_type,
                                                field.field_id,
                                                Some(value))
            }
            None => self.write_byte(transport, value as i8)
        }
    }

    fn write_byte<T: Transport>(
        &mut self,
        transport: &mut T,
        value: i8
    ) -> Result<()> {
        Ok(try!(transport.write_i8(value)))
    }

    fn write_i16<T: Transport>(
        &mut self,
        transport: &mut T,
        value: i16
    ) -> Result<()> {
        self.write_i32(transport, value as i32)
    }

    fn write_i32<T: Transport>(
        &mut self,
        transport: &mut T,
        value: i32
    ) -> Result<()> {
        self.write_var_i32(transport, i32_to_zigzag(value) as i32)
    }

    fn write_i64<T: Transport>(
        &mut self,
        transport: &mut T,
        value: i64
    ) -> Result<()> {
        self.write_var_i64(transport, i64_to_zigzag(value) as i64)
    }

    fn write_double<T: Transport>(
        &mut self,
        transport: &mut T,
        value: f64
    ) -> Result<()> {
        Ok(try!(transport.write_f64::<LittleEndian>(value)))
    }

    fn write_str<T: Transport>(
        &mut self,
        transport: &mut T,
        value: &str
    ) -> Result<()> {
        self.write_binary(transport, value.as_bytes())
    }

    fn write_string<T: Transport>(
        &mut self,
        transport: &mut T,
        value: &String
    ) -> Result<()> {
        self.write_str(transport, value)
    }

    fn write_binary<T: Transport>(
        &mut self,
        transport: &mut T,
        value: &[u8]
    ) -> Result<()> {
        if value.len() > i32::max_value() as usize {
            return Err(Error::ProtocolViolation.into());
        }

        try!(self.write_var_i32(transport, value.len() as i32));
        Ok(try!(transport.write_all(value)))
    }

    fn read_message_begin<T: Transport>(
        &mut self,
        transport: &mut T
    ) -> Result<(String, MessageType, i32)> {
        let protocol_id = try!(self.read_byte(transport));
        if protocol_id != PROTOCOL_ID {
            return Err(Error::ProtocolViolation.into());
        }

        let version_and_type = try!(self.read_byte(transport));
        let version = version_and_type & VERSION_MASK;
        if version != VERSION_N {
            return Err(Error::ProtocolViolation.into());
        }

        let message_type = (version_and_type >> TYPE_SHIFT_AMOUNT) & TYPE_BITS;
        let message_type = match MessageType::from_num(message_type as u64) {
            Some(message_type) => message_type,
            None => return Err(Error::ProtocolViolation.into())
        };

        let sequence_id = try!(self.read_var_i32(transport));
        let name = try!(self.read_string(transport));

        Ok((name, message_type, sequence_id))
    }

    fn read_message_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_struct_begin<T: Transport>(
        &mut self,
        _transport: &mut T
    ) -> Result<String> {
        self.last_field.push(self.last_field_id);
        self.last_field_id = 0;
        Ok("".into())
    }

    fn read_struct_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        self.last_field_id = self.last_field.pop().unwrap();
        Ok(())
    }

    fn read_field_begin<T: Transport>(
        &mut self,
        transport: &mut T
    ) -> Result<(String, Type, i16)> {
        let byte = try!(self.read_byte(transport)) as u8;
        let type_ = match CType::from_u8(byte & 0x0f) {
            Some(type_) => type_,
            None => return Err(Error::ProtocolViolation.into()),
        };

        if type_ == CType::Stop {
            return Ok(("".into(), Type::Stop, 0));
        }

        let modifier = ((byte & 0xf0) >> 4) as i16;
        let field_id = if modifier == 0 {
            try!(self.read_i16(transport))
        } else {
            self.last_field_id + modifier
        };

        match type_ {
            CType::BooleanTrue => self.bool_value = Some(true),
            CType::BooleanFalse => self.bool_value = Some(false),
            _ => {}
        }

        self.last_field_id = field_id;
        Ok(("".into(), type_.to_type(), field_id))
    }
    fn read_field_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_map_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, Type, i32)> {
        let msize = try!(self.read_var_i32(transport));
        if msize == 0 {
            return Ok((Type::Void, Type::Void, msize));
        }

        if msize < 0 {
            return Err(Error::ProtocolViolation.into());
        }

        let kvtype = try!(self.read_byte(transport)) as u8;
        let key_type = match CType::from_u8(kvtype >> 4) {
            Some(key_type) => key_type.to_type(),
            None => return Err(Error::ProtocolViolation.into()),
        };
        let value_type = match CType::from_u8(kvtype & 0xf) {
            Some(value_type) => value_type.to_type(),
            None => return Err(Error::ProtocolViolation.into()),
        };

        Ok((key_type, value_type, msize))
    }

    fn read_map_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_list_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)> {
        let size_and_type = try!(self.read_byte(transport)) as u8;
        let mut lsize = ((size_and_type >> 4) & 0xf) as i32;
        if lsize == 15 {
            lsize = try!(self.read_var_i32(transport));
        }

        if lsize < 0 {
            return Err(Error::ProtocolViolation.into());
        }

        let elem_type = match CType::from_u8(size_and_type & 0xf) {
            Some(elem_type) => elem_type.to_type(),
            None => return Err(Error::ProtocolViolation.into()),
        };

        Ok((elem_type, lsize))
    }

    fn read_list_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_set_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)> {
        self.read_list_begin(transport)
    }

    fn read_set_end<T: Transport>(&mut self, _transport: &mut T) -> Result<()> {
        Ok(())
    }

    fn read_bool<T: Transport>(&mut self, transport: &mut T) -> Result<bool> {
        match self.bool_value.take() {
            Some(value) => Ok(value),
            None => {
                let value = try!(self.read_byte(transport));
                Ok(value == CType::BooleanTrue as i8)
            }
        }
    }

    fn read_byte<T: Transport>(&mut self, transport: &mut T) -> Result<i8> {
        Ok(try!(transport.read_i8()))
    }

    fn read_i16<T: Transport>(&mut self, transport: &mut T) -> Result<i16> {
        let rsize = try!(self.read_var_i32(transport));
        Ok(zigzag_to_i32(rsize as u32) as i16)
    }

    fn read_i32<T: Transport>(&mut self, transport: &mut T) -> Result<i32> {
        let rsize = try!(self.read_var_i32(transport));
        Ok(zigzag_to_i32(rsize as u32))
    }

    fn read_i64<T: Transport>(&mut self, transport: &mut T) -> Result<i64> {
        let rsize = try!(self.read_var_i64(transport));
        Ok(zigzag_to_i64(rsize as u64))
    }

    fn read_double<T: Transport>(&mut self, transport: &mut T) -> Result<f64> {
        Ok(try!(transport.read_f64::<LittleEndian>()))
    }

    fn read_string<T: Transport>(&mut self, transport: &mut T) -> Result<String> {
        self.read_binary(transport)
            .and_then(|b| String::from_utf8(b)
                      .map_err(|e| Error::InvalidUtf8(e.utf8_error()).into()))
    }

    fn read_binary<T: Transport>(&mut self, transport: &mut T) -> Result<Vec<u8>> {
        let rsize = try!(self.read_var_i32(transport));
        if rsize < 0 {
            return Err(Error::ProtocolViolation.into());
        }

        Ok(try!(transport.read_exact(rsize as usize)))
    }

    fn skip<T: Transport>(&mut self, _transport: &mut T, _type_: Type) -> Result<()> {
        Ok(())
    }
}
