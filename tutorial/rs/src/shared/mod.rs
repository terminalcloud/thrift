///////////////////////////////////////////////////////////////
// Autogenerated by Thrift Compiler (1.0.0-dev)
//
// DO NOT EDIT UNLESS YOU ARE SURE YOU KNOW WHAT YOU ARE DOING
///////////////////////////////////////////////////////////////

#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};
use thrift::protocol::{MessageType, Type};
use thrift::transport::Transport;
use thrift::protocol::Protocol;
use thrift::protocol::{Readable, Writeable};
use thrift::TResult;
use thrift::ThriftErr;
use thrift::ThriftErr::*;
use std::num::FromPrimitive;
use thrift::protocol::ProtocolHelpers;


#[allow(dead_code)]
#[derive(Show)]
pub struct SharedStruct {
  pub key: i32,
  pub value: String,
}

impl SharedStruct {
  pub fn new() -> SharedStruct {
    SharedStruct {
      key: 0,
      value: String::new(),
    }
  }
}

impl Writeable for SharedStruct {

  #[allow(unused_variables)]
  #[allow(dead_code)]
  fn write(&self, oprot: &Protocol, transport: &mut Transport) -> TResult<()> {
    oprot.write_struct_begin(transport, "SharedStruct");

    oprot.write_field_begin(transport, "key", Type::TI32, 1);
    oprot.write_i32(transport, self.key);
    oprot.write_field_end(transport);
    
    oprot.write_field_begin(transport, "value", Type::TString, 2);
    oprot.write_string(transport, &self.value);
    oprot.write_field_end(transport);
    
    oprot.write_field_stop(transport);
    oprot.write_struct_end(transport);
    Ok(())
  }

}

impl Readable for SharedStruct {

  fn read(& mut self, iprot: &Protocol, transport: & mut Transport) -> TResult<()> {
    let mut have_result = false;
    iprot.read_struct_begin(transport);
    loop {
      match try!(iprot.read_field_begin(transport)) {
        (_, Type::TStop, _) => {
          try!(iprot.read_field_end(transport));
          break;
        }
        (_, Type::TI32, 1) => {
          self.key = try!(iprot.read_i32(transport));
          have_result = true;
        }
        (_, Type::TString, 2) => {
          self.value = try!(iprot.read_string(transport));
          have_result = true;
        }
        (_, ftype, _) => {
          try!(iprot.skip(transport, ftype));
        }
      }
      try!(iprot.read_field_end(transport));
    }
    try!(iprot.read_struct_end(transport));
    if have_result { Ok(()) } else { Err(ProtocolError) }
  }
}

#[allow(dead_code)]
#[derive(Show)]
pub struct SharedServiceGetStructArgs {
  pub key: i32,
}

impl Writeable for SharedServiceGetStructArgs {

  #[allow(unused_variables)]
  #[allow(dead_code)]
  fn write(&self, oprot: &Protocol, transport: &mut Transport) -> TResult<()> {
    oprot.write_struct_begin(transport, "SharedService_getStruct_args");

    oprot.write_field_begin(transport, "key", Type::TI32, 1);
    oprot.write_i32(transport, self.key);
    oprot.write_field_end(transport);
    
    oprot.write_field_stop(transport);
    oprot.write_struct_end(transport);
    Ok(())
  }

}

#[allow(dead_code)]
#[derive(Show)]
pub struct SharedServiceGetStructResult {
  pub success: SharedStruct,
}

impl SharedServiceGetStructResult {
  pub fn new() -> SharedServiceGetStructResult {
    SharedServiceGetStructResult {
      success: SharedStruct::new(),
    }
  }
}

impl Readable for SharedServiceGetStructResult {

  fn read(& mut self, iprot: &Protocol, transport: & mut Transport) -> TResult<()> {
    let mut have_result = false;
    iprot.read_struct_begin(transport);
    loop {
      match try!(iprot.read_field_begin(transport)) {
        (_, Type::TStop, _) => {
          try!(iprot.read_field_end(transport));
          break;
        }
        (_, Type::TStruct, 0) => {
          try!(self.success.read(iprot, transport));
          have_result = true;
        }
        (_, ftype, _) => {
          try!(iprot.skip(transport, ftype));
        }
      }
      try!(iprot.read_field_end(transport));
    }
    try!(iprot.read_struct_end(transport));
    if have_result { Ok(()) } else { Err(ProtocolError) }
  }
}

pub trait SharedServiceClient {
  #[allow(non_snake_case)]
  fn getStruct(
    &mut self,
    key: i32,
    ) -> TResult<SharedStruct>;
}

pub struct SharedServiceClientImpl<P: Protocol, T: Transport> {
  pub protocol: P,
  pub transport: T,
}

impl <P: Protocol, T: Transport> SharedServiceClientImpl<P, T> {
  pub fn new(protocol: P, transport: T) -> SharedServiceClientImpl<P, T> {
    SharedServiceClientImpl {
      protocol: protocol,
      transport: transport,
    }
  }
}

impl <P: Protocol, T: Transport> SharedServiceClient for SharedServiceClientImpl<P, T> {

  #[allow(non_snake_case)]
  fn getStruct(
    &mut self,
    key: i32,
    ) -> TResult<SharedStruct> {
      let args = SharedServiceGetStructArgs {
      key: key,
      };
      try!(ProtocolHelpers::send(&self.protocol, &mut self.transport, "getStruct", MessageType::MtCall, &args));
      let mut result = SharedServiceGetStructResult::new();
      try!(ProtocolHelpers::receive(&self.protocol, &mut self.transport, "getStruct", &mut result));
      Ok(result.success)
  }

}
