// Automatically generated rust module for 'net.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use core::{convert::{TryFrom, TryInto}, ops::{Deref, DerefMut}};use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Net<'a> {
    pub meta: Option<net::Meta<'a>>,
    pub ping: Option<net::PingPong>,
    pub pong: Option<net::PingPong>,
    pub request: Option<net::Request<'a>>,
    pub response: Option<net::Response<'a>>,
}

impl<'a> MessageInfo for Net<'a> {
    const PATH : &'static str = "quix.net.Net";
}

impl<'a> MessageRead<'a> for Net<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.meta = Some(r.read_message::<net::Meta>(bytes)?),
                Ok(18) => msg.ping = Some(r.read_message::<net::PingPong>(bytes)?),
                Ok(26) => msg.pong = Some(r.read_message::<net::PingPong>(bytes)?),
                Ok(34) => msg.request = Some(r.read_message::<net::Request>(bytes)?),
                Ok(42) => msg.response = Some(r.read_message::<net::Response>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Net<'a> {
    fn get_size(&self) -> usize {
        0
        + self.meta.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.ping.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.pong.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.request.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.response.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.meta { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.ping { w.write_with_tag(18, |w| w.write_message(s))?; }
        if let Some(ref s) = self.pong { w.write_with_tag(26, |w| w.write_message(s))?; }
        if let Some(ref s) = self.request { w.write_with_tag(34, |w| w.write_message(s))?; }
        if let Some(ref s) = self.response { w.write_with_tag(42, |w| w.write_message(s))?; }
        Ok(())
    }
}


            #[derive(Debug)]
            struct NetOwnedInner {
                buf: Vec<u8>,
                proto: Net<'static>,
                _pin: core::marker::PhantomPinned,
            }

            impl NetOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = Net::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, Net<'static>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = proto;
                    }
                    Ok(pinned)
                }
            }

            pub struct NetOwned {
                inner: core::pin::Pin<Box<NetOwnedInner>>,
            }

            impl MessageInfo for NetOwned {
                const PATH: &'static str = "quix.net.Net";
            }

            #[allow(dead_code)]
            impl NetOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto(&self) -> &Net {
                    &self.inner.proto
                }
            }

            impl core::fmt::Debug for NetOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.fmt(f)
                }
            }

            impl Deref for NetOwned {
                type Target = Net<'static>;

                fn deref(&self) -> &Self::Target {
                    &self.inner.proto
                }
            }

            impl DerefMut for NetOwned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut self.inner.as_mut().get_unchecked_mut().proto }
                }
            }

            impl TryFrom<Vec<u8>> for NetOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: NetOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for NetOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.deref().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            #[cfg(feature = "test_helpers")]
            impl<'a> From<Net<'a>> for NetOwned {
                fn from(proto: Net) -> Self {
                    use quick_protobuf::{MessageWrite, Writer};

                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    proto.write_message(&mut writer).expect("bad proto serialization");
                    Self { inner: NetOwnedInner::new(buf).unwrap() }
                }
            }
            
#[derive(Debug, Default, PartialEq, Clone)]
pub struct PingPong { }

impl MessageInfo for PingPong {
    const PATH : &'static str = "quix.net.PingPong";
}

impl<'a> MessageRead<'a> for PingPong {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for PingPong { }

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Meta<'a> {
    pub nodeid: Cow<'a, [u8]>,
}

impl<'a> MessageInfo for Meta<'a> {
    const PATH : &'static str = "quix.net.Meta";
}

impl<'a> MessageRead<'a> for Meta<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.nodeid = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Meta<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.nodeid).len())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.nodeid))?;
        Ok(())
    }
}


            #[derive(Debug)]
            struct MetaOwnedInner {
                buf: Vec<u8>,
                proto: Meta<'static>,
                _pin: core::marker::PhantomPinned,
            }

            impl MetaOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = Meta::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, Meta<'static>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = proto;
                    }
                    Ok(pinned)
                }
            }

            pub struct MetaOwned {
                inner: core::pin::Pin<Box<MetaOwnedInner>>,
            }

            impl MessageInfo for MetaOwned {
                const PATH: &'static str = "quix.net.Meta";
            }

            #[allow(dead_code)]
            impl MetaOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto(&self) -> &Meta {
                    &self.inner.proto
                }
            }

            impl core::fmt::Debug for MetaOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.fmt(f)
                }
            }

            impl Deref for MetaOwned {
                type Target = Meta<'static>;

                fn deref(&self) -> &Self::Target {
                    &self.inner.proto
                }
            }

            impl DerefMut for MetaOwned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut self.inner.as_mut().get_unchecked_mut().proto }
                }
            }

            impl TryFrom<Vec<u8>> for MetaOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: MetaOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for MetaOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.deref().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            #[cfg(feature = "test_helpers")]
            impl<'a> From<Meta<'a>> for MetaOwned {
                fn from(proto: Meta) -> Self {
                    use quick_protobuf::{MessageWrite, Writer};

                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    proto.write_message(&mut writer).expect("bad proto serialization");
                    Self { inner: MetaOwnedInner::new(buf).unwrap() }
                }
            }
            
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Request<'a> {
    pub procid: Option<Cow<'a, [u8]>>,
    pub correlation: Option<i64>,
    pub method: Cow<'a, str>,
    pub body: Cow<'a, [u8]>,
}

impl<'a> MessageInfo for Request<'a> {
    const PATH : &'static str = "quix.net.Request";
}

impl<'a> MessageRead<'a> for Request<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.procid = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(16) => msg.correlation = Some(r.read_int64(bytes)?),
                Ok(26) => msg.method = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(34) => msg.body = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Request<'a> {
    fn get_size(&self) -> usize {
        0
        + self.procid.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.correlation.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + 1 + sizeof_len((&self.method).len())
        + 1 + sizeof_len((&self.body).len())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.procid { w.write_with_tag(10, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) = self.correlation { w.write_with_tag(16, |w| w.write_int64(*s))?; }
        w.write_with_tag(26, |w| w.write_string(&**&self.method))?;
        w.write_with_tag(34, |w| w.write_bytes(&**&self.body))?;
        Ok(())
    }
}


            #[derive(Debug)]
            struct RequestOwnedInner {
                buf: Vec<u8>,
                proto: Request<'static>,
                _pin: core::marker::PhantomPinned,
            }

            impl RequestOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = Request::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, Request<'static>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = proto;
                    }
                    Ok(pinned)
                }
            }

            pub struct RequestOwned {
                inner: core::pin::Pin<Box<RequestOwnedInner>>,
            }

            impl MessageInfo for RequestOwned {
                const PATH: &'static str = "quix.net.Request";
            }

            #[allow(dead_code)]
            impl RequestOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto(&self) -> &Request {
                    &self.inner.proto
                }
            }

            impl core::fmt::Debug for RequestOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.fmt(f)
                }
            }

            impl Deref for RequestOwned {
                type Target = Request<'static>;

                fn deref(&self) -> &Self::Target {
                    &self.inner.proto
                }
            }

            impl DerefMut for RequestOwned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut self.inner.as_mut().get_unchecked_mut().proto }
                }
            }

            impl TryFrom<Vec<u8>> for RequestOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: RequestOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for RequestOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.deref().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            #[cfg(feature = "test_helpers")]
            impl<'a> From<Request<'a>> for RequestOwned {
                fn from(proto: Request) -> Self {
                    use quick_protobuf::{MessageWrite, Writer};

                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    proto.write_message(&mut writer).expect("bad proto serialization");
                    Self { inner: RequestOwnedInner::new(buf).unwrap() }
                }
            }
            
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Response<'a> {
    pub correlation: Option<i64>,
    pub body: Cow<'a, [u8]>,
}

impl<'a> MessageInfo for Response<'a> {
    const PATH : &'static str = "quix.net.Response";
}

impl<'a> MessageRead<'a> for Response<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.correlation = Some(r.read_int64(bytes)?),
                Ok(18) => msg.body = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Response<'a> {
    fn get_size(&self) -> usize {
        0
        + self.correlation.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + 1 + sizeof_len((&self.body).len())
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.correlation { w.write_with_tag(8, |w| w.write_int64(*s))?; }
        w.write_with_tag(18, |w| w.write_bytes(&**&self.body))?;
        Ok(())
    }
}


            #[derive(Debug)]
            struct ResponseOwnedInner {
                buf: Vec<u8>,
                proto: Response<'static>,
                _pin: core::marker::PhantomPinned,
            }

            impl ResponseOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = Response::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, Response<'static>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = proto;
                    }
                    Ok(pinned)
                }
            }

            pub struct ResponseOwned {
                inner: core::pin::Pin<Box<ResponseOwnedInner>>,
            }

            impl MessageInfo for ResponseOwned {
                const PATH: &'static str = "quix.net.Response";
            }

            #[allow(dead_code)]
            impl ResponseOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto(&self) -> &Response {
                    &self.inner.proto
                }
            }

            impl core::fmt::Debug for ResponseOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.fmt(f)
                }
            }

            impl Deref for ResponseOwned {
                type Target = Response<'static>;

                fn deref(&self) -> &Self::Target {
                    &self.inner.proto
                }
            }

            impl DerefMut for ResponseOwned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut self.inner.as_mut().get_unchecked_mut().proto }
                }
            }

            impl TryFrom<Vec<u8>> for ResponseOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: ResponseOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for ResponseOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.deref().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            #[cfg(feature = "test_helpers")]
            impl<'a> From<Response<'a>> for ResponseOwned {
                fn from(proto: Response) -> Self {
                    use quick_protobuf::{MessageWrite, Writer};

                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    proto.write_message(&mut writer).expect("bad proto serialization");
                    Self { inner: ResponseOwnedInner::new(buf).unwrap() }
                }
            }
            
