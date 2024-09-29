use ::std::{borrow::Cow, pin::Pin};

#[cfg(feature="sse")]
use ::futures_core::Stream;

pub enum Body {
    Payload(Cow<'static, [u8]>),

    #[cfg(feature="sse")]
    Stream(Pin<Box<dyn Stream<Item = String> + Send>>),

    #[cfg(feature="ws")]
    WebSocket(()),
}
