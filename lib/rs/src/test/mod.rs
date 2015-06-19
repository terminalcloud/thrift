use protocol::{Type, Encode, Decode};
use mock::*;

mod prim;
mod strukt;
mod enom;
mod generated;

pub fn encode<T: Encode>(x: &T) -> MockProtocol {
    let mut protocol = MockProtocol::new();
    let mut transport = MockTransport::new(vec![]);
    x.encode(&mut protocol, &mut transport).unwrap();
    protocol
}

pub fn decode<T: Decode>(protocol: &mut MockProtocol) -> T {
    let mut instance = T::default();
    instance.decode(protocol, &mut MockTransport::new(vec![])).unwrap();
    instance
}

pub fn field_end() -> ProtocolAction {
    Field(Begin((String::new(), Type::Stop, 0)))
}

