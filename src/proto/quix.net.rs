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
    #[prost(fixed64, required, tag="3")]
    pub methodid: u64,
    #[prost(bytes, required, tag="4")]
    pub body: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(int64, required, tag="1")]
    pub correlation: i64,
    #[prost(bytes, required, tag="2")]
    pub body: std::vec::Vec<u8>,
}
