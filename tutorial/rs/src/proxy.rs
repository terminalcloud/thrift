#[macro_use]
extern crate terminal_thrift as thrift;
extern crate bufstream;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

use thrift::protocol::binary_protocol::BinaryProtocol;
use thrift::transport::server::TransportServer;
use thrift::processor::Processor;
use thrift::protocol::ProtocolFactory;
use thrift::proxy::SimpleProxy;
use thrift::transport::RwTransport;

use bufferserver::BufferServer;
use bufstream::BufStream;

use shared::*;

mod bufferserver;
mod shared;

#[derive(Clone)]
struct Handler {
    sender: mpsc::Sender<i32>
}

impl SharedService for Handler {
    fn getStruct(&self, id: i32) -> SharedStruct {
        self.sender.send(id).unwrap();

        SharedStruct { key: id, value: String::new() }
    }
}

struct LimitedServer<P, PF, TS> {
    limit: i32,
    processor: P,
    protocols: PF,
    transports: TS
}

impl<P, PF, TS> LimitedServer<P, PF, TS>
where P: Processor<PF::Protocol, TS::Transport>,
      PF: ProtocolFactory, TS: TransportServer {
    fn serve(&mut self) {
        'serve: loop {
            let mut transport = self.transports.accept().unwrap();
            let mut protocol = self.protocols.new_protocol();

            while let Ok(_) = self.processor.process(&mut protocol, &mut transport) {
                self.limit -= 1;
                if self.limit == 0 { break 'serve; }
            }
        }
    }
}

fn main() {
    let requests = 5;
    let proxy_addr = "127.0.0.1:8001";
    let receiver_addr = "127.0.0.1:8002";

    let (source_tx, source_rx) = mpsc::channel();
    let (receiver_tx, receiver_rx) = mpsc::channel();

    let server_guard = thread::spawn(move || {
        let source = Handler { sender: source_tx };
        let receiver = Handler { sender: receiver_tx };

        let mut source_processor = SharedServiceProcessor::new(source);
        source_processor.proxy(SimpleProxy::new(|| BinaryProtocol,
                                                move || Ok(RwTransport(try!(TcpStream::connect(receiver_addr))))));

        let mut proxy_server = LimitedServer {
            limit: requests,
            processor: source_processor,
            protocols: || BinaryProtocol,
            transports: BufferServer(TcpListener::bind(proxy_addr).unwrap())
        };

        let mut receiver_server = LimitedServer {
            limit: requests,
            processor: SharedServiceProcessor::new(receiver),
            protocols: || BinaryProtocol,
            transports: BufferServer(TcpListener::bind(receiver_addr).unwrap())
        };

        let receiver_server_guard = thread::spawn(move || { receiver_server.serve(); receiver_server });
        let proxy_server_guard = thread::spawn(move || { proxy_server.serve(); proxy_server });

        receiver_server_guard.join().unwrap();
        proxy_server_guard.join().unwrap();
    });

    // Very slightly racy so just sleep a bit to let the server start before sending requests.
    thread::sleep_ms(100);

    let client_guard = thread::spawn(move || {
        let stream = RwTransport(BufStream::new(TcpStream::connect(proxy_addr).unwrap()));
        let mut client = SharedServiceClient::new(BinaryProtocol, stream);

        for i in 0..requests {
            client.getStruct(i).unwrap();
        }
    });

    client_guard.join().unwrap();
    server_guard.join().unwrap();

    let source_record = source_rx.iter().collect::<Vec<_>>();
    let receiver_record = receiver_rx.iter().collect::<Vec<_>>();

    assert_eq!(source_record.len() as i32, requests);
    assert_eq!(receiver_record.len() as i32, requests);
    assert_eq!(source_record, receiver_record);

    println!("Passed!");
}

