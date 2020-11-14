use std::fmt::Write;

pub struct Generator;

impl prost_build::ServiceGenerator for Generator {
    fn generate(&mut self, service: Service, buf: &mut String) {
        for m in service.methods {
            let outtype = if m.output_type == "Empty" {
                "()".to_string()
            } else {
                format!("Result<{}, ()>", m.output_type)
            };

            let intype = if m.input_type == "Empty" {
                "()".to_string()
            } else {
                m.input_type
            };


            write!(buf, r#"
pub struct {name}(pub {input});
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
impl crate::util::Service for {name} {{
    const NAME: &'static str = "{pkg}.{svc}.{name}";
     fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), ()> {{
        prost::Message::encode(&self.0, b).map_err(|_| ())
    }}
    fn read(b: impl bytes::Buf) -> Result<Self, ()> {{
        Ok(Self(prost::Message::decode(b).map_err(|_| ())?))
    }}
}}
impl actix::Message for {name} {{
    type Result = {res};
}}
            "#,
                   name = m.proto_name,
                   svc = service.name,
                   pkg = service.package,
                   res = outtype,
                   input = intype
            ).unwrap();
        }
    }
}

pub use prost_build::*;

