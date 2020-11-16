#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Net {
    #[prost(message, optional, tag="1")]
    pub meta: ::std::option::Option<Meta>,
    #[prost(message, optional, tag="2")]
    pub ping: ::std::option::Option<PingPong>,
    #[prost(message, optional, tag="3")]
    pub pong: ::std::option::Option<PingPong>,
    #[prost(message, optional, tag="4")]
    pub request: ::std::option::Option<Request>,
    #[prost(message, optional, tag="5")]
    pub response: ::std::option::Option<Response>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingPong {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Meta {
    #[prost(bytes, required, tag="1")]
    pub nodeid: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(bytes, optional, tag="1")]
    pub procid: ::std::option::Option<std::vec::Vec<u8>>,
    #[prost(int64, optional, tag="2")]
    pub correlation: ::std::option::Option<i64>,
    #[prost(fixed32, required, tag="3")]
    pub methodid: u32,
    #[prost(bytes, required, tag="4")]
    pub body: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(int64, required, tag="1")]
    pub correlation: i64,
    /// TODO: make required
    #[prost(bytes, optional, tag="2")]
    pub body: ::std::option::Option<std::vec::Vec<u8>>,
    #[prost(enumeration="InvokeError", optional, tag="3")]
    pub error: ::std::option::Option<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct M1 {
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum InvokeError {
    /// Process for invocation was not found
    ProcessNotFound = 1,
    /// Invocation target could not handle this method
    MethodNotFound = 2,
    /// Node specified in the message could not be found
    NodeNotFound = 3,
    /// Invocation args could not be deserialized
    MessageFormat = 4,
    /// Not sent but used internally, message handling timed out.
    Timeout = 5,
}
use quix::derive::*;
use quix::derive::*;
pub struct Method(pub M1);

pub trait MethodAddr {
    fn method(&self, arg: M1) -> BoxFuture<'static, Result<M1, DispatchError>>;
}

impl<A> MethodAddr for Pid<A> where A: Handler<Method> + DynHandler {
    fn method(&self, arg: M1) -> BoxFuture<'static, Result<M1, DispatchError>> {
        Box::pin(self.send(Method(arg)).map(|r| r.and_then(|r|r) ))
    }
}
impl MethodAddr for PidRecipient<Method> {
    fn method(&self, arg: M1) -> BoxFuture<'static, Result<M1, DispatchError>> {
        Box::pin(self.send(Method(arg)).map(|r| r.and_then(|r|r) ))
    }
}
impl MethodAddr for NodeId {
    fn method(&self, arg: M1) ->BoxFuture<'static, Result<M1, DispatchError>> {
        Box::pin(self.send(Method(arg)))
    }
}

impl actix::Message for Method {
    type Result = Result<M1, DispatchError>;
}

impl quix::derive::RpcMethod for Method {

    const NAME: &'static str = "quix.net.Exec.method";
    const ID: u32 = 529753007;

    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
        prost::Message::encode(&self.0, b).map_err(|_| DispatchError::MessageFormat)
    }
    fn read(b: impl bytes::Buf) -> Result<Self, DispatchError> {
        Ok(Self(prost::Message::decode(b).map_err(|_| DispatchError::MessageFormat)?))
    }

    fn read_result(b: impl bytes::Buf) -> Self::Result {
        Ok(<M1>::decode(b).unwrap())
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
        let a: &M1 = res.as_ref().unwrap(); a.encode(b).unwrap();
        Ok(())
    }
}

impl From<M1> for Method {
    fn from(a: M1) -> Self {
        Self(a)
    }
}

impl Into<M1> for Method {
    fn into(self) -> M1 {
        self.0
    }
}

impl ::core::ops::Deref for Method {
    type Target = M1;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for Method {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
            