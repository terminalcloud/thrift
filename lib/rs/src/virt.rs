use protocol::{ThriftTyped, Encode, Protocol, Type, MessageType};
use transport::Transport;

use {Result};

pub type VirtualEncodeObject<'e> = &'e for<'p, 't> VirtualEncode<VirtualProtocolObject<'p>, &'t mut Transport>;
pub type VirtualProtocolObject<'p> = &'p mut for<'t> VirtualProtocol<&'t mut Transport>;

pub trait VirtualEncode<P, T>: ThriftTyped {
    fn virt_encode(&self, P, T) -> Result<()>;

    fn should_encode(&self) -> bool { true }
}

impl<E, P, T> VirtualEncode<P, T> for E
where E: Encode, P: Protocol, T: Transport {
    fn virt_encode(&self, mut protocol: P, mut transport: T) -> Result<()> {
        self.encode(&mut protocol, &mut transport)
    }

    fn should_encode(&self) -> bool { Encode::should_encode(self) }
}

impl<'e> Encode for VirtualEncodeObject<'e> {
    fn encode<'p, 't, P1, T1>(&self, mut protocol: &'p mut P1, mut transport: &'t mut T1) -> Result<()>
    where P1: Protocol, T1: Transport {
        let protocol: VirtualProtocolObject<'p> = protocol;
        self.virt_encode(protocol, &mut transport)
    }
}

pub trait VirtualProtocol<T> {
    fn virt_write_message_begin(
        &mut self,
        transport: T,
        name: &str,
        message_type: MessageType,
        sequence_id: i32
    ) -> Result<()>;
    fn virt_write_message_end(&mut self, transport: T) -> Result<()>;

    fn virt_write_struct_begin(&mut self, transport: T, name: &str) -> Result<()>;
    fn virt_write_struct_end(&mut self, transport: T) -> Result<()>;

    fn virt_write_field_begin(
        &mut self,
        transport: T,
        name: &str,
        field_type: Type,
        field_id: i16
    ) -> Result<()>;
    fn virt_write_field_end(&mut self, transport: T) -> Result<()>;
    fn virt_write_field_stop(&mut self, transport: T) -> Result<()>;

    fn virt_write_map_begin(
        &mut self,
        transport: T,
        key_type: Type,
        value_type: Type,
        size: usize
    ) -> Result<()>;
    fn virt_write_map_end(&mut self, transport: T) -> Result<()>;

    fn virt_write_list_begin(&mut self, transport: T, elem_type: Type, size: usize) -> Result<()>;
    fn virt_write_list_end(&mut self, transport: T) -> Result<()>;

    fn virt_write_set_begin(&mut self, transport: T, elem_type: Type, size: usize) -> Result<()>;
    fn virt_write_set_end(&mut self, transport: T) -> Result<()>;

    fn virt_write_bool(&mut self, transport: T, value: bool) -> Result<()>;
    fn virt_write_byte(&mut self, transport: T, value: i8) -> Result<()>;
    fn virt_write_i16(&mut self, transport: T, value: i16) -> Result<()>;
    fn virt_write_i32(&mut self, transport: T, value: i32) -> Result<()>;
    fn virt_write_i64(&mut self, transport: T, value: i64) -> Result<()>;
    fn virt_write_double(&mut self, transport: T, value: f64) -> Result<()>;
    fn virt_write_str(&mut self, transport: T, value: &str) -> Result<()>;
    fn virt_write_string(&mut self, transport: T, value: &String) -> Result<()>;
    fn virt_write_binary(&mut self, transport: T, value: &[u8]) -> Result<()>;

    fn virt_read_message_begin(&mut self, transport: T) -> Result<(String, MessageType, i32)>;
    fn virt_read_message_end(&mut self, transport: T) -> Result<()>;

    fn virt_read_struct_begin(&mut self, transport: T) -> Result<String>;
    fn virt_read_struct_end(&mut self, transport: T) -> Result<()>;

    fn virt_read_field_begin(&mut self, transport: T) -> Result<(String, Type, i16)>;
    fn virt_read_field_end(&mut self, transport: T) -> Result<()>;

    fn virt_read_map_begin(&mut self, transport: T) -> Result<(Type, Type, i32)>;
    fn virt_read_map_end(&mut self, transport: T) -> Result<()>;

    fn virt_read_list_begin(&mut self, transport: T) -> Result<(Type, i32)>;
    fn virt_read_list_end(&mut self, transport: T) -> Result<()>;

    fn virt_read_set_begin(&mut self, transport: T) -> Result<(Type, i32)>;
    fn virt_read_set_end(&mut self, transport: T) -> Result<()>;

    fn virt_read_bool(&mut self, transport: T) -> Result<bool>;
    fn virt_read_byte(&mut self, transport: T) -> Result<i8>;
    fn virt_read_i16(&mut self, transport: T) -> Result<i16>;
    fn virt_read_i32(&mut self, transport: T) -> Result<i32>;
    fn virt_read_i64(&mut self, transport: T) -> Result<i64>;
    fn virt_read_double(&mut self, transport: T) -> Result<f64>;
    fn virt_read_string(&mut self, transport: T) -> Result<String>;
    fn virt_read_binary(&mut self, transport: T) -> Result<Vec<u8>>;

    fn virt_skip(&mut self, transport: T, type_: Type) -> Result<()>;
}

impl<P, T> VirtualProtocol<T> for P where P: Protocol, T: Transport {
    fn virt_write_message_begin(&mut self, mut transport: T, name: &str,
                           message_type: MessageType, sequence_id: i32) -> Result<()> {
        Protocol::write_message_begin(self, &mut transport, name, message_type, sequence_id)
    }

    fn virt_write_message_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::write_message_end(self, &mut transport)
    }

    fn virt_write_struct_begin(&mut self, mut transport: T, name: &str) -> Result<()> {
        Protocol::write_struct_begin(self, &mut transport, name)
    }

    fn virt_write_struct_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::write_struct_end(self, &mut transport)
    }

    fn virt_write_field_begin(&mut self, mut transport: T, name: &str,
                         field_type: Type, field_id: i16) -> Result<()> {
        Protocol::write_field_begin(self, &mut transport, name, field_type, field_id)
    }

    fn virt_write_field_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::write_field_end(self, &mut transport)
    }

    fn virt_write_field_stop(&mut self, mut transport: T) -> Result<()> {
        Protocol::write_field_stop(self, &mut transport)
    }

    fn virt_write_map_begin(&mut self, mut transport: T, key_type: Type,
                       value_type: Type, size: usize) -> Result<()> {
        Protocol::write_map_begin(self, &mut transport, key_type, value_type, size)
    }

    fn virt_write_map_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::write_map_end(self, &mut transport)
    }

    fn virt_write_list_begin(&mut self, mut transport: T, elem_type: Type, size: usize) -> Result<()> {
        Protocol::write_list_begin(self, &mut transport, elem_type, size)
    }

    fn virt_write_list_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::write_list_end(self, &mut transport)
    }

    fn virt_write_set_begin(&mut self, mut transport: T, elem_type: Type, size: usize) -> Result<()> {
        Protocol::write_set_begin(self, &mut transport, elem_type, size)
    }

    fn virt_write_set_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::write_set_end(self, &mut transport)
    }

    fn virt_write_bool(&mut self, mut transport: T, value: bool) -> Result<()> {
        Protocol::write_bool(self, &mut transport, value)
    }

    fn virt_write_byte(&mut self, mut transport: T, value: i8) -> Result<()> {
         Protocol::write_byte(self, &mut transport, value)
    }

    fn virt_write_i16(&mut self, mut transport: T, value: i16) -> Result<()> {
        Protocol::write_i16(self, &mut transport, value)
    }

    fn virt_write_i32(&mut self, mut transport: T, value: i32) -> Result<()> {
        Protocol::write_i32(self, &mut transport, value)
    }

    fn virt_write_i64(&mut self, mut transport: T, value: i64) -> Result<()> {
        Protocol::write_i64(self, &mut transport, value)
    }

    fn virt_write_double(&mut self, mut transport: T, value: f64) -> Result<()> {
        Protocol::write_double(self, &mut transport, value)
    }

    fn virt_write_str(&mut self, mut transport: T, value: &str) -> Result<()> {
        Protocol::write_str(self, &mut transport, value)
    }

    fn virt_write_string(&mut self, mut transport: T, value: &String) -> Result<()> {
        Protocol::write_string(self, &mut transport, value)
    }

    fn virt_write_binary(&mut self, mut transport: T, value: &[u8]) -> Result<()> {
        Protocol::write_binary(self, &mut transport, value)
    }

    fn virt_read_message_begin(&mut self, mut transport: T) -> Result<(String, MessageType, i32)> {
        Protocol::read_message_begin(self, &mut transport)
    }

    fn virt_read_message_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::read_message_end(self, &mut transport)
    }

    fn virt_read_struct_begin(&mut self, mut transport: T) -> Result<String> {
        Protocol::read_struct_begin(self, &mut transport)
    }

    fn virt_read_struct_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::read_struct_end(self, &mut transport)
    }

    fn virt_read_field_begin(&mut self, mut transport: T) -> Result<(String, Type, i16)> {
        Protocol::read_field_begin(self, &mut transport)
    }

    fn virt_read_field_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::read_field_end(self, &mut transport)
    }

    fn virt_read_map_begin(&mut self, mut transport: T) -> Result<(Type, Type, i32)> {
        Protocol::read_map_begin(self, &mut transport)
    }

    fn virt_read_map_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::read_map_end(self, &mut transport)
    }

    fn virt_read_list_begin(&mut self, mut transport: T) -> Result<(Type, i32)> {
        Protocol::read_list_begin(self, &mut transport)
    }

    fn virt_read_list_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::read_list_end(self, &mut transport)
    }

    fn virt_read_set_begin(&mut self, mut transport: T) -> Result<(Type, i32)> {
        Protocol::read_set_begin(self, &mut transport)
    }

    fn virt_read_set_end(&mut self, mut transport: T) -> Result<()> {
        Protocol::read_set_end(self, &mut transport)
    }

    fn virt_read_bool(&mut self, mut transport: T) -> Result<bool> {
        Protocol::read_bool(self, &mut transport)
    }

    fn virt_read_byte(&mut self, mut transport: T) -> Result<i8> {
        Protocol::read_byte(self, &mut transport)
    }

    fn virt_read_i16(&mut self, mut transport: T) -> Result<i16> {
        Protocol::read_i16(self, &mut transport)
    }

    fn virt_read_i32(&mut self, mut transport: T) -> Result<i32> {
        Protocol::read_i32(self, &mut transport)
    }

    fn virt_read_i64(&mut self, mut transport: T) -> Result<i64> {
        Protocol::read_i64(self, &mut transport)
    }

    fn virt_read_double(&mut self, mut transport: T) -> Result<f64> {
        Protocol::read_double(self, &mut transport)
    }

    fn virt_read_string(&mut self, mut transport: T) -> Result<String> {
        Protocol::read_string(self, &mut transport)
    }

    fn virt_read_binary(&mut self, mut transport: T) -> Result<Vec<u8>> {
        Protocol::read_binary(self, &mut transport)
    }

    fn virt_skip(&mut self, mut transport: T, type_: Type) -> Result<()> {
        Protocol::skip(self, &mut transport, type_)
    }
}

impl<'p> Protocol for VirtualProtocolObject<'p> {
    fn write_message_begin<T: Transport>(&mut self, transport: &mut T, name: &str,
                           message_type: MessageType, sequence_id: i32) -> Result<()> {
        (*self).virt_write_message_begin(transport, name, message_type, sequence_id)
    }

    fn write_message_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_write_message_end(transport)
    }

    fn write_struct_begin<T: Transport>(&mut self, transport: &mut T, name: &str) -> Result<()> {
        (*self).virt_write_struct_begin(transport, name)
    }

    fn write_struct_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_write_struct_end(transport)
    }

    fn write_field_begin<T: Transport>(&mut self, transport: &mut T, name: &str,
                         field_type: Type, field_id: i16) -> Result<()> {
        (*self).virt_write_field_begin(transport, name, field_type, field_id)
    }

    fn write_field_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_write_field_end(transport)
    }

    fn write_field_stop<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_write_field_stop(transport)
    }

    fn write_map_begin<T: Transport>(&mut self, transport: &mut T, key_type: Type,
                       value_type: Type, size: usize) -> Result<()> {
        (*self).virt_write_map_begin(transport, key_type, value_type, size)
    }

    fn write_map_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_write_map_end(transport)
    }

    fn write_list_begin<T: Transport>(&mut self, transport: &mut T, elem_type: Type, size: usize) -> Result<()> {
        (*self).virt_write_list_begin(transport, elem_type, size)
    }

    fn write_list_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_write_list_end(transport)
    }

    fn write_set_begin<T: Transport>(&mut self, transport: &mut T, elem_type: Type, size: usize) -> Result<()> {
        (*self).virt_write_set_begin(transport, elem_type, size)
    }

    fn write_set_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_write_set_end(transport)
    }

    fn write_bool<T: Transport>(&mut self, transport: &mut T, value: bool) -> Result<()> {
        (*self).virt_write_bool(transport, value)
    }

    fn write_byte<T: Transport>(&mut self, transport: &mut T, value: i8) -> Result<()> {
         (*self).virt_write_byte(transport, value)
    }

    fn write_i16<T: Transport>(&mut self, transport: &mut T, value: i16) -> Result<()> {
        (*self).virt_write_i16(transport, value)
    }

    fn write_i32<T: Transport>(&mut self, transport: &mut T, value: i32) -> Result<()> {
        (*self).virt_write_i32(transport, value)
    }

    fn write_i64<T: Transport>(&mut self, transport: &mut T, value: i64) -> Result<()> {
        (*self).virt_write_i64(transport, value)
    }

    fn write_double<T: Transport>(&mut self, transport: &mut T, value: f64) -> Result<()> {
        (*self).virt_write_double(transport, value)
    }

    fn write_str<T: Transport>(&mut self, transport: &mut T, value: &str) -> Result<()> {
        (*self).virt_write_str(transport, value)
    }

    fn write_string<T: Transport>(&mut self, transport: &mut T, value: &String) -> Result<()> {
        (*self).virt_write_string(transport, value)
    }

    fn write_binary<T: Transport>(&mut self, transport: &mut T, value: &[u8]) -> Result<()> {
        (*self).virt_write_binary(transport, value)
    }

    fn read_message_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(String, MessageType, i32)> {
        (*self).virt_read_message_begin(transport)
    }

    fn read_message_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_read_message_end(transport)
    }

    fn read_struct_begin<T: Transport>(&mut self, transport: &mut T) -> Result<String> {
        (*self).virt_read_struct_begin(transport)
    }

    fn read_struct_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_read_struct_end(transport)
    }

    fn read_field_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(String, Type, i16)> {
        (*self).virt_read_field_begin(transport)
    }

    fn read_field_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_read_field_end(transport)
    }

    fn read_map_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, Type, i32)> {
        (*self).virt_read_map_begin(transport)
    }

    fn read_map_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_read_map_end(transport)
    }

    fn read_list_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)> {
        (*self).virt_read_list_begin(transport)
    }

    fn read_list_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_read_list_end(transport)
    }

    fn read_set_begin<T: Transport>(&mut self, transport: &mut T) -> Result<(Type, i32)> {
        (*self).virt_read_set_begin(transport)
    }

    fn read_set_end<T: Transport>(&mut self, transport: &mut T) -> Result<()> {
        (*self).virt_read_set_end(transport)
    }

    fn read_bool<T: Transport>(&mut self, transport: &mut T) -> Result<bool> {
        (*self).virt_read_bool(transport)
    }

    fn read_byte<T: Transport>(&mut self, transport: &mut T) -> Result<i8> {
        (*self).virt_read_byte(transport)
    }

    fn read_i16<T: Transport>(&mut self, transport: &mut T) -> Result<i16> {
        (*self).virt_read_i16(transport)
    }

    fn read_i32<T: Transport>(&mut self, transport: &mut T) -> Result<i32> {
        (*self).virt_read_i32(transport)
    }

    fn read_i64<T: Transport>(&mut self, transport: &mut T) -> Result<i64> {
        (*self).virt_read_i64(transport)
    }

    fn read_double<T: Transport>(&mut self, transport: &mut T) -> Result<f64> {
        (*self).virt_read_double(transport)
    }

    fn read_string<T: Transport>(&mut self, transport: &mut T) -> Result<String> {
        (*self).virt_read_string(transport)
    }

    fn read_binary<T: Transport>(&mut self, transport: &mut T) -> Result<Vec<u8>> {
        (*self).virt_read_binary(transport)
    }

    fn skip<T: Transport>(&mut self, transport: &mut T, type_: Type) -> Result<()> {
        (*self).virt_skip(transport, type_)
    }
}

fn _test_virt_impls() {
    fn _is_encode<E: Encode + ?Sized>() {}
    fn _is_protocol<P: Protocol + ?Sized>() {}
    fn _is_transport<T: Transport + ?Sized>() {}

    _is_transport::<&mut Transport>();
    _is_protocol::<&mut VirtualProtocolObject>();
    _is_encode::<&VirtualEncodeObject>();
}

