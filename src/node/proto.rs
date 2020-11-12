/// Messages exchanged between individual nodes on lowest protocol level
#[derive(prost::Message)]
pub struct NodeProtoMessage {
    #[prost(message, optional, tag = "1")]
    pub ping: Option<PingPong>,

    #[prost(message, optional, tag = "2")]
    pub pong: Option<PingPong>,

    #[prost(message, optional, tag = "3")]
    pub meta: Option<Meta>,

    #[prost(message, optional, tag = "4")]
    pub request: Option<Request>,

    #[prost(message, optional, tag = "5")]
    pub response: Option<Response>,

}

#[derive(prost::Message)]
pub struct PingPong {
    #[prost(string, tag = "1")]
    pub value: String
}

#[derive(prost::Message)]
pub struct Meta {
    #[prost(string, tag = "1")]
    pub id: String
}

#[derive(prost::Message)]
pub struct Request {
    #[prost(string, required, tag = "1")]
    pub procid: String,

    #[prost(string, required, tag = "2")]
    pub method: String,

    #[prost(int64, optional, tag = "3")]
    pub correlation: Option<i64>,

    #[prost(bytes, required, tag = "4")]
    pub body: Vec<u8>,
}

#[derive(prost::Message)]
pub struct Response {
    #[prost(int64, required, tag = "1")]
    pub correlation: i64,

    #[prost(bytes, required, tag = "2")]
    pub body: Vec<u8>,
}
