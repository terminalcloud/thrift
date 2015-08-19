use std::error::Error;

use virt::{VirtualEncodeObject};
use protocol::{MessageType, Encode, ProtocolFactory};
use transport::server::{TransportServer};

type VirtualProxy = for<'e> Proxy<VirtualEncodeObject<'e>>;

#[derive(Default)]
pub struct Proxies {
    proxies: Vec<Box<for<'e> Proxy<VirtualEncodeObject<'e>> + Send + Sync>>
}

impl Proxies {
    pub fn new() -> Proxies { Proxies::default() }

    pub fn proxy<P: for<'e> Proxy<VirtualEncodeObject<'e>> + Send + Sync + 'static>(&mut self, proxy: P) {
        self.proxies.push(Box::new(proxy));
    }
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

impl<PF, TS> SimpleProxy<PF, TS> {
    /// Create a new `SimpleProxy` that will replay messages over the given
    /// server transports using the given protocols.
    pub fn new(factory: PF, server: TS) -> SimpleProxy<PF, TS> {
        SimpleProxy {
            protocol_factory: factory,
            transport_server: server
        }
    }
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

