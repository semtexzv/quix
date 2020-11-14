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
pub struct KvPut(pub Entry);

impl actix::Message for KvPut {
    type Result = ();
}

impl quix::derive::Service for KvPut {
    const NAME: &'static str = "quix.kv.Kv.kv_put";
    const ID: u64 = 12135346219214501435;
    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        prost::Message::encode(&self.0, b).map_err(|_| ())
    }
    fn read(b: impl bytes::Buf) -> Result<Self, ()> {
        Ok(Self(prost::Message::decode(b).map_err(|_| ())?))
    }

    fn read_result(b: impl bytes::Buf) -> Result<Self::Result, ()> {
        Ok(())
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        ();
        Ok(())
    }
}

impl From<Entry> for KvPut {
    fn from(a: Entry) -> Self {
        Self(a)
    }
}

impl Into<Entry> for KvPut {
    fn into(self) -> Entry {
        self.0
    }
}

impl ::core::ops::Deref for KvPut {
    type Target = Entry;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for KvPut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
            
use quix::derive::*;
pub struct KvGet(pub Key);

impl actix::Message for KvGet {
    type Result = Result<Value, ()>;
}

impl quix::derive::Service for KvGet {
    const NAME: &'static str = "quix.kv.Kv.kv_get";
    const ID: u64 = 4438170575017344694;
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

impl From<Key> for KvGet {
    fn from(a: Key) -> Self {
        Self(a)
    }
}

impl Into<Key> for KvGet {
    fn into(self) -> Key {
        self.0
    }
}

impl ::core::ops::Deref for KvGet {
    type Target = Key;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for KvGet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
            
use quix::derive::*;
pub struct KvcDelete(pub Key);

impl actix::Message for KvcDelete {
    type Result = ();
}

impl quix::derive::Service for KvcDelete {
    const NAME: &'static str = "quix.kv.Kv.kvc_delete";
    const ID: u64 = 12641160675030770921;
    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        prost::Message::encode(&self.0, b).map_err(|_| ())
    }
    fn read(b: impl bytes::Buf) -> Result<Self, ()> {
        Ok(Self(prost::Message::decode(b).map_err(|_| ())?))
    }

    fn read_result(b: impl bytes::Buf) -> Result<Self::Result, ()> {
        Ok(())
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        ();
        Ok(())
    }
}

impl From<Key> for KvcDelete {
    fn from(a: Key) -> Self {
        Self(a)
    }
}

impl Into<Key> for KvcDelete {
    fn into(self) -> Key {
        self.0
    }
}

impl ::core::ops::Deref for KvcDelete {
    type Target = Key;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for KvcDelete {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
            