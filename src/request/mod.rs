mod method;

pub use method::Method;

use crate::headers::{Header, Headers, SetHeader, Value};
use crate::bytes::{Bytes, IntoBytes, IntoStr, Str};
use crate::Response;
use ::unsaferef::UnsafeRef;
use ::percent_encoding::percent_decode;

const BUF_SIZE: usize = 1024;

pub struct Request {
    __buf__: Option<Box<[u8; BUF_SIZE]>>,
    method:  Method,
    path:    Str,
    query:   Option<Bytes>,
    headers: Headers,
    body:    Option<Bytes>,
}

impl Request {
    pub fn method(&self) -> Method {
        self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query_raw(&self) -> Option<&[u8]> {
        match &self.query {
            Some(q) => Some(&q),
            None => None
        }
    }
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

    pub fn header(&self, header: Header) -> Option<&str> {
        self.headers.get(header)
    }

    pub fn body(&self) -> Option<&[u8]> {
        match &self.body {
            Some(b) => Some(&b),
            None => None
        }
    }
}

impl Request {
    #[inline]
    pub fn set(&mut self, header: Header, setter: impl SetHeader) {
        self.headers.set(header, setter);
    }

    #[inline]
    pub fn append(&mut self, header: Header, value: impl Into<Value>) {
        self.headers.append(header, value);
    }

    #[inline]
    pub fn set_method(&mut self, method: Method) {
        self.method = method;
    }

    #[inline]
    pub fn set_path(&mut self, path: impl IntoStr) {
        self.path = path.into_str();
    }

    #[inline]
    pub fn set_query(&mut self, query: impl IntoBytes) {
        self.query = Some(query.into_bytes());
    }

    #[inline]
    pub fn set_body(&mut self, body: impl IntoBytes) {
        self.body = Some(body.into_bytes());
    }
}

impl Request {
    pub fn init_buf() -> Self {
        Self {
            __buf__: Some(Box::new([0; BUF_SIZE])),
            method:  Method::GET,
            path:    Str::Ref(unsafe {UnsafeRef::new("/")}),
            query:   None,
            headers: Headers::with_capacity(8),
            body:    None,
        }
    }

    pub fn buf(&self) -> Option<&[u8; BUF_SIZE]> {
        self.__buf__.as_deref()
    }
    pub fn buf_mut(&mut self) -> &mut Box<[u8; BUF_SIZE]> {
        if self.__buf__.is_none() {
            self.__buf__ = Some(Box::new([0; BUF_SIZE]));
        }
        unsafe {self.__buf__.as_mut().unwrap_unchecked()}
    }

    #[inline]
    pub fn parse_method(this: &mut Self, bytes: &[u8]) -> Result<(), Response> {
        let method = Method::from_bytes(bytes)
            .ok_or_else(|| Response::NotImplemented().with_text("custom method is not available"))?;
        Ok(this.method = method)
    }

    #[inline]
    /// SAFETY: `bytes` must be alive as long as `path` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn parse_path(this: &mut Self, bytes: &[u8]) -> Result<(), Response> {
        (bytes.len() > 0 && *bytes.get_unchecked(0) == b'/')
            .then_some(())
            .ok_or_else(Response::BadRequest)?;
        let path = std::str::from_utf8(bytes)
            .map_err(|_|Response::BadRequest())?;
        Ok(this.path = Str::Ref(UnsafeRef::new(path)))
    }

    #[inline]
    /// Store bytes like `query=value`, `q1=v1&q2=v2` into `this.query`.
    /// 
    /// SAFETY: `bytes` must be alive as long as `query` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn parse_query(this: &mut Self, bytes: &[u8]) {
        this.query = Some(Bytes::Ref(UnsafeRef::new(bytes)))
    }

    #[inline]
    /// Parse bytes like `Header-Name: Value` as a pair of `Header`, `Value`, and
    /// append into `this.headers`.
    /// 
    /// SAFETY: `bytes` must be alive as long as `headers` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn parse_header(this: &mut Self, mut bytes: &[u8]) -> Result<(), Response> {
        let header = 'header: {
            for i in 0..bytes.len() {
                if *bytes.get_unchecked(i) == b':' {
                    let (header_bytes, rest) = (
                        bytes.get_unchecked(0..i),
                        bytes.get_unchecked(i..)
                    );
                    bytes = rest;
                    break 'header Some(Header::parse(header_bytes)
                        .map_err(|e| Response::BadRequest().with_text(e.to_string()))?
                    )
                }
            }; None
        }.ok_or_else(Response::BadRequest)?;

        let value = {
            while let Some((b' ', rest)) = bytes.split_first() {
                bytes = rest;
            }
            Value::parse(bytes)
                .map_err(|e| Response::BadRequest().with_text(e.to_string()))?
        };

        Ok({this.headers.append(header, value);})
    }

    #[inline]
    /// SAFETY: `bytes` must be alive as long as `body` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn parse_body(this: &mut Self, bytes: &[u8]) {
        this.body = Some(Bytes::Ref(UnsafeRef::new(bytes)))
    }
}
