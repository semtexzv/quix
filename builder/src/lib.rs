#![feature(format_args_capture)]

use std::fmt::Write;

pub struct Generator;

impl prost_build::ServiceGenerator for Generator {
    fn generate(&mut self, service: Service, buf: &mut String) {
        for m in service.methods {
            eprintln!("{:?}", m);
            let output = if m.output_proto_type == ".google.protobuf.Empty" {
                "()".to_string()
            } else {
                format!("Result<{}, DispatchError>", m.output_type)
            };

            let intype = if m.input_proto_type == ".google.protobuf.Empty" {
                "()".to_string()
            } else {
                m.input_type
            };

            // TODO: Fix this crap
            let (out_read, out_write) = if m.output_proto_type == ".google.protobuf.Empty" {
                ("()".to_string(), "()".to_string())
            } else {
                (format!("Ok(<{}>::decode(b).unwrap())", m.output_type),
                 format!("let a: &{o} = res.as_ref().unwrap(); a.encode(b).unwrap()", o = m.output_type)
                )
            };


            let svc_spec = format!("{}.{}.{}", service.package, service.name, m.name);
            let svc_id = crc::crc32::checksum_ieee(svc_spec.as_bytes());

            write!(buf, r#"
use quix::derive::*;
pub struct {name}(pub {input});

impl actix::Message for {name} {{
    type Result = {output};
}}

impl quix::derive::RpcMethod for {name} {{

    const NAME: &'static str = "{svc_spec}";
    const ID: u32 = {svc_id};

    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {{
        prost::Message::encode(&self.0, b).map_err(|_| DispatchError::Format)
    }}
    fn read(b: impl bytes::Buf) -> Result<Self, DispatchError> {{
        Ok(Self(prost::Message::decode(b).map_err(|_| DispatchError::Format)?))
    }}

    fn read_result(b: impl bytes::Buf) -> Result<Self::Result, DispatchError> {{
        Ok({out_read})
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
            "#,
                   name = m.proto_name,
                   input = intype
            ).unwrap();
        }
    }
}

pub use prost_build::*;

