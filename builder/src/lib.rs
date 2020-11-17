#![feature(format_args_capture)]

use std::fmt::Write;

pub struct Generator {
    gen_addr_traits: bool,
}

pub fn generator(gen_addr_traits: bool) -> Box<dyn prost_build::ServiceGenerator> {
    Box::new(Generator {
        gen_addr_traits
    })
}

impl prost_build::ServiceGenerator for Generator {
    fn generate(&mut self, service: Service, buf: &mut String) {
        write!(buf, r#"use quix::derive::*;"#).unwrap();
        //let svcname = &service.name;

        for m in service.methods {
            let methodname = &m.name;
            let name = &m.proto_name;
            let rettype = &m.output_type;
            let input = &m.input_type;

            // TODO: Fix this crap
            let (out_read, out_write) = if m.output_proto_type == ".google.protobuf.Empty" {
                ("()".to_string(), "()".to_string())
            } else {
                (format!("<{}>::decode(b)", m.output_type),
                 format!("res.encode(b)?;")
                )
            };


            let callspec = format!("{}.{}.{}", service.package, service.name, m.name);
            let callid = crc::crc32::checksum_ieee(callspec.as_bytes());

            if self.gen_addr_traits {
                write!(buf, r#"
use quix::derive::*;
pub struct {name}(pub {input});

pub trait {name}Addr {{
    fn {methodname}(&self, arg: {input}) -> BoxFuture<'static, {rettype}>;
}}

impl<A> {name}Addr for Pid<A> where A: Handler<{name}> + DynHandler {{
    fn {methodname}(&self, arg: {input}) -> BoxFuture<'static, {rettype}> {{
        Box::pin(self.send({name}(arg)).map(|r| r.and_then(|r|r) ))
    }}
}}
impl {name}Addr for PidRecipient<{name}> {{
    fn {methodname}(&self, arg: {input}) -> BoxFuture<'static, {rettype}> {{
        Box::pin(self.send({name}(arg)).map(|r| r.and_then(|r|r) ))
    }}
}}
impl {name}Addr for NodeId {{
    fn {methodname}(&self, arg: {input}) ->BoxFuture<'static, {rettype}> {{
        Box::pin(self.send({name}(arg)))
    }}
}}
"#).unwrap();
            }
            write!(buf, r#"
impl actix::Message for {name} {{
    type Result = {rettype};
}}

impl quix::derive::RpcMethod for {name} {{
    const NAME: &'static str = "{callspec}";
    const ID: u32 = {callid};


    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {{
        prost::Message::encode(&self.0, b).map_err(|_| DispatchError::MessageFormat)
    }}
    fn read(b: impl bytes::Buf) -> Result<Self, DispatchError> {{
        Ok(Self(prost::Message::decode(b).map_err(|_| DispatchError::MessageFormat)?))
    }}

    fn read_result(b: impl bytes::Buf) -> Self::Result {{
        {out_read}
    }}

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {{
        {out_write};
        Ok(())
    }}
}}

impl From<{input}> for {name} {{
    fn from(a: {input}) -> Self {{
        Self(a)
    }}
}}

impl Into<{input}> for {name} {{
    fn into(self) -> {input} {{
        self.0
    }}
}}

impl ::core::ops::Deref for {name} {{
    type Target = {input};
    fn deref(&self) -> &Self::Target {{
        &self.0
    }}
}}
impl ::core::ops::DerefMut for {name} {{
    fn deref_mut(&mut self) -> &mut Self::Target {{
        &mut self.0
    }}
}}
            "#).unwrap();
        }
    }
}

pub use prost_build::*;

