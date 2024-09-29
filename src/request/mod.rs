mod method;

pub use method::Method;

use crate::headers::{Header, Headers, SetHeader, Value};
use crate::bytes::{Bytes, IntoBytes, IntoStr, Str};
use crate::Response;
use ::percent_encoding::percent_decode_str;

const BUF_SIZE: usize = 1024;

pub struct Request {
    __buf__: Option<Box<[u8; BUF_SIZE]>>,
    method:  Method,
    path:    Str,
    query:   Option<Str>,
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

    pub fn query(&self) -> Option<std::borrow::Cow<str>> {
        match &self.query {
            Some(s) => match percent_decode_str(s).decode_utf8() {
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
    pub fn set_query(&mut self, query: impl IntoStr) {
        self.query = Some(query.into_str());
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
            path:    Str::Ref(unsafe {::unsaferef::UnsafeRef::new("/")}),
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
        this.method = method;
        Ok(())
    }

    #[inline]
    /// SAFETY: `bytes` must be alive as long as `path` of `this` is in use
    pub unsafe fn parse_path(this: &mut Self, bytes: &[u8]) -> Result<(), Response> {
        
    }
}
