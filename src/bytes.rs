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

    impl IntoBytes for Vec<u8> {
        #[inline]
        fn into_bytes(self) -> Bytes {
            Bytes::Own(self)
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