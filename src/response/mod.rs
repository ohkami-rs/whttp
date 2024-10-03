mod status;
mod body;

#[cfg(feature="ws")]
mod ws;

pub use status::Status;
pub use body::Body;

use crate::headers::{Header, Headers, SetHeader, Value};
use crate::header::{ContentLength, ContentType};
use ::std::borrow::Cow;
use ::serde::Serialize;

#[cfg(feature="sse")]
use ::futures_core::Stream;

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
    pub fn set(&mut self, header: Header, value: impl SetHeader) -> &mut Self {
        self.headers.set(header, value);
        self
    }

    #[inline]
    pub fn append(mut self, header: Header, value: impl Into<Value>) -> Self {
        self.headers.append(header, value);
        self
    }

    #[inline(always)]
    pub fn with(mut self, header: Header, value: impl Into<Value>) -> Self {
        self.headers.insert(header, value);
        self
    }

    pub fn with_payload(
        mut self,
        content_type: &'static str,
        payload: impl Into<Cow<'static, [u8]>>
    ) -> Self {
        let payload: Cow<'static, [u8]>= payload.into();
        self.set(ContentType, content_type)
            .set(ContentLength, payload.len());
        self.body = Some(Body::Payload(payload));
        self
    }

    #[inline]
    pub fn with_text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        let text: Cow<'static, str> = text.into();
        self.headers
            .set(ContentType, "text/plain; charset=UTF-8")
            .set(ContentLength, text.len());
        self.body = Some(Body::Payload(match text {
            Cow::Borrowed(b) => Cow::Borrowed(b.as_bytes()),
            Cow::Owned(o) => Cow::Owned(o.into_bytes())
        }));
        self
    }

    #[inline]
    pub fn with_html(mut self, html: impl Into<Cow<'static, str>>) -> Self {
        let html: Cow<'static, str> = html.into();
        self.headers
            .set(ContentType, "text/html; charset=UTF-8")
            .set(ContentLength, html.len());
        self.body = Some(Body::Payload(match html {
            Cow::Borrowed(b) => Cow::Borrowed(b.as_bytes()),
            Cow::Owned(o) => Cow::Owned(o.into_bytes())
        }));
        self
    }

    #[inline]
    pub fn with_json(mut self, json: impl Serialize) -> Self {
        let json: Vec<u8> = ::serde_json::to_vec(&json).expect("failed to serialize");
        self.headers
            .set(ContentType, "application/json")
            .set(ContentLength, json.len());
        self.body = Some(Body::Payload(Cow::Owned(json)));
        self
    }

    #[cfg(feature="sse")]
    pub fn with_stream(
        mut self,
        stream: impl Stream<Item = String> + Send + 'static
    ) -> Self {
        use crate::header::{CacheControl, TransferEncoding};

        self.headers
            .set(ContentType, "text/event-stream")
            .set(CacheControl, "no-cache, must-revalidate")
            .set(TransferEncoding, "chunked");
        self.body = Some(Body::Stream(Box::pin(stream)));
        self
    }
}
