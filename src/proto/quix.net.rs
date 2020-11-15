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
    #[prost(bytes, required, tag="2")]
    pub body: std::vec::Vec<u8>,
    #[prost(enumeration="InvokeError", optional, tag="3")]
    pub error: ::std::option::Option<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum InvokeError {
    /// Process for invocation was not found
    ProcNotFound = 1,
    /// Invocation target could not handle this method
    HandlerNotFound = 2,
    /// Node specified in the message could not be found
    NodeNotFound = 3,
    /// Invocation args could not be deserialized
    MessageFormat = 4,
    /// Not sent but used internally, message handling timed out.
    Timeout = 5,
}
