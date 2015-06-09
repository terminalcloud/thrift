use protocol::Protocol;
use transport::Transport;
use Result;

pub trait Processor<P: Protocol, T: Transport> {
    fn process(&mut self, prot: &mut P, transport: &mut T) -> Result<()>;
}

