mod status;
mod body;

#[cfg(feature="ws")]
mod ws;

pub use status::Status;
pub use body::Body;

use crate::headers::{Header, Headers, SetHeader, Value};

pub struct Response {
    status:  Status,
    headers: Headers,
    body:    Option<Body>,
}

impl Response {
    #[inline]
    pub fn of(status: Status) -> Self {
        Self {
            status,
            headers: Headers::with_capacity(4),
            body:    None
        }
    }
}

impl Response {
    pub const fn status(&self) -> Status {
        self.status
    }

    pub fn header(&self, header: Header) -> Option<&str> {
        self.headers.get(header)
    }

    pub const fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }
}

impl Response {
    #[inline]
    pub fn with(mut self, header: Header, value: impl SetHeader) -> Self {
        self.headers.set(header, value);
        self
    }

    #[inline]
    pub fn with_body(mut self)
}
