#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Key {
    #[prost(bytes, required, tag="1")]
    pub data: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(bytes, optional, tag="1")]
    pub data: ::std::option::Option<std::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entry {
    #[prost(message, required, tag="1")]
    pub key: Key,
    #[prost(message, required, tag="2")]
    pub value: Value,
}

use quix::derive::*;
pub struct Get(pub Key);

impl actix::Message for Get {
    type Result = Result<Value, ()>;
}

impl quix::derive::Service for Get {
    const NAME: &'static str = "quix.memkv.MemKv.get";
    const ID: u64 = 1538967013152268372;
    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        prost::Message::encode(&self.0, b).map_err(|_| ())
    }
    fn read(b: impl bytes::Buf) -> Result<Self, ()> {
        Ok(Self(prost::Message::decode(b).map_err(|_| ())?))
    }

    fn read_result(b: impl bytes::Buf) -> Result<Self::Result, ()> {
        Ok(Ok(<Value>::decode(b).unwrap()))
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        let a: &Value = res.as_ref().unwrap(); a.encode(b).unwrap();
        Ok(())
    }
}

impl From<Key> for Get {
    fn from(a: Key) -> Self {
        Self(a)
    }
}

impl Into<Key> for Get {
    fn into(self) -> Key {
        self.0
    }
}

impl ::core::ops::Deref for Get {
    type Target = Key;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for Get {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
            