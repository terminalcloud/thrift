#[macro_export]
macro_rules! service {
    (trait_name = $name:ident,
     processor_name = $processor_name:ident,
     client_name = $client_name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => [$($sename:ident: $sety:ty => $seid:expr,)*],)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => [$($pename:ident: $pety:ty => $peid:expr,)*],)*],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub trait $name {
            $(fn $smname(&self, $($saname: $saty),*) -> $soname;)*
        }

        service_processor! {
            processor_name = $processor_name,
            service_methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => [$($sename: $sety => $seid,)*],)*],
            parent_methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => [$($pename: $pety => $peid,)*],)*],
            bounds = [$($boundty: $bound,)*],
            fields = [$($fname: $fty,)*]
        }

        service_client! {
            client_name = $client_name,
            service_methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => [$($sename: $sety => $seid,)*],)*],
            parent_methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => [$($pename: $pety => $peid,)*],)*]
        }
    }
}

#[macro_export]
macro_rules! service_processor {
    (processor_name = $name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => [$($sename:ident: $sety:ty => $seid:expr,)*],)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => [$($pename:ident: $pety:ty => $peid:expr,)*],)*],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub struct $name<$($boundty: $bound),*> {
            $($fname: $fty,)*
            _ugh: ()
        }

        $(strukt! { name = $siname, fields = { $($saname: $saty => $said,)* } }
          strukt! { name = $soname, fields = { success: $srty => 0, $($sename: $sety => $seid,)* } })*

        impl<$($boundty: $bound),*> $name<$($boundty),*> {
            pub fn new($($fname: $fty),*) -> Self {
                $name { $($fname: $fname,)* _ugh: () }
            }

            pub fn dispatch<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                                       name: &str, ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
                match name {
                    $(stringify!($smname) => self.$smname(prot, transport, ty, id),)*
                    $(stringify!($pmname) => self.$pmname(prot, transport, ty, id),)*
                    _ => Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }
            }

            service_processor_methods! { methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => [$($sename: $sety => $seid,)*],)*] }
            service_processor_methods! { methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => [$($pename: $pety => $peid,)*],)*] }
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

#[macro_export]
macro_rules! service_processor_methods {
    (methods = [$($iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => [$($ename:ident: $ety:ty => $eid:expr,)*],)*]) => {
        $(fn $mname<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                               ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
            static MNAME: &'static str = stringify!($mname);

            let mut args = $iname::default();
            try!($crate::protocol::helpers::receive_body(prot, transport, MNAME,
                                                         &mut args, MNAME, ty, id));

            // TODO: Further investigate this unwrap.
            let result = self.$fname.$mname($(args.$aname.unwrap()),*);
            try!($crate::protocol::helpers::send(prot, transport, MNAME,
                                                 $crate::protocol::MessageType::Reply, &result));

            Ok(())
        })*
    }
}

#[macro_export]
macro_rules! service_client {
    (client_name = $client_name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => [$($sename:ident: $sety:ty => $seid:expr,)*],)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => [$($pename:ident: $pety:ty => $peid:expr,)*],)*]) => {
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

            service_client_methods! { methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => [$($sename: $sety => $seid,)*],)*] }
            service_client_methods! { methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => [$($pename: $pety => $peid,)*],)*] }
        }
    }
}

#[macro_export]
macro_rules! service_client_methods {
    (methods = [$($iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => [$($ename:ident: $ety:ty => $eid:expr,)*],)*]) => {
        $(pub fn $mname(&mut self, $($aname: $aty,)*) -> $crate::Result<$oname> {
            static MNAME: &'static str = stringify!($mname);

            let mut args = $iname::default();
            $(args.$aname = Some($aname);)*
            try!($crate::protocol::helpers::send(&mut self.protocol, &mut self.transport,
                                                 MNAME, $crate::protocol::MessageType::Call, &mut args));

            let mut result = $oname::default();
            try!($crate::protocol::helpers::receive(&mut self.protocol, &mut self.transport,
                                                    MNAME, &mut result));

            Ok(result)
        })*
    }
}

#[macro_export]
macro_rules! strukt {
    (name = $name:ident,
     fields = { $($fname:ident: $fty:ty => $id:expr,)+ }) => {
        #[derive(Debug, Clone, Default)]
        pub struct $name {
            $(pub $fname: Option<$fty>,)+
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ() -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Encode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::{Protocol};

                try!(protocol.write_struct_begin(transport, stringify!($name)));

                $(if let Some(ref x) = self.$fname {
                    try!(protocol.write_field_begin(transport, stringify!($fname), <$fty as ThriftTyped>::typ(), $id));
                    try!(x.encode(protocol, transport));
                    try!(protocol.write_field_end(transport));
                })*

                try!(protocol.write_field_stop(transport));
                try!(protocol.write_struct_end(transport));

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

                try!(protocol.read_struct_begin(transport));

                loop {
                    let (_, typ, id) = try!(protocol.read_field_begin(transport));

                    if typ == $crate::protocol::Type::Stop {
                        break;
                    } $(else if (typ, id) == (<$fty as ThriftTyped>::typ(), $id) {
                        try!(self.$fname.decode(protocol, transport));
                    })* else {
                        try!(protocol.skip(transport, typ));
                    }

                    try!(protocol.read_field_end(transport));
                }

                try!(protocol.read_struct_end(transport));

                Ok(())
            }
        }
    };
    (name = $name:ident, fields = {}) => {
        #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name;

        impl $crate::protocol::ThriftTyped for $name {
            fn typ() -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::Protocol;

                try!(protocol.write_struct_begin(transport, stringify!($name)));
                try!(protocol.write_field_stop(transport));
                try!(protocol.write_struct_end(transport));

                Ok(())
            }
        }

        impl $crate::protocol::Decode for $name {
            fn decode<P, T>(&mut self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::Protocol;

                try!(protocol.read_struct_begin(transport));

                let (_, ty, _) = try!(protocol.read_field_begin(transport));
                if ty != $crate::protocol::Type::Stop {
                     return Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }

                try!(protocol.read_struct_end(transport));

                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! enom {
    (name = $name:ident,
     values = [$($vname:ident = $val:expr,)*],
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

