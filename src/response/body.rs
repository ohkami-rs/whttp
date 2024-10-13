use std::borrow::Cow;

#[cfg(feature="sse")]
use ::futures_core::Stream;

#[cfg(feature="ws")]
use ::mews::WebSocket;

pub enum Body {
    Payload(Cow<'static, [u8]>),

    #[cfg(feature="sse")]
    Stream(std::pin::Pin<Box<dyn Stream<Item = String> + Send>>),

    #[cfg(feature="ws")]
    WebSocket(WebSocket),
}

impl PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Body::Payload(this), Body::Payload(other)) => this == other,

            #[cfg(feature="sse")]
            (Body::Stream(_), Body::Stream(_)) => false/* can't compare */,

            #[cfg(feature="ws")]
            (Body::WebSocket(_), Body::WebSocket(_)) => false/* can't compare */,

            _ => false
        }
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Payload(p) => f.debug_tuple("Body::Payload")
                .field(p)
                .finish(),
            
            #[cfg(feature="sse")]
            Self::Stream(_) => f.debug_tuple("Body::Stream")
                .field(&"...")
                .finish(),

            #[cfg(feature="ws")]
            Self::WebSocket(_) => f.debug_tuple("Body::WebSocket")
                .field(&"...")
                .finish()
        }
    }
}
