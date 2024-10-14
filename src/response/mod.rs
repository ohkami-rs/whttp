mod status;
pub use status::Status;

mod body;
pub use body::Body;

use crate::headers::{Header, Headers, SetHeader, Value};
use ::std::borrow::Cow;
use ::serde::Serialize;

#[cfg(feature="sse")]
use ::futures_core::Stream;

#[cfg(feature="ws")]
use ::mews::WebSocket;

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

    pub fn header(&self, header: &Header) -> Option<&str> {
        self.headers.get(header)
    }

    pub const fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }
}

impl Response {
    #[inline]
    pub fn set(&mut self, header: &Header, value: impl SetHeader) -> &mut Self {
        self.headers.set(header, value);
        self
    }

    #[inline]
    pub fn append(&mut self, header: &Header, value: impl Into<Value>) -> &mut Self {
        self.headers.append(header, value);
        self
    }

    #[inline(always)]
    pub fn with(mut self, header: &Header, value: impl Into<Value>) -> Self {
        self.headers.insert(header, value);
        self
    }

    #[inline]
    pub fn with_payload(
        mut self,
        content_type: &'static str,
        payload: impl Into<Cow<'static, [u8]>>
    ) -> Self {
        use crate::header::{ContentLength, ContentType};

        let payload: Cow<'static, [u8]> = payload.into();
        self.set(ContentType, content_type)
            .set(ContentLength, payload.len());
        self.body = Some(Body::Payload(payload));
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
        use crate::header::{ContentType, CacheControl, TransferEncoding};

        self.set(ContentType, "text/event-stream")
            .set(CacheControl, "no-cache, must-revalidate")
            .set(TransferEncoding, "chunked");
        self.body = Some(Body::Stream(Box::pin(stream)));
        self
    }

    #[cfg(feature="ws")]
    pub fn with_websocket(
        mut self,
        websocket: WebSocket
    ) -> Self {
        self.body = Some(Body::WebSocket(websocket));
        self
    }
}

const _: () = {
    impl PartialEq for Response {
        fn eq(&self, other: &Self) -> bool {
            self.status == other.status &&
            self.headers == other.headers &&
            self.body == other.body
        }
    }

    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Response")
                .field("status", &self.status)
                .field("body", &self.body)
                .field("", &self.headers)
                .finish()
        }
    }
};
