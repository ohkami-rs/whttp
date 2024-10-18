mod method;
mod memory;

pub use method::Method;
use memory::Memory;

use crate::headers::{Header, Headers, SetHeader, Value};
use crate::util::{Bytes, IntoBytes, IntoStr, Str};
use ::std::{borrow::Cow, net::IpAddr};
use ::unsaferef::UnsafeRef;
use ::percent_encoding::{percent_decode, percent_encode, NON_ALPHANUMERIC};

// TODO: rethink the size of this struct taking CPU cacheline to consideration
// (current: 144 bytes -> if remove some 16 bytes: 128 bytes)
pub struct Request {
    __buf__: Option<Box<[u8; parse::BUF_SIZE]>>,
    ip:      Option<IpAddr>,
    memory:  Memory,
    method:  Method,
    path:    Str,
    query:   Option<Bytes>,
    headers: Headers,
    body:    Option<Bytes>,
}

impl Request {
    #[inline]
    pub fn of(method: Method, path: impl IntoStr) -> Self {
        Self {
            __buf__: None,
            ip: None,
            memory:  Memory::new(),
            method,
            path:    path.into_str(),
            query:   None,
            headers: Headers::with_capacity(4),
            body:    None
        }
    }

    #[inline]
    pub fn with(mut self, header: &Header, value: impl Into<Value>) -> Self {
        self.headers.insert(header, value);
        self
    }

    pub fn with_query(mut self, key: impl IntoStr, value: impl IntoStr) -> Self {
        let key = key.into_str();
        let key = <Cow<str>>::from(percent_encode(key.as_bytes(), NON_ALPHANUMERIC));

        let value = value.into_str();
        let value = <Cow<str>>::from(percent_encode(value.as_bytes(), NON_ALPHANUMERIC));

        match &mut self.query {
            None => self.query = Some(Bytes::Own(
                Vec::with_capacity(key.len() + 1 + value.len())
            )),
            Some(query) => {
                let query = query.to_mut();
                query.push(b'&');
                query.reserve(key.len() + 1 + value.len());
            }
        }

        let Some(Bytes::Own(query)) = &mut self.query else {unreachable!()};
        query.extend_from_slice(key.as_bytes());
        query.push(b'=');
        query.extend_from_slice(value.as_bytes());

        self
    }

    #[inline]
    pub fn with_body(mut self, content_type: &'static str, body: impl IntoBytes) -> Self {
        use crate::header::{ContentType, ContentLength};
        let body = body.into_bytes();
        self.set(ContentType, content_type)
            .set(ContentLength, body.len());
        self.body = Some(body);
        self
    }

    pub fn with_text(self, text: impl Into<Cow<'static, str>>) -> Self {
        self.with_body("text/plain; charset=UTF-8", match text.into() {
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(s)    => Cow::Owned(s.into_bytes())
        })
    }

    pub fn with_json(self, json: impl ::serde::Serialize) -> Self {
        self.with_body("application/json",
            ::serde_json::to_vec(&json).expect("failed to serialize")
        )
    }
}

impl Request {
    pub const fn ip(&self) -> Option<&IpAddr> {
        match &self.ip {
            Some(ip) => Some(ip),
            None => None
        }
    }

    pub fn memory<Data: Send + Sync + 'static>(&self) -> Option<&Data> {
        self.memory.get()
    }

    pub const fn method(&self) -> Method {
        self.method
    }

    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[inline]
    pub fn query_raw(&self) -> Option<&[u8]> {
        match &self.query {
            Some(q) => Some(&q),
            None => None
        }
    }
    #[inline]
    pub fn query(&self) -> Option<std::borrow::Cow<str>> {
        match &self.query {
            Some(q) => match percent_decode(q).decode_utf8() {
                Ok(dq) => Some(dq),
                Err(_e) => {
                    #[cfg(debug_assertions)] {
                        eprintln!("`Request::query` found invalid query: {_e}");
                    }
                    None
                }
            },
            None => None
        }
    }

    #[inline]
    pub fn header(&self, header: &Header) -> Option<&str> {
        self.headers.get(header)
    }

    #[inline]
    pub fn body(&self) -> Option<&[u8]> {
        match &self.body {
            Some(b) => Some(&b),
            None => None
        }
    }
}

impl Request {
    #[inline]
    pub fn memorize<Data: Send + Sync + 'static>(&mut self, data: Data) {
        self.memory.insert(data);
    }

    #[inline]
    pub fn set(&mut self, header: &Header, setter: impl SetHeader) -> &mut Self {
        self.headers.set(header, setter);
        self
    }

    #[inline]
    pub fn append(&mut self, header: &Header, value: impl Into<Value>) -> &mut Self {
        self.headers.append(header, value);
        self
    }

    #[inline]
    pub fn set_method(&mut self, method: Method) -> &mut Self {
        self.method = method;
        self
    }

    #[inline]
    pub fn set_path(&mut self, path: impl IntoStr) -> &mut Self {
        self.path = path.into_str();
        self
    }

    #[inline]
    pub fn set_query(&mut self, query: impl IntoBytes) -> &mut Self {
        self.query = Some(query.into_bytes());
        self
    }

    #[inline]
    pub fn set_body(&mut self, body: impl IntoBytes) -> &mut Self {
        self.body = Some(body.into_bytes());
        self
    }
}

pub mod parse {
    use super::*;
    use crate::Status;
    use std::pin::Pin;

    pub const BUF_SIZE: usize = 1024;

    pub fn new() -> Request {
        Request {
            ip: None,
            /////////
            __buf__: Some(Box::new([0; BUF_SIZE])),
            memory:  Memory::new(),
            method:  Method::GET,
            path:    Str::Ref(unsafe {UnsafeRef::new("/")}),
            query:   None,
            headers: Headers::with_capacity(8),
            body:    None,
        }
    }
    pub fn new_from(ip: IpAddr) -> Request {
        Request {
            ip: Some(ip),
            /////////////
            __buf__: Some(Box::new([0; BUF_SIZE])),
            memory:  Memory::new(),
            method:  Method::GET,
            path:    Str::Ref(unsafe {UnsafeRef::new("/")}),
            query:   None,
            headers: Headers::with_capacity(8),
            body:    None,
        }
    }

    #[inline]
    pub fn clear(this: &mut Pin<&mut Request>) {
        let Some(buf) = &mut this.__buf__ else {return};
        if buf[0] == 0 {return}

        for b in &mut **buf {
            match b {
                0 => break,
                _ => *b = 0
            }
        }
        this.memory.clear();
        this.path = Str::Ref(unsafe {UnsafeRef::new("/")});
        this.query = None;
        this.headers.clear();
        this.body = None;
    }

    pub fn buf(this: Pin<&mut Request>) -> &mut Box<[u8; BUF_SIZE]> {
        let buf = &mut this.get_mut().__buf__;
        if buf.is_none() {
            *buf = Some(Box::new([0; BUF_SIZE]));
        }
        unsafe {buf.as_mut().unwrap_unchecked()}
    }

    #[inline]
    pub fn method(this: &mut Pin<&mut Request>, bytes: &[u8]) -> Result<(), Status> {
        let method = Method::from_bytes(bytes)
            .ok_or(Status::NotImplemented)?;
        Ok(this.method = method)
    }

    #[inline]
    /// SAFETY: `bytes` must be alive as long as `path` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn path(this: &mut Pin<&mut Request>, bytes: &[u8]) -> Result<(), Status> {
        (bytes.len() > 0 && *bytes.get_unchecked(0) == b'/')
            .then_some(())
            .ok_or(Status::BadRequest)?;
        let path = std::str::from_utf8(bytes)
            .map_err(|_| Status::BadRequest)?;
        Ok(this.path = Str::Ref(UnsafeRef::new(path)))
    }

    #[inline]
    /// Store bytes like `query=value`, `q1=v1&q2=v2` into `this.query`.
    /// 
    /// SAFETY: `bytes` must be alive as long as `query` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn query(this: &mut Pin<&mut Request>, bytes: &[u8]) {
        this.query = Some(Bytes::Ref(UnsafeRef::new(bytes)))
    }

    #[inline]
    /// SAFETY: `name_bytes` and `value_bytes` must be alive as long as `headers` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn header(this: &mut Pin<&mut Request>, name_bytes: &[u8], value_bytes: &[u8]) -> Result<(), Status> {
        let name  = Header::parse_mainly_standard(name_bytes).map_err(|_| Status::BadRequest)?;
        let value = Value::parse(value_bytes).map_err(|_| Status::BadRequest)?;
        this.headers.push(name, value);
        Ok(())
    }

    #[inline]
    /// SAFETY: `bytes` must be alive as long as `body` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn body(this: &mut Pin<&mut Request>, bytes: &[u8]) {
        this.body = Some(Bytes::Ref(UnsafeRef::new(bytes)))
    }
}

const _: () = {
    impl PartialEq for Request {
        fn eq(&self, other: &Request) -> bool {
            self.method == other.method &&
            self.path == other.path &&
            self.query == other.query &&
            self.headers == other.headers &&
            self.body == other.body
        }
    }

    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Request")
                .field("method", &self.method)
                .field("path", &self.path)
                .field("query", &self.query())
                .field("body", &self.body)
                .field("", &self.headers)
                .finish()
        }
    }
};
