pub use protocol::{self, Encode, Decode, Type, ThriftTyped};
pub use {Protocol, Transport, Result, Error};

pub use std::collections::{HashSet, HashMap};
pub use std::hash::Hash;

impl ThriftTyped for bool { fn typ() -> Type { Type::Bool } }
impl ThriftTyped for i8  { fn typ() -> Type { Type::Byte } }
impl ThriftTyped for i16 { fn typ() -> Type { Type::I16 } }
impl ThriftTyped for i32 { fn typ() -> Type { Type::I32 } }
impl ThriftTyped for i64 { fn typ() -> Type { Type::I64 } }
impl ThriftTyped for f64 { fn typ() -> Type { Type::Double } }
impl ThriftTyped for String { fn typ() -> Type { Type::String } }
impl<T: ThriftTyped> ThriftTyped for Vec<T> { fn typ() -> Type { Type::List } }
impl<T: ThriftTyped> ThriftTyped for HashSet<T> { fn typ() -> Type { Type::Set } }
impl<K: ThriftTyped, V: ThriftTyped> ThriftTyped for HashMap<K, V> { fn typ() -> Type { Type::Map } }

impl<X: Encode> Encode for Vec<X> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_list_begin(transport, X::typ(), self.len()));

        for el in self {
            try!(el.encode(protocol, transport));
        }

        try!(protocol.write_list_end(transport));

        Ok(())
    }
}

impl<X: Encode + Hash + Eq> Encode for HashSet<X> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_set_begin(transport, X::typ(), self.len()));

        for el in self {
            try!(el.encode(protocol, transport));
        }

        try!(protocol.write_set_end(transport));

        Ok(())
    }
}

impl<K: Encode + Hash + Eq, V: Encode> Encode for HashMap<K, V> {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_map_begin(transport, K::typ(), V::typ(), self.len()));

        for (k, v) in self.iter() {
            try!(k.encode(protocol, transport));
            try!(v.encode(protocol, transport));
        }

        try!(protocol.write_map_end(transport));

        Ok(())
    }
}

impl Encode for String {
    fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        try!(protocol.write_string(transport, self));
        Ok(())
    }
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

impl<X: Decode> Decode for Vec<X> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let (typ, len) = try!(protocol.read_list_begin(transport));

        if typ == X::typ() {
            self.reserve(len as usize);
            for _ in 0..len { self.push(try!(decode(protocol, transport))); }
            try!(protocol.read_list_end(transport));
            Ok(())
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation))
        }
    }
}

impl<X: Decode + Eq + Hash> Decode for HashSet<X> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let (typ, len) = try!(protocol.read_set_begin(transport));

        if typ == X::typ() {
            self.reserve(len as usize);
            for _ in 0..len { self.insert(try!(decode(protocol, transport))); }
            try!(protocol.read_set_end(transport));
            Ok(())
        } else {
            Err(Error::from(protocol::Error::ProtocolViolation))
        }
    }
}

impl<K: Decode + Eq + Hash, V: Decode> Decode for HashMap<K, V> {
    fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> Result<()>
    where P: Protocol, T: Transport {
        let (ktyp, vtyp, len) = try!(protocol.read_map_begin(transport));

        if ktyp == K::typ() && vtyp == V::typ() {
            self.reserve(len as usize);
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
    String => read_string
}

