use protocol::Protocol;
use transport::Transport;
use Result;

pub trait Processor<P: Protocol, T: Transport> {
    fn process(&self, prot: &mut P, transport: &mut T) -> Result<()>;
}

