/// Messages exchanged between individual nodes on lowest protocol level
#[derive(prost::Message)]
pub struct Net {
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
    #[prost(bytes, tag = "1")]
    pub id: Vec<u8>
}

#[derive(prost::Message)]
pub struct Request {
    #[prost(bytes, required, tag = "1")]
    pub procid: Vec<u8>,

    #[prost(int64, optional, tag = "2")]
    pub correlation: Option<i64>,

    #[prost(string, required, tag = "3")]
    pub method: String,

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
