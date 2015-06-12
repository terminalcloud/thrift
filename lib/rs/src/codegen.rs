#[macro_export]
macro_rules! service {
    (trait_name = $name:ident,
     processor_name = $processor_name:ident,
     client_name = $client_name:ident,
     methods = [$($iname:ident -> $oname:ident = $mfname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr),*) -> $rty:ty),+],
     bounds = [$($bounds:tt)*],
     fields = [$($fname:ident: $fty:ty)+]) => {
        pub trait $name {
            $(fn $mname(&self, $($aname: $aty)*) -> $rty;)+
        }

        service_processor! {
            processor_name = $processor_name,
            methods = [$($iname -> $oname = $mfname.$mname($($aname: $aty => $aid),*) -> $rty),+],
            bounds = [$($bounds)*],
            fields = [$($fname: $fty),+]
        }

        service_client! {
            client_name = $client_name,
            methods = [$($iname -> $oname = $mfname.$mname($($aname: $aty => $aid),*) -> $rty),+]
        }
    }
}

macro_rules! service_processor {
    (processor_name = $name:ident,
     methods = [$($iname:ident -> $oname:ident = $mfname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr),*) -> $rty:ty),+],
     bounds = [<$($boundty:ident: $bound:ident),*>],
     fields = [$($fname:ident: $fty:ty),+]) => {
        pub struct $name<$($boundty: $bound),*> {
            $($fname: $fty),+
        }

        $(strukt! { name = $iname, fields = { $($aname: $aty => $aid),* } }
          strukt! { name = $oname, fields = { success: $rty => 0 } })+

        impl<$($boundty: $bound),*> $name<$($boundty),*> {
            pub fn new($($fname: $fty)+) -> Self {
                $name { $($fname: $fname,)+ }
            }

            pub fn dispatch<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                                       name: &str, ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
                match name {
                    $(stringify!($mname) => self.$mname(prot, transport, ty, id))+,
                    _ => Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }
            }

            $(fn $mname<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                                   ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
                static MNAME: &'static str = stringify!($mname);

                let mut args = $iname::default();
                try!($crate::protocol::helpers::receive_body(prot, transport, MNAME,
                                                             &mut args, MNAME, ty, id));

                let mut result = $oname::default();
                result.success = self.$fname.$mname($(args.$aname)*);

                try!($crate::protocol::helpers::send(prot, transport, MNAME,
                                                     $crate::protocol::MessageType::Reply, &result));

                Ok(())
            })+
        }

        impl<P: $crate::Protocol, T: $crate::Transport, $($boundty: $bound),*> $crate::Processor<P, T> for $name<$($boundty),*> {
            fn process(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()> {
                #[allow(unused_imports)]
                use $crate::Protocol;

                let (name, ty, id) = try!(protocol.read_message_begin(transport));
                self.dispatch(protocol, transport, &name, ty, id)
            }
        }
    }
}

macro_rules! service_client {
    (client_name = $client_name:ident,
     methods = [$($iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr),*) -> $rty:ty),+]) => {
        pub struct $client_name<P: $crate::Protocol, T: $crate::Transport> {
            pub protocol: P,
            pub transport: T
        }

        impl<P: $crate::Protocol, T: $crate::Transport> $client_name<P, T> {
            pub fn new(protocol: P, transport: T) -> Self {
                $client_name {
                    protocol: protocol,
                    transport: transport
                }
            }

            $(pub fn $mname(&mut self, $($aname: $aty,)*) -> $crate::Result<$rty> {
                static MNAME: &'static str = stringify!($mname);

                let args = $iname { $($aname: $aname,)* };
                try!($crate::protocol::helpers::send(&mut self.protocol, &mut self.transport,
                                                     MNAME, $crate::protocol::MessageType::Call, &args));

                let mut result = $oname::default();
                try!($crate::protocol::helpers::receive(&mut self.protocol, &mut self.transport,
                                                        MNAME, &mut result));

                Ok(result.success)
            })+
        }
    }
}

#[macro_export]
macro_rules! strukt {
    (name = $name:ident,
     fields = { $($fname:ident: $fty:ty => $id:expr),* }) => {
        #[derive(Debug, Clone, Default)]
        pub struct $name { $($fname: $fty),* }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ() -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Encode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::Protocol;

                $(try!(protocol.write_field_begin(transport, stringify!($fname), <$fty as ThriftTyped>::typ(), $id));
                  try!(self.$fname.encode(protocol, transport));
                  try!(protocol.write_field_end(transport));)*

                Ok(())
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Decode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::Protocol;

                let mut has_result = false;
                try!(protocol.read_struct_begin(transport));

                loop {
                    let (_, typ, id) = try!(protocol.read_field_begin(transport));

                    if typ == $crate::protocol::Type::Stop {
                        try!(protocol.read_field_end(transport));
                        break;
                    } $(else if (typ, id) == (<$fty as ThriftTyped>::typ(), $id) {
                        try!(self.$fname.decode(protocol, transport));
                        has_result = true;
                    })* else {
                        try!(protocol.skip(transport, typ));
                    }

                    try!(protocol.read_field_end(transport));
                }

                try!(protocol.read_struct_end(transport));

                if has_result {
                    Ok(())
                } else {
                    Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }
            }
        }
    };
}

#[macro_export]
macro_rules! enom {
    (name = $name:ident,
     values = [$($vname:ident = $val:expr),*],
     default = $dname:ident) => {
        #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(i32)]
        pub enum $name {
            $($vname = $val),*
        }

        impl Default for $name {
            fn default() -> Self { $name::$dname }
        }

        impl $crate::protocol::FromNum for $name {
            fn from_num(num: i32) -> Option<Self> {
                match num {
                    $($val => Some($name::$vname)),*,
                    _ => None
                }
            }
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ() -> $crate::protocol::Type { $crate::protocol::Type::I32 }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::Protocol;

                protocol.write_i32(transport, *self as i32)
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                *self = try!($crate::protocol::helpers::read_enum(protocol, transport));
                Ok(())
            }
        }
    }
}

