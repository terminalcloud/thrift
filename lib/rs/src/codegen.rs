#[macro_export]
macro_rules! service {
    (trait_name = $name:ident,
     processor_name = $processor_name:ident,
     client_name = $client_name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => $senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)*] ($srrty:ty),)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => $penname:ident = [$($pevname:ident($pename:ident: $pety:ty => $peid:expr),)*] ($prrty:ty),)*],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub trait $name {
            $(fn $smname(&self, $($saname: $saty),*) -> $srrty;)*
        }

        service_processor! {
            processor_name = $processor_name,
            service_methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*],
            parent_methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*],
            bounds = [$($boundty: $bound,)*],
            fields = [$($fname: $fty,)*]
        }

        service_client! {
            client_name = $client_name,
            service_methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*],
            parent_methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*]
        }
    }
}

#[macro_export]
macro_rules! service_processor {
    (processor_name = $name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => $senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)*] ($srrty:ty),)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => $penname:ident = [$($pevname:ident($pename:ident: $pety:ty => $peid:expr),)*] ($prrty:ty),)*],
     bounds = [$($boundty:ident: $bound:ident,)*],
     fields = [$($fname:ident: $fty:ty,)*]) => {
        pub struct $name<$($boundty: $bound),*> {
            $($fname: $fty,)*
            proxies: $crate::proxy::Proxies
        }

        $(strukt! { name = $siname, fields = { $($saname: Option<$saty> => $said,)* } }
          strukt! { name = $soname, fields = { success: Option<$srty> => 0, $($sename: Option<$sety> => $seid,)* } }
          service_processor_error_enum! { $senname = [ $($sevname($sename: $sety => $seid),)*] })*

        impl<$($boundty: $bound),*> $name<$($boundty),*> {
            pub fn new($($fname: $fty),*) -> Self {
                $name { $($fname: $fname,)* proxies: Default::default() }
            }

            /// Add a `Proxy` to be used for all incoming messages.
            pub fn proxy<P>(&mut self, proxy: P)
            where P: 'static + Send + Sync + for<'e> $crate::proxy::Proxy<$crate::virt::VirtualEncodeObject<'e>> {
                self.proxies.proxy(proxy)
            }

            pub fn dispatch<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                                       name: &str, ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
                match name {
                    $(stringify!($smname) => self.$smname(prot, transport, ty, id),)*
                    $(stringify!($pmname) => self.$pmname(prot, transport, ty, id),)*
                    _ => Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }
            }

            service_processor_methods! { methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*] }
            service_processor_methods! { methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*] }
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
macro_rules! service_processor_error_enum {
    ($senname:ident = []) => {};
    ($senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)+]) => {
        #[derive(Debug, Clone)]
        pub enum $senname {
            $(
                $sevname($sety),
            )+
        }

        $(
            impl From<$sety> for $senname {
                fn from(v: $sety) -> $senname {
                    $senname::$sevname(v)
                }
            }
        )+
    }
}

#[macro_export]
macro_rules! service_processor_methods {
    (methods = [$($iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => $enname:ident = [$($evname:ident($ename:ident: $ety:ty => $eid:expr),)*] ($rrty:ty),)*]) => {
        $(fn $mname<P: $crate::Protocol, T: $crate::Transport>(&self, prot: &mut P, transport: &mut T,
                                                               ty: $crate::protocol::MessageType, id: i32) -> $crate::Result<()> {
            use $crate::proxy::Proxy;

            static MNAME: &'static str = stringify!($mname);

            let mut args = $iname::default();
            try!($crate::protocol::helpers::receive_body(prot, transport, MNAME,
                                                         &mut args, MNAME, ty, id));

            self.proxies.proxy(ty, MNAME, id, &args);

            // TODO: Further investigate this unwrap.
            let result = self.$fname.$mname($(args.$aname.unwrap()),*);
            let result = service_processor_methods_translate_return!(
                result, $oname, $enname = [$($evname($ename: $ety => $eid),)*]);
            try!($crate::protocol::helpers::send(prot, transport, MNAME,
                                                 $crate::protocol::MessageType::Reply, &result, id));

            Ok(())
        })*
    }
}

#[macro_export]
macro_rules! service_processor_methods_translate_return {
    ($result:expr, $oname:ident, $enname:ident = []) => {{
        let mut result = $oname::default();
        result.success = Some($result);
        result
    }};
    ($result:expr, $oname:ident, $enname:ident = [$($evname:ident($ename:ident: $ty:ty => $eid:expr),)+]) => {{
        let mut result = $oname::default();
        match $result {
            Ok(r) => result.success = Some(r),
            $(
                Err($enname::$evname(e)) => result.$ename = Some(e),
            )+
        }
        result
    }}
}

#[macro_export]
macro_rules! service_client {
    (client_name = $client_name:ident,
     service_methods = [$($siname:ident -> $soname:ident = $smfname:ident.$smname:ident($($saname:ident: $saty:ty => $said:expr,)*) -> $srty:ty => $senname:ident = [$($sevname:ident($sename:ident: $sety:ty => $seid:expr),)*] ($srrty:ty),)*],
     parent_methods = [$($piname:ident -> $poname:ident = $pmfname:ident.$pmname:ident($($paname:ident: $paty:ty => $paid:expr,)*) -> $prty:ty => $penname:ident = [$($pevname:ident($pename:ident: $pety:ty => $peid:expr),)*] ($prrty:ty),)*]) => {
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

            service_client_methods! { methods = [$($siname -> $soname = $smfname.$smname($($saname: $saty => $said,)*) -> $srty => $senname = [$($sevname($sename: $sety => $seid),)*] ($srrty),)*] }
            service_client_methods! { methods = [$($piname -> $poname = $pmfname.$pmname($($paname: $paty => $paid,)*) -> $prty => $penname = [$($pevname($pename: $pety => $peid),)*] ($prrty),)*] }
        }
    }
}

#[macro_export]
macro_rules! service_client_methods {
    (methods = [$($iname:ident -> $oname:ident = $fname:ident.$mname:ident($($aname:ident: $aty:ty => $aid:expr,)*) -> $rty:ty => $enname:ident = [$($evname:ident($ename:ident: $ety:ty => $eid:expr),)*] ($rrty:ty),)*]) => {
        $(pub fn $mname(&mut self, $($aname: $aty,)*) -> $crate::Result<$rrty> {
            #[allow(unused_imports)]
            use $crate::protocol::Decode;
            static MNAME: &'static str = stringify!($mname);

            let mut args = $iname::default();
            $(args.$aname = Some($aname);)*
            try!($crate::protocol::helpers::send(&mut self.protocol, &mut self.transport,
                                                 MNAME, $crate::protocol::MessageType::Call, &mut args, 0));

            let mut result = $oname::default();
            let (name, ty, id) = try!(self.protocol.read_message_begin(&mut self.transport));
            if ty == $crate::protocol::MessageType::Exception {
                // receive exception in ename, but use id to identify which one
                // println!("name:{}, ty:{}, id:{}", name, ty, id);
                match id + 1 {
                    $( $eid => { let mut arg: $ety = Default::default();
                                 try!(arg.decode(&mut self.protocol, &mut self.transport));
                                 try!(self.protocol.read_message_end(&mut self.transport));
                                 result.$ename = Some(arg); } ),*
                    _ => return Err($crate::Error::from($crate::protocol::Error::ProtocolViolation))
                }
            } else {
                try!($crate::protocol::helpers::receive_body(&mut self.protocol, &mut self.transport,
                                                             MNAME, &mut result, &name, ty, id));
            }

            let result = service_client_methods_translate_result!(
                result, $enname = [$($evname($ename: $ety => $eid),)*]);
            Ok(result)
        })*
    }
}

#[macro_export]
macro_rules! service_client_methods_translate_result {
    ($result:expr, $enname:ident = []) => {{
        use $crate::protocol::Encode;

        let result = $result;

        if result.success.should_encode() {
            result.success.unwrap()
        } else {
            result.success.unwrap_or_default()
        }
    }};
    ($result:expr, $enname:ident = [$($evname:ident($ename:ident: $ety:ty => $eid:expr),)*]) => {{
        let result = $result;
        if let Some(s) = result.success {
            Ok(s)
        }
        $(
            else if let Some(e) = result.$ename {
                Err($enname::$evname(e))
            }
        )*
        else {
            // TODO investigate this
            unreachable!()
        }
    }}
}

#[macro_export]
macro_rules! strukt {
    (name = $name:ident,
     fields = { $($fname:ident: $fty:ty => $id:expr,)+ }) => {
        #[derive(Debug, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
        pub struct $name {
            $(pub $fname: $fty,)+
        }

        impl $crate::protocol::ThriftTyped for $name {
            fn typ(&self) -> $crate::protocol::Type { $crate::protocol::Type::Struct }
        }

        impl $crate::protocol::Encode for $name {
            fn encode<P, T>(&self, protocol: &mut P, transport: &mut T) -> $crate::Result<()>
            where P: $crate::Protocol, T: $crate::Transport {
                #[allow(unused_imports)]
                use $crate::protocol::{Encode, ThriftTyped};
                #[allow(unused_imports)]
                use $crate::{Protocol};

                try!(protocol.write_struct_begin(transport, stringify!($name)));

                $(if $crate::protocol::Encode::should_encode(&self.$fname) {
                    try!(protocol.write_field_begin(transport, stringify!($fname),
                                                    $crate::protocol::helpers::typ::<$fty>(), $id));
                    try!($crate::protocol::Encode::encode(&self.$fname, protocol, transport));
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
                    } $(else if (typ, id) == ($crate::protocol::helpers::typ::<$fty>(), $id) {
                        try!($crate::protocol::Decode::decode(&mut self.$fname, protocol, transport));
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
            fn typ(&self) -> $crate::protocol::Type { $crate::protocol::Type::Struct }
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
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
            fn typ(&self) -> $crate::protocol::Type { $crate::protocol::Type::I32 }
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

