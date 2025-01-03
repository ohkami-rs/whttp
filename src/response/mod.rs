mod status;
pub use status::Status;

mod body;
pub use body::Body;


use crate::headers::{Header, Value, Headers, SetHeader};
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

    pub fn headers(&self) -> &Headers {
        &self.headers
    }
    #[inline]
    pub fn header(&self, header: &Header) -> Option<&str> {
        self.headers.get(header)
    }

    #[inline]
    pub const fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    pub fn payload(&self) -> Option<&[u8]> {
        match self.body()? {
            Body::Payload(p) => Some(p),

            #[cfg(any(feature="sse", feature="ws"))]
            _ => None
        }
    }

    pub fn take(&mut self, header: &Header) -> Option<Value> {
        self.headers.remove(header)
    }

    #[inline]
    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }
}

impl Response {
    pub fn set_status(&mut self, status: Status) -> &mut Self {
        self.status = status;
        self
    }

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

    /// drop `body`, `Content-Type` and `Content-Length` if exists
    pub fn drop_body(&mut self) -> &mut Self {
        use crate::header::{ContentLength, ContentType};

        self.set(ContentType, None)
            .set(ContentLength, None);
        self.body = None;
        self
    }

    /// drop only `body`, remaining `Content-Type` and `Content-Length`
    /// if exists for responding to a HEAD request
    pub fn as_head(&mut self) -> &mut Self {
        self.body = None;
        self
    }

    #[inline]
    pub fn set_payload(
        &mut self,
        content_type: &'static str,
        payload: impl Into<Cow<'static, [u8]>>
    ) -> &mut Self {
        use crate::header::{ContentLength, ContentType};

        let payload: Cow<'static, [u8]> = payload.into();
        self.set(ContentType, content_type)
            .set(ContentLength, payload.len());
        self.body = Some(Body::Payload(payload));
        self
    }

    #[inline]
    pub fn set_text(&mut self, text: impl Into<Cow<'static, str>>) -> &mut Self {
        let text: Cow<'static, str> = text.into();
        self.set_payload("text/plain; charset=UTF-8", match text {
            Cow::Borrowed(b) => Cow::Borrowed(b.as_bytes()),
            Cow::Owned(o) => Cow::Owned(o.into_bytes())
        })
    }

    #[inline]
    pub fn set_html(&mut self, html: impl Into<Cow<'static, str>>) -> &mut Self {
        let text: Cow<'static, str> = html.into();
        self.set_payload("text/html; charset=UTF-8", match text {
            Cow::Borrowed(b) => Cow::Borrowed(b.as_bytes()),
            Cow::Owned(o) => Cow::Owned(o.into_bytes())
        })
    }

    #[inline]
    pub fn set_json(&mut self, json: impl Serialize) -> &mut Self {
        let json: Vec<u8> = ::serde_json::to_vec(&json).expect("failed to serialize");
        self.set_payload("application/json", json)
    }

    #[cfg(feature="sse")]
    pub fn set_stream(
        &mut self,
        stream: impl Stream<Item = String> + Send + 'static
    ) -> &mut Self {
        use crate::header::{ContentType, CacheControl, TransferEncoding};

        self.set(ContentType, "text/event-stream")
            .set(CacheControl, "no-cache, must-revalidate")
            .set(TransferEncoding, "chunked");
        self.body = Some(Body::Stream(Box::pin(stream)));
        self
    }

    #[cfg(feature="ws")]
    pub fn set_websocket(
        &mut self,
        sec_websocket_accept: String,
        websocket: WebSocket
    ) -> &mut Self {
        use crate::header::{Connection, Upgrade, SecWebSocketAccept};

        self.status = Status::SwitchingProtocols;
        self.set(Connection, "Upgrade")
            .set(Upgrade, "websocket")
            .set(SecWebSocketAccept, sec_websocket_accept);
        self.body = Some(Body::WebSocket(websocket));
        self
    }
}

impl Response {
    pub fn with_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    #[inline(always)]
    pub fn with(mut self, header: &Header, value: impl Into<Value>) -> Self {
        self.headers.append(header, value);
        self
    }

    /// without `body`, `Content-Type` and `Content-Length`
    pub fn without_body(mut self) -> Self {
        self.drop_body();
        self
    }

    /// without `body` but remaining `Content-Type` and `Content-Length`
    /// if exists for responding to a HEAD request
    pub fn into_head(mut self) -> Self {
        self.as_head();
        self
    }

    #[inline]
    pub fn with_payload(
        mut self,
        content_type: &'static str,
        payload: impl Into<Cow<'static, [u8]>>
    ) -> Self {
        self.set_payload(content_type, payload);
        self
    }

    #[inline]
    pub fn with_text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.set_text(text);
        self
    }

    #[inline]
    pub fn with_html(mut self, html: impl Into<Cow<'static, str>>) -> Self {
        self.set_html(html);
        self
    }

    #[inline]
    pub fn with_json(mut self, json: impl Serialize) -> Self {
        self.set_json(json);
        self
    }

    #[cfg(feature="sse")]
    pub fn with_stream(
        mut self,
        stream: impl Stream<Item = String> + Send + 'static
    ) -> Self {
        self.set_stream(stream);
        self
    }

    #[cfg(feature="ws")]
    pub fn with_websocket(
        mut self,
        sec_websocket_accept: String,
        websocket: WebSocket
    ) -> Self {
        self.set_websocket(sec_websocket_accept, websocket);
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
                .field("<status>", &self.status)
                .field("<headers>", &self.headers)
                .field("<body>", &self.body)
                .finish()
        }
    }
};
