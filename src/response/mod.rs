mod status;
pub use status::Status;

use crate::headers::{Header, Headers, SetHeader, Value};
use crate::header::{ContentLength, ContentType};
use ::std::borrow::Cow;
use ::serde::Serialize;

#[cfg(feature="sse")]
use ::futures_core::Stream;

pub struct Response<B: Body> {
    status:  Status,
    headers: Headers,
    body:    Option<B>,
}

pub trait Body {
    fn payload(payload: impl Into<Cow<'static, [u8]>>) -> Self;

    #[cfg(feature="sse")]
    fn stream(stream: impl Stream<Item = String> + Send + 'static) -> Self;

    #[cfg(feature="ws")]
    type WebSocket;
    #[cfg(feature="ws")]
    fn websocket(websocket: Self::WebSocket) -> Self;
}

impl<B: Body> Response<B> {
    #[inline]
    pub fn of(status: Status) -> Self {
        Self {
            status,
            headers: Headers::with_capacity(4),
            body:    None
        }
    }
}

impl<B: Body> Response<B> {
    pub const fn status(&self) -> Status {
        self.status
    }

    pub fn header(&self, header: Header) -> Option<&str> {
        self.headers.get(header)
    }

    pub const fn body(&self) -> Option<&B> {
        self.body.as_ref()
    }
}

impl<B: Body> Response<B> {
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

    #[inline]
    pub fn with_payload(
        mut self,
        content_type: &'static str,
        payload: impl Into<Cow<'static, [u8]>>
    ) -> Self {
        let payload: Cow<'static, [u8]> = payload.into();
        self.set(ContentType, content_type)
            .set(ContentLength, payload.len());
        self.body = Some(Body::payload(payload));
        self
    }

    #[inline]
    pub fn with_text(self, text: impl Into<Cow<'static, str>>) -> Self {
        let text: Cow<'static, str> = text.into();
        self.with_payload("text/plain; charset=UTF-8", match text {
            Cow::Borrowed(b) => Cow::Borrowed(b.as_bytes()),
            Cow::Owned(o) => Cow::Owned(o.into_bytes())
        })
    }

    #[inline]
    pub fn with_html(self, html: impl Into<Cow<'static, str>>) -> Self {
        let text: Cow<'static, str> = html.into();
        self.with_payload("text/html; charset=UTF-8", match text {
            Cow::Borrowed(b) => Cow::Borrowed(b.as_bytes()),
            Cow::Owned(o) => Cow::Owned(o.into_bytes())
        })
    }

    #[inline]
    pub fn with_json(self, json: impl Serialize) -> Self {
        let json: Vec<u8> = ::serde_json::to_vec(&json).expect("failed to serialize");
        self.with_payload("application/json", json)
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
        self.body = Some(Body::stream(stream));
        self
    }

    #[cfg(feature="ws")]
    pub fn with_websocket(
        mut self,
        websocket: impl Into<B::WebSocket>
    ) -> Self {
        self.body = Some(Body::websocket(websocket.into()));
        self
    }
}

const _: () = {
    impl<B: Body + PartialEq> PartialEq for Response<B> {
        fn eq(&self, other: &Self) -> bool {
            self.status == other.status &&
            self.headers == other.headers &&
            self.body == other.body
        }
    }

    impl<B: Body + std::fmt::Debug> std::fmt::Debug for Response<B> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Response")
                .field("status", &self.status)
                .field("body", &self.body)
                .field("", &self.headers)
                .finish()
        }
    }
};
