#![recursion_limit="1024"]
extern crate podio;
extern crate ordered_float;
extern crate byteorder;

#[macro_use]
extern crate log;

use std::{io, fmt};
use std::error::Error as StdError;

pub use protocol::Protocol;
pub use transport::Transport;
pub use processor::Processor;

pub mod rt {
    pub use ordered_float::OrderedFloat;
}

pub mod protocol;
pub mod transport;
pub mod server;
pub mod processor;
pub mod proxy;
pub mod virt;

#[macro_use]
mod codegen;
mod impls;
mod compiletest;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum Error {
    /// An error occurred when reading from/writing to the underlying transport
    TransportError(io::Error),

    /// An error occurred when encoding/decoding the data
    /// (this usually indicates a bug in the library)
    ProtocolError(protocol::Error),

    /// The server code threw a user-defined exception
    UserException,
}

impl From<protocol::Error> for Error {
    fn from(err: protocol::Error) -> Error {
        Error::ProtocolError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::TransportError(err)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "Thrift Error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::TransportError(ref err) => Some(err),
            Error::ProtocolError(ref err) => Some(err),
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
