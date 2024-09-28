use unsaferef::{UnsafeCow, UnsafeRef};

pub(crate) type Bytes = UnsafeCow<[u8]>;

pub trait IntoBytes: Sized {
    /// SAFETY: `self` is owned type or reference valid whenever the `Bytes` can be accessed
    unsafe fn into_bytes(self) -> Bytes;
}
const _: () = {
    impl IntoBytes for &[u8] {
        #[inline]
        unsafe fn into_bytes(self) -> Bytes {
            Bytes::Ref(UnsafeRef::new(self))
        }
    }

    impl IntoBytes for Vec<u8> {
        #[inline]
        unsafe fn into_bytes(self) -> Bytes {
            Bytes::Own(self)
        }
    }

    impl IntoBytes for std::borrow::Cow<'_, [u8]> {
        unsafe fn into_bytes(self) -> Bytes {
            match self {
                std::borrow::Cow::Borrowed(b) => b.into_bytes(),
                std::borrow::Cow::Owned(o) => o.into_bytes()
            }
        }
    }
};

pub(crate) type Str = UnsafeCow<str>;

pub trait IntoStr: Sized {
    /// SAFETY: `self` is owned type or reference valid whenever the `Str` can be accessed
    unsafe fn into_str(self) -> Str;
}
const _: () = {
    impl IntoStr for &str {
        #[inline]
        unsafe fn into_str(self) -> Str {
            Str::Ref(UnsafeRef::new(self))
        }
    }

    impl IntoStr for String {
        #[inline]
        unsafe fn into_str(self) -> Str {
            Str::Own(self)
        }
    }

    impl IntoStr for std::borrow::Cow<'_, str> {
        unsafe fn into_str(self) -> Str {
            match self {
                std::borrow::Cow::Borrowed(b) => b.into_str(),
                std::borrow::Cow::Owned(o) => o.into_str()
            }
        }
    }
};
