mod method;

pub use method::Method;

use crate::headers::{Header, Headers, SetHeader, Value};
use crate::bytes::{Bytes, IntoBytes, IntoStr, Str};
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
}

pub mod parse {
    use super::*;
    use crate::Status;

    pub fn new() -> Request {
        Request {
            __buf__: Some(Box::new([0; BUF_SIZE])),
            method:  Method::GET,
            path:    Str::Ref(unsafe {UnsafeRef::new("/")}),
            query:   None,
            headers: Headers::with_capacity(8),
            body:    None,
        }
    }

    pub fn buf(this: &Request) -> Option<&[u8; BUF_SIZE]> {
        this.__buf__.as_deref()
    }
    pub fn buf_mut(this: &mut Request) -> &mut Box<[u8; BUF_SIZE]> {
        if this.__buf__.is_none() {
            this.__buf__ = Some(Box::new([0; BUF_SIZE]));
        }
        unsafe {this.__buf__.as_mut().unwrap_unchecked()}
    }

    #[inline]
    pub fn method(this: &mut Request, bytes: &[u8]) -> Result<(), Status> {
        let method = Method::from_bytes(bytes)
            .ok_or(Status::NotImplemented)?;
        Ok(this.method = method)
    }

    #[inline]
    /// SAFETY: `bytes` must be alive as long as `path` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn path(this: &mut Request, bytes: &[u8]) -> Result<(), Status> {
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
    pub unsafe fn query(this: &mut Request, bytes: &[u8]) {
        this.query = Some(Bytes::Ref(UnsafeRef::new(bytes)))
    }

    #[inline]
    /// SAFETY: `bytes` must be alive as long as `body` of `this` is in use;
    /// especially, reading from `this.buf`
    pub unsafe fn body(this: &mut Request, bytes: &[u8]) {
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
