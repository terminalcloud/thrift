use std::error::Error;

use virt::{VirtualEncodeObject};
use protocol::{MessageType, Encode, ProtocolFactory};
use transport::server::{TransportServer};

type VirtualProxy = for<'e> Proxy<VirtualEncodeObject<'e>>;

#[derive(Default)]
pub struct Proxies {
    proxies: Vec<Box<VirtualProxy>>
}

impl Proxies {
    pub fn new() -> Proxies { Proxies::default() }
}

impl<E: Encode> Proxy<E> for Proxies {
    fn proxy(&self, mtype: MessageType, operation: &str, id: i32, message: E) {
        for proxy in &self.proxies {
            let message: VirtualEncodeObject = &message;
            proxy.proxy(mtype, operation, id, message);
        }
    }
}

pub trait Proxy<E: Encode> {
    fn proxy(&self, mtype: MessageType, operation: &str, id: i32, message: E);
}

pub struct SimpleProxy<PF, TS> {
    protocol_factory: PF,
    transport_server: TS
}

impl<E, PF, TS> Proxy<E> for SimpleProxy<PF, TS>
where PF: ProtocolFactory, TS: TransportServer, E: Encode {
    fn proxy(&self, mtype: MessageType, operation: &str, id: i32, message: E) {
        let _: Result<(), Box<Error>> = (|| {
            let mut protocol = self.protocol_factory.new_protocol();
            let mut transport = try!(self.transport_server.accept());

            Ok(try!(::protocol::helpers::send(&mut protocol, &mut transport, operation, mtype, &message, id)))
        })();
    }
}

