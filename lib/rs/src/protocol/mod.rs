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
use std::{str, fmt};
use std::error::Error as StdError;

use transport::Transport;
use Result;

pub mod binary_protocol;
pub mod compact_protocol;

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Protocol version mismatch
    BadVersion,
    /// Sender violated the protocol, for instance, sent an unknown enum value
    ProtocolViolation,
    /// Received string cannot be converted to a UTF8 string
    InvalidUtf8(str::Utf8Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        "Thrift Protocol Error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
             Error::InvalidUtf8(ref e) => Some(e),
             _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::InvalidUtf8(e)
    }
}

pub trait ProtocolFactory {
    type Protocol: Protocol;

    fn new_protocol(&self) -> Self::Protocol;
}

impl<F, P: Protocol> ProtocolFactory for F where F: Fn() -> P {
    type Protocol = P;

    fn new_protocol(&self) -> P {
        (*self)()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Type {
    Stop = 0x00,
    Void = 0x01,
    Bool = 0x02,
    Byte = 0x03,
    Double = 0x04,
    I16 = 0x06,
    I32 = 0x08,
    I64 = 0x0a,
    String = 0x0b,
    Struct = 0x0c,
    Map = 0x0d,
    Set = 0x0e,
    List = 0x0f
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Type::Stop => "Stop",
            Type::Void => "Void",
            Type::Bool => "Bool",
            Type::Byte => "Byte",
            Type::Double => "Double",
            Type::I16 => "I16",
            Type::I32 => "I32",
            Type::I64 => "I64",
            Type::String => "String",
            Type::Struct => "Struct",
            Type::Map => "Map",
            Type::Set => "Set",
            Type::List => "List"
        })
    }
}

impl ::std::str::FromStr for Type {
    type Err = ();

    fn from_str(string: &str) -> ::std::result::Result<Type, ()> {
        Ok(match string {
            "Stop" => Type::Stop,
            "Void" => Type::Void,
            "Bool" => Type::Bool,
            "Byte" => Type::Byte,
            "Double" => Type::Double,
            "I16" => Type::I16,
            "I32" => Type::I32,
            "I64" => Type::I64,
            "String" => Type::String,
            "Struct" => Type::Struct,
            "Map" => Type::Map,
            "Set" => Type::Set,
            "List" => Type::List,
            _ => return Err(())
        })
    }
}

impl Type {
    pub fn from_num(num: u64) -> Option<Type> {
        match num {
            0x00 => Some(Type::Stop),
            0x01 => Some(Type::Void),
            0x02 => Some(Type::Bool),
            0x03 => Some(Type::Byte),
            0x04 => Some(Type::Double),
            0x06 => Some(Type::I16),
            0x08 => Some(Type::I32),
            0x0a => Some(Type::I64),
            0x0b => Some(Type::String),
            0x0c => Some(Type::Struct),
            0x0d => Some(Type::Map),
            0x0e => Some(Type::Set),
            0x0f => Some(Type::List),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MessageType {
    Call = 0x01,
    Reply = 0x02,
    Exception = 0x03,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            MessageType::Call => "Call",
            MessageType::Reply => "Reply",
            MessageType::Exception => "Exception"
        })
    }
}

impl ::std::str::FromStr for MessageType {
    type Err = ();

    fn from_str(string: &str) -> ::std::result::Result<MessageType, ()> {
        Ok(match string {
            "Call" => MessageType::Call,
            "Reply" => MessageType::Reply,
            "Exception" => MessageType::Exception,
            _ => return Err(())
        })
    }
}

impl MessageType {
    pub fn from_num(num: u64) -> Option<MessageType> {
        match num {
            0x01 => Some(MessageType::Call),
            0x02 => Some(MessageType::Reply),
            0x03 => Some(MessageType::Exception),
            _ => None,
        }
    }
}

pub trait ThriftTyped {
    fn typ(&self) -> Type;
}

pub trait Encode: ThriftTyped {
    fn encode<P, T>(&self, &mut P, &mut T) -> Result<()>
    where P: Protocol, T: Transport;

    fn should_encode(&self) -> bool { true }
}

pub trait Decode: ThriftTyped + Default {
    fn decode<P, T>(&mut self, &mut P, &mut T) -> Result<()>
    where P: Protocol, T: Transport;
}

pub trait Protocol {
    fn write_message_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        name: &str,
        message_type: MessageType,
        sequence_id: i32
    ) -> Result<()>;
    fn write_message_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn write_struct_begin<T: Transport>(&mut self, transport: &mut T, name: &str) -> Result<()>;
    fn write_struct_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn write_field_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        name: &str,
        field_type: Type,
        field_id: i16
    ) -> Result<()>;
    fn write_field_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;
    fn write_field_stop<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn write_map_begin<T: Transport>(
        &mut self,
        transport: &mut T,
        key_type: Type,
        value_type: Type,
        size: usize
    ) -> Result<()>;
    fn write_map_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn write_list_begin<T: Transport>(&mut self, transport: &mut T, elem_type: Type, size: usize) -> Result<()>;
    fn write_list_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn write_set_begin<T: Transport>(&mut self, transport: &mut T, elem_type: Type, size: usize) -> Result<()>;
    fn write_set_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn write_bool<T: Transport>(&mut self, transport: &mut T, value: bool) -> Result<()>;
    fn write_byte<T: Transport>(&mut self, transport: &mut T, value: i8) -> Result<()>;
    fn write_i16<T: Transport>(&mut self, transport: &mut T, value: i16) -> Result<()>;
    fn write_i32<T: Transport>(&mut self, transport: &mut T, value: i32) -> Result<()>;
    fn write_i64<T: Transport>(&mut self, transport: &mut T, value: i64) -> Result<()>;
    fn write_double<T: Transport>(&mut self, transport: &mut T, value: f64) -> Result<()>;
    fn write_str<T: Transport>(&mut self, transport: &mut T, value: &str) -> Result<()>;
    fn write_string<T: Transport>(&mut self, transport: &mut T, value: &String) -> Result<()>;
    fn write_binary<T: Transport>(&mut self, transport: &mut T, value: &[u8]) -> Result<()>;

    fn read_message_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(String, MessageType, i32)>;
    fn read_message_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn read_struct_begin<T: Transport>(&mut self, transport: &mut T) -> Result<String>;
    fn read_struct_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn read_field_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(String, Type, i16)>;
    fn read_field_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn read_map_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, Type, i32)>;
    fn read_map_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn read_list_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)>;
    fn read_list_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn read_set_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)>;
    fn read_set_end<T: Transport>(&mut self, transport: &mut T) -> Result<()>;

    fn read_bool<T: Transport>(&mut self, transport: &mut T) -> Result<bool>;
    fn read_byte<T: Transport>(&mut self, transport: &mut T) -> Result<i8>;
    fn read_i16<T: Transport>(&mut self, transport: &mut T) -> Result<i16>;
    fn read_i32<T: Transport>(&mut self, transport: &mut T) -> Result<i32>;
    fn read_i64<T: Transport>(&mut self, transport: &mut T) -> Result<i64>;
    fn read_double<T: Transport>(&mut self, transport: &mut T) -> Result<f64>;
    fn read_string<T: Transport>(&mut self, transport: &mut T) -> Result<String>;
    fn read_binary<T: Transport>(&mut self, transport: &mut T) -> Result<Vec<u8>>;

    fn skip<T: Transport>(&mut self, transport: &mut T, type_: Type) -> Result<()>;
}

impl<'a, T: ?Sized> ThriftTyped for &'a T where T: ThriftTyped {
    fn typ(&self) -> Type { <T as ThriftTyped>::typ(self) }
}

impl<'a, E: ?Sized> Encode for &'a E where E: Encode {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        <E as Encode>::encode(self, protocol, transport)
    }

    fn should_encode(&self) -> bool { <E as Encode>::should_encode(self) }
}

impl<'a, P: ?Sized> Protocol for &'a mut P where P: Protocol {
    fn write_message_begin<T: Transport>(&mut self, transport: &mut T, name: &str,
                           message_type: MessageType, sequence_id: i32) -> Result<()> {
        <P as Protocol>::write_message_begin(self, transport, name, message_type, sequence_id)
    }

    fn write_message_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::write_message_end(self, transport)
    }

    fn write_struct_begin<T: Transport>(&mut self, transport: &mut T, name: &str) -> Result<()> {
        <P as Protocol>::write_struct_begin(self, transport, name)
    }

    fn write_struct_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::write_struct_end(self, transport)
    }

    fn write_field_begin<T: Transport>(&mut self, transport: &mut T, name: &str,
                         field_type: Type, field_id: i16) -> Result<()> {
        <P as Protocol>::write_field_begin(self, transport, name, field_type, field_id)
    }

    fn write_field_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::write_field_end(self, transport)
    }

    fn write_field_stop<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::write_field_stop(self, transport)
    }

    fn write_map_begin<T: Transport>(&mut self, transport: &mut T, key_type: Type,
                       value_type: Type, size: usize) -> Result<()> {
        <P as Protocol>::write_map_begin(self, transport, key_type, value_type, size)
    }

    fn write_map_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::write_map_end(self, transport)
    }

    fn write_list_begin<T: Transport>(&mut self, transport: &mut T, elem_type: Type, size: usize) -> Result<()> {
        <P as Protocol>::write_list_begin(self, transport, elem_type, size)
    }

    fn write_list_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::write_list_end(self, transport)
    }

    fn write_set_begin<T: Transport>(&mut self, transport: &mut T, elem_type: Type, size: usize) -> Result<()> {
        <P as Protocol>::write_set_begin(self, transport, elem_type, size)
    }

    fn write_set_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::write_set_end(self, transport)
    }

    fn write_bool<T: Transport>(&mut self, transport: &mut T, value: bool) -> Result<()> {
        <P as Protocol>::write_bool(self, transport, value)
    }

    fn write_byte<T: Transport>(&mut self, transport: &mut T, value: i8) -> Result<()> {
         <P as Protocol>::write_byte(self, transport, value)
    }

    fn write_i16<T: Transport>(&mut self, transport: &mut T, value: i16) -> Result<()> {
        <P as Protocol>::write_i16(self, transport, value)
    }

    fn write_i32<T: Transport>(&mut self, transport: &mut T, value: i32) -> Result<()> {
        <P as Protocol>::write_i32(self, transport, value)
    }

    fn write_i64<T: Transport>(&mut self, transport: &mut T, value: i64) -> Result<()> {
        <P as Protocol>::write_i64(self, transport, value)
    }

    fn write_double<T: Transport>(&mut self, transport: &mut T, value: f64) -> Result<()> {
        <P as Protocol>::write_double(self, transport, value)
    }

    fn write_str<T: Transport>(&mut self, transport: &mut T, value: &str) -> Result<()> {
        <P as Protocol>::write_str(self, transport, value)
    }

    fn write_string<T: Transport>(&mut self, transport: &mut T, value: &String) -> Result<()> {
        <P as Protocol>::write_string(self, transport, value)
    }

    fn write_binary<T: Transport>(&mut self, transport: &mut T, value: &[u8]) -> Result<()> {
        <P as Protocol>::write_binary(self, transport, value)
    }

    fn read_message_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(String, MessageType, i32)> {
        <P as Protocol>::read_message_begin(self, transport)
    }

    fn read_message_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::read_message_end(self, transport)
    }

    fn read_struct_begin<T: Transport>(&mut self, transport: &mut T) -> Result<String> {
        <P as Protocol>::read_struct_begin(self, transport)
    }

    fn read_struct_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::read_struct_end(self, transport)
    }

    fn read_field_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(String, Type, i16)> {
        <P as Protocol>::read_field_begin(self, transport)
    }

    fn read_field_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::read_field_end(self, transport)
    }

    fn read_map_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, Type, i32)> {
        <P as Protocol>::read_map_begin(self, transport)
    }

    fn read_map_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::read_map_end(self, transport)
    }

    fn read_list_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)> {
        <P as Protocol>::read_list_begin(self, transport)
    }

    fn read_list_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::read_list_end(self, transport)
    }

    fn read_set_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)> {
        <P as Protocol>::read_set_begin(self, transport)
    }

    fn read_set_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        <P as Protocol>::read_set_end(self, transport)
    }

    fn read_bool<T: Transport>(&mut self, transport: &mut T) -> Result<bool> {
        <P as Protocol>::read_bool(self, transport)
    }

    fn read_byte<T: Transport>(&mut self, transport: &mut T) -> Result<i8> {
        <P as Protocol>::read_byte(self, transport)
    }

    fn read_i16<T: Transport>(&mut self, transport: &mut T) -> Result<i16> {
        <P as Protocol>::read_i16(self, transport)
    }

    fn read_i32<T: Transport>(&mut self, transport: &mut T) -> Result<i32> {
        <P as Protocol>::read_i32(self, transport)
    }

    fn read_i64<T: Transport>(&mut self, transport: &mut T) -> Result<i64> {
        <P as Protocol>::read_i64(self, transport)
    }

    fn read_double<T: Transport>(&mut self, transport: &mut T) -> Result<f64> {
        <P as Protocol>::read_double(self, transport)
    }

    fn read_string<T: Transport>(&mut self, transport: &mut T) -> Result<String> {
        <P as Protocol>::read_string(self, transport)
    }

    fn read_binary<T: Transport>(&mut self, transport: &mut T) -> Result<Vec<u8>> {
        <P as Protocol>::read_binary(self, transport)
    }

    fn skip<T: Transport>(&mut self, transport: &mut T, type_: Type) -> Result<()> {
        <P as Protocol>::skip(self, transport, type_)
    }
}

pub trait FromNum: Sized {
    fn from_num(num: i32) -> Option<Self>;
}

pub mod helpers {
    use protocol::{ThriftTyped, Protocol, Type, MessageType, FromNum, Decode, Encode, Error};
    use transport::Transport;
    use Result;

    pub fn typ<T: ThriftTyped + Default>() -> Type {
        T::default().typ()
    }

    pub fn read_enum<F, T, P>(iprot: &mut P, transport: &mut T) -> Result<F>
    where F: FromNum, T: Transport, P: Protocol {
        let i = try!(iprot.read_i32(transport));
        match <F as FromNum>::from_num(i) {
            Some(v) => Ok(v),
            None => Err(::Error::from(Error::ProtocolViolation)),
        }
    }

    pub fn send<W, T, P>(protocol: &mut P, transport: &mut T,
                         name: &str, _type: MessageType,
                         args: &W, cseqid: i32) -> Result<()>
    where W: Encode, T: Transport, P: Protocol {
        try!(protocol.write_message_begin(transport, name, _type, cseqid));
        try!(args.encode(protocol, transport));
        try!(protocol.write_message_end(transport));
        try!(transport.flush());
        Ok(())
    }

    pub fn receive<R, T, P>(protocol: &mut P, transport: &mut T,
                            op: &str, result: &mut R) -> Result<()>
    where R: Decode, T: Transport, P: Protocol {
        let (name, ty, id) = try!(protocol.read_message_begin(transport));
        receive_body(protocol, transport, op, result, &name, ty, id)
    }

    pub fn receive_body<R, T, P>(protocol: &mut P, transport: &mut T, op: &str,
                                 result: &mut R, name: &str, ty: MessageType,
                                 id: i32) -> Result<()>
    where R: Decode, T: Transport, P: Protocol {
        match (name, ty, id) {
            (_, MessageType::Exception, _) => {
                println!("got exception");
                // TODO
                //let x = ApplicationException;
                //x.read(&mut protocol)
                //protocol.read_message_end();
                //transport.read_end();
                //throw x
                Err(::Error::UserException)
            }
            // TODO: Make sure the client doesn't receive Call messages and that the server
            // doesn't receive Reply messages
            (fname, _, _) => {
                if &fname[..] == op {
                    try!(result.decode(protocol, transport));
                    try!(protocol.read_message_end(transport));
                    Ok(())
                 }
                else {
                    // FIXME: shall we err in this case?
                    try!(protocol.skip(transport, Type::Struct));
                    try!(protocol.read_message_end(transport));
                    Err(::Error::from(Error::ProtocolViolation))
                }
            }
        }
    }
}

