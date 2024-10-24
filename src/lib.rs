#[cfg(all(
    feature="ws",
    not(any(
        feature="rt_tokio",
        feature="rt_async-std",
        feature="rt_smol",
        feature="rt_glommio",
    ))
))]
compile_error! {"`ws` feature can't be activate without a `rt_*` feature"}

#[cfg(all(
    feature="http1",
    not(any(
        feature="rt_tokio",
        feature="rt_async-std",
        feature="rt_smol",
        feature="rt_glommio",
    ))
))]
compile_error! {"`http1` feature can't be activate without a `rt_*` feature"}

#[cfg(any(
    all(feature="rt_tokio",feature="rt_async-std"),
    all(feature="rt_tokio",feature="rt_smol"),
    all(feature="rt_tokio",feature="rt_glommio"),
    all(feature="rt_async-std",feature="rt_smol"),
    all(feature="rt_async-std",feature="rt_glommio"),
    all(feature="rt_smol",feature="rt_glommio"),
))]
compile_error! {"more than one runtime features can't be activated once"}

pub mod headers;
pub mod request;
pub mod response;

pub mod header {pub use crate::headers::standard::*;}
pub use headers::{Header, Value, Headers};
pub use request::{Method, Request};
pub use response::{Status, Response};

#[cfg(feature="ws")]
pub mod ws {pub use mews::*;}

#[cfg(feature="http1")]
pub mod http1;

#[cfg(any(feature="http1"))]
mod io {
    #[cfg(feature="rt_tokio")]
    pub use tokio::io::{AsyncReadExt as Read, AsyncWriteExt as Write};

    #[cfg(feature="rt_async-std")]
    pub use async_std::io::{ReadExt as Read, WriteExt as Write};

    #[cfg(feature="rt_smol")]
    pub use smol::io::{AsyncReadExt as Read, AsyncWriteExt as Write};

    #[cfg(feature="rt_glommio")]
    pub use futures_util::{AsyncReadExt as Read, AsyncWriteExt as Write};
}

pub mod util {
    pub use crate::headers::util::*;

    ////////////////////////////////////////////////////////////////////////////////////////////
    /// internal ///////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////////

    use unsaferef::{UnsafeCow, UnsafeRef};

    pub(crate) type Bytes = UnsafeCow<[u8]>;
    
    pub trait IntoBytes: Sized {
        /// SAFETY: `self` is owned type or reference valid whenever the `Bytes` can be accessed
        fn into_bytes(self) -> Bytes;
    }
    const _: () = {
        impl IntoBytes for &'static [u8] {
            #[inline]
            fn into_bytes(self) -> Bytes {
                Bytes::Ref(unsafe {UnsafeRef::new(self)})
            }
        }
        impl IntoBytes for &'static str {
            #[inline]
            fn into_bytes(self) -> Bytes {
                Bytes::Ref(unsafe {UnsafeRef::new(self.as_bytes())})
            }
        }
    
        impl IntoBytes for Vec<u8> {
            #[inline]
            fn into_bytes(self) -> Bytes {
                Bytes::Own(self)
            }
        }
        impl IntoBytes for String {
            #[inline]
            fn into_bytes(self) -> Bytes {
                Bytes::Own(String::into_bytes(self))
            }
        }
    
        impl IntoBytes for std::borrow::Cow<'static, [u8]> {
            fn into_bytes(self) -> Bytes {
                match self {
                    std::borrow::Cow::Borrowed(b) => b.into_bytes(),
                    std::borrow::Cow::Owned(o) => o.into_bytes()
                }
            }
        }
        impl IntoBytes for std::borrow::Cow<'static, str> {
            fn into_bytes(self) -> Bytes {
                match self {
                    std::borrow::Cow::Borrowed(b) => IntoBytes::into_bytes(b),
                    std::borrow::Cow::Owned(o) => IntoBytes::into_bytes(o)
                }
            }
        }
    };
    
    pub(crate) type Str = UnsafeCow<str>;
    
    pub trait IntoStr: Sized {
        /// SAFETY: `self` is owned type or reference valid whenever the `Str` can be accessed
        fn into_str(self) -> Str;
    }
    const _: () = {
        impl IntoStr for &'static str {
            #[inline]
            fn into_str(self) -> Str {
                Str::Ref(unsafe {UnsafeRef::new(self)})
            }
        }
    
        impl IntoStr for String {
            #[inline]
            fn into_str(self) -> Str {
                Str::Own(self)
            }
        }
    
        impl IntoStr for std::borrow::Cow<'static, str> {
            fn into_str(self) -> Str {
                match self {
                    std::borrow::Cow::Borrowed(b) => b.into_str(),
                    std::borrow::Cow::Owned(o) => o.into_str()
                }
            }
        }
    };
}
