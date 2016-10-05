pub use protocol::{self, Encode, Decode, Type, ThriftTyped};
pub use {Protocol, Transport, Result, Error};

pub use std::collections::{BTreeSet, BTreeMap};

use protocol::helpers::typ;
use rt::OrderedFloat;

impl ThriftTyped for bool { fn typ(&self) -> Type { Type::Bool } }
impl ThriftTyped for i8  { fn typ(&self) -> Type { Type::Byte } }
impl ThriftTyped for i16 { fn typ(&self) -> Type { Type::I16 } }
impl ThriftTyped for i32 { fn typ(&self) -> Type { Type::I32 } }
impl ThriftTyped for i64 { fn typ(&self) -> Type { Type::I64 } }
impl ThriftTyped for f64 { fn typ(&self) -> Type { Type::Double } }
impl ThriftTyped for () { fn typ(&self) -> Type { Type::Void } }
impl ThriftTyped for String { fn typ(&self) -> Type { Type::String } }
impl ThriftTyped for Vec<u8> { fn typ(&self) -> Type { Type::String } }
impl ThriftTyped for OrderedFloat<f64> { fn typ(&self) -> Type { Type::Double } }
impl<T: ThriftTyped> ThriftTyped for Vec<T> { fn typ(&self) -> Type { Type::List } }
impl<T: ThriftTyped + Default> ThriftTyped for Option<T> { fn typ(&self) -> Type { typ::<T>() } }
impl<T: ThriftTyped> ThriftTyped for BTreeSet<T> { fn typ(&self) -> Type { Type::Set } }
impl<K: ThriftTyped, V: ThriftTyped> ThriftTyped for BTreeMap<K, V> { fn typ(&self) -> Type { Type::Map } }

impl<X: Encode + Default> Encode for Vec<X> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_list_begin(transport, typ::<X>(), self.len()));

        for el in self {
            try!(el.encode(protocol, transport));
        }

        try!(protocol.write_list_end(transport));

        Ok(())
    }
}

impl<X: Encode + Ord + Default> Encode for BTreeSet<X> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_set_begin(transport, typ::<X>(), self.len()));

        for el in self {
            try!(el.encode(protocol, transport));
        }

        try!(protocol.write_set_end(transport));

        Ok(())
    }
}

impl<K: Encode + Ord + Default, V: Encode + Default> Encode for BTreeMap<K, V> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_map_begin(transport, typ::<K>(), typ::<V>(), self.len()));

        for (k, v) in self.iter() {
            try!(k.encode(protocol, transport));
            try!(v.encode(protocol, transport));
        }

        try!(protocol.write_map_end(transport));

        Ok(())
    }
}

impl<X: Encode + Default> Encode for Option<X> {
    fn should_encode(&self) -> bool {
        match *self {
            Some(ref v) => v.should_encode(),
            None => false,
        }
    }

    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        self.as_ref().map(|this| this.encode(protocol, transport)).unwrap_or(Ok(()))
    }
}

impl Encode for String {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_string(transport, self));
        Ok(())
    }
}

impl Encode for Vec<u8> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_binary(transport, self));
        Ok(())
    }
}

impl Encode for OrderedFloat<f64> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let d = self.as_ref();
        try!(protocol.write_double(transport, *d));
        Ok(())
    }
}

impl Encode for () {
    fn should_encode(&self) -> bool {
        false
    }

    fn encode<P, T>(&self, _: &mut P, _: &mut T) -> Result<()>
    where P: Protocol, T: Transport { Ok(()) }
}

macro_rules! prim_encode {
    ($($T:ty => $method:ident),*) => {
        $(impl Encode for $T {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
            where P: Protocol, T: Transport {
                try!(protocol.$method(transport, *self));
                Ok(())
            }
        })*
    }
}

prim_encode! {
    bool => write_bool, i8 => write_byte, i16 => write_i16,
    i32 => write_i32, i64 => write_i64, f64 => write_double
}

fn decode<D, P, T>(protocol: &mut P, transport: &mut T) -> Result<D>
where D: Decode, P: Protocol, T: Transport {
     let mut elem = D::default();
     try!(elem.decode(protocol, transport));
     Ok(elem)
}

impl Decode for OrderedFloat<f64> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let d = try!(protocol.read_double(transport));
        *self = From::from(d);
        Ok(())
    }
}

impl<X: Decode> Decode for Vec<X> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let (type_, len) = try!(protocol.read_list_begin(transport));

        if type_ == typ::<X>() {
            self.reserve(len as usize);
            for _ in 0..len { self.push(try!(decode(protocol, transport))); }
            try!(protocol.read_list_end(transport));
            Ok(())
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation))
        }
    }
}

impl<X: Decode + Ord> Decode for BTreeSet<X> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let (type_, len) = try!(protocol.read_set_begin(transport));

        if type_ == typ::<X>() {
            for _ in 0..len { self.insert(try!(decode(protocol, transport))); }
            try!(protocol.read_set_end(transport));
            Ok(())
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation))
        }
    }
}

impl<K: Decode + Ord, V: Decode> Decode for BTreeMap<K, V> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let (ktyp, vtyp, len) = try!(protocol.read_map_begin(transport));

        if ktyp == typ::<K>() && vtyp == typ::<V>() {
            for _ in 0..len {
                let key = try!(decode(protocol, transport));
                let value = try!(decode(protocol, transport));
                self.insert(key, value);
            }

            try!(protocol.read_map_end(transport));
            Ok(())
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation))
        }
    }
}

impl<X: Decode> Decode for Option<X> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let mut this = X::default();
        try!(this.decode(protocol, transport));
        *self = Some(this);
        Ok(())
    }
}

impl Decode for () {
    fn decode<P, T>(&mut self, _: &mut P, _: &mut T) -> Result<()>
    where P: Protocol, T: Transport { Ok(()) }
}

macro_rules! prim_decode {
    ($($T:ty => $method:ident),*) => {
        $(impl Decode for $T {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
            where P: Protocol, T: Transport {
                *self = try!(protocol.$method(transport));
                Ok(())
            }
        })*
    }
}

prim_decode! {
    bool => read_bool, i8 => read_byte, i16 => read_i16,
    i32 => read_i32, i64 => read_i64, f64 => read_double,
    String => read_string,
    Vec<u8> => read_binary
}

