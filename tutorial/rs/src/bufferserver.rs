use thrift::transport::server::TransportServer;
use thrift::transport::RwTransport;

use bufstream::BufStream;

use std::net::{TcpListener, TcpStream};
use std::io;

pub struct BufferServer(pub TcpListener);

impl TransportServer for BufferServer {
     type Transport = RwTransport<BufStream<TcpStream>>;

     fn accept(&self) -> io::Result<Self::Transport> {
        self.0.accept().map(|res| RwTransport(BufStream::new(res.0)))
     }
}

