use protocol::Encode;
use mock::{MockProtocol, MockTransport};

mod prim;
mod strukt;
mod enom;
mod generated;

pub fn encode<T: Encode>(x: T) -> MockProtocol {
    let mut protocol = MockProtocol::new();
    let mut transport = MockTransport::new(vec![]);
    x.encode(&mut protocol, &mut transport).unwrap();
    protocol
}

