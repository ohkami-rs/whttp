use ::std::pin::Pin;
use ::unsaferef::UnsafeCow;
use ::futures_core::Stream;

pub enum Body {
    Payload(UnsafeCow<[u8]>),

    #[cfg(feature="sse")]
    Stream(Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>),

    #[cfg(feature="ws")]
    WebSocket(()),
}
