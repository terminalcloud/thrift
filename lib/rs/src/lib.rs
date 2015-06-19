#![feature(optin_builtin_traits)]
extern crate podio;

#[macro_use]
extern crate log;

use std::{io, fmt};
use std::error::Error as StdError;
use std::collections::{HashSet, HashMap};

pub use protocol::Protocol;
pub use transport::Transport;
pub use processor::Processor;

pub mod protocol;
pub mod transport;
pub mod server;
pub mod processor;

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

// Some machinery for optional fields.
#[doc(hidden)]
pub trait Exists {
    #[inline(always)]
    fn exists(&self) -> bool { true }
}

impl<T> Exists for Option<T> {
    #[inline(always)]
    fn exists(&self) -> bool { self.is_some() }
}

impl Exists for bool { }
impl Exists for i8  { }
impl Exists for i16 { }
impl Exists for i32 { }
impl Exists for i64 { }
impl Exists for f64 { }
impl Exists for () { }
impl Exists for String { }
impl<T> Exists for Vec<T> { }
impl<T> Exists for HashSet<T> { }
impl<K, V> Exists for HashMap<K, V> { }

