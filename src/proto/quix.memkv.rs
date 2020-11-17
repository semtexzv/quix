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
use quix::derive::*;
pub struct Get(pub Key);

pub trait GetAddr {
    fn get(&self, arg: Key) -> BoxFuture<'static, Value>;
}

impl<A> GetAddr for Pid<A> where A: Handler<Get> + DynHandler {
    fn get(&self, arg: Key) -> BoxFuture<'static, Value> {
        Box::pin(self.send(Get(arg)).map(|r| r.and_then(|r|r) ))
    }
}
impl GetAddr for PidRecipient<Get> {
    fn get(&self, arg: Key) -> BoxFuture<'static, Value> {
        Box::pin(self.send(Get(arg)).map(|r| r.and_then(|r|r) ))
    }
}
impl GetAddr for NodeId {
    fn get(&self, arg: Key) ->BoxFuture<'static, Value> {
        Box::pin(self.send(Get(arg)))
    }
}

impl actix::Message for Get {
    type Result = Value;
}

impl quix::derive::RpcMethod for Get {
    const NAME: &'static str = "quix.memkv.MemKv.get";
    const ID: u32 = 2761597858;


    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
        prost::Message::encode(&self.0, b).map_err(|_| DispatchError::MessageFormat)
    }
    fn read(b: impl bytes::Buf) -> Result<Self, DispatchError> {
        Ok(Self(prost::Message::decode(b).map_err(|_| DispatchError::MessageFormat)?))
    }

    fn read_result(b: impl bytes::Buf) -> Self::Result {
        <Value>::decode(b)
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
        res.encode(b)?;;
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
            