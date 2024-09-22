#[derive(Clone)]
enum UnsafeCow {
    Ref(std::ptr::NonNull<[u8]>),
    Own(Vec<u8>)
}

#[derive(Clone)]
pub struct Bytes(UnsafeCow);
const _: () = {
    impl Bytes {
        /// SAFETY: `s` is valid reference whenever returned `Str` can be accessed
        #[inline(always)]
        pub(crate) const unsafe fn unchecked_ref(s: &[u8]) -> Self {
            Bytes(UnsafeCow::Ref(std::ptr::NonNull::new_unchecked(s as *const [u8] as *mut [u8])))
        }

        #[inline(always)]
        pub(crate) fn as_bytes(&self) -> &[u8] {
            // SAFETY: `Str` constructors' SAFETY
            match &self.0 {
                UnsafeCow::Ref(r) => unsafe {r.as_ref()},
                UnsafeCow::Own(o) => o.as_slice()
            }
        }
    }

    unsafe impl Send for Bytes {}
    unsafe impl Sync for Bytes {}

    impl From<&'static [u8]> for Bytes {
        #[inline]
        fn from(s: &'static [u8]) -> Self {
            // SAFETY: 'static reference is always valid
            unsafe {Self::unchecked_ref(s)}
        }
    }
    impl From<Vec<u8>> for Bytes {
        #[inline]
        fn from(vec: Vec<u8>) -> Self {
            Self(UnsafeCow::Own(vec))
        }
    }
    impl From<std::borrow::Cow<'static, [u8]>> for Bytes {
        fn from(cow: std::borrow::Cow<'static, [u8]>) -> Self {
            match cow {
                std::borrow::Cow::Borrowed(b) => Self::from(b),
                std::borrow::Cow::Owned(o) => Self::from(o)
            }
        }
    }
};

pub trait IntoBytes: Sized {
    /// SAFETY: `self` is owned type or reference valid whenever the `Bytes` can be accessed
    unsafe fn into_bytes(self) -> Bytes;
}
const _: () = {
    impl IntoBytes for &[u8] {
        #[inline]
        unsafe fn into_bytes(self) -> Bytes {
            Bytes::unchecked_ref(self)
        }
    }

    impl IntoBytes for Vec<u8> {
        #[inline]
        unsafe fn into_bytes(self) -> Bytes {
            Bytes::from(self)
        }
    }

    impl IntoBytes for std::borrow::Cow<'_, [u8]> {
        unsafe fn into_bytes(self) -> Bytes {
            match self {
                std::borrow::Cow::Borrowed(b) => Bytes::unchecked_ref(b),
                std::borrow::Cow::Owned(o) => Bytes::from(o)
            }
        }
    }
};

#[derive(Clone)]
pub struct Str(UnsafeCow);
const _: () = {
    impl Str {
        /// SAFETY: `s` is valid reference whenever returned `Str` can be accessed
        #[inline(always)]
        pub(crate) const unsafe fn unchecked_ref(s: &str) -> Self {
            Str(UnsafeCow::Ref(std::ptr::NonNull::new_unchecked(s.as_bytes() as *const [u8] as *mut [u8])))
        }

        pub(crate) const fn from_static(s: &'static str) -> Self {
            // SAFETY: 'static reference is always valid
            unsafe {Self::unchecked_ref(s)}
        }

        #[inline(always)]
        pub(crate) fn as_str(&self) -> &str {
            // SAFETY: `Str` constructors' SAFETY
            unsafe {std::str::from_utf8_unchecked(match &self.0 {
                UnsafeCow::Ref(r) => r.as_ref(),
                UnsafeCow::Own(o) => o.as_slice()
            })}
        }

        #[inline]
        pub(crate) fn push(&mut self, byte: u8) {
            #[cfg(debug_assertions)] {
                assert!(byte.is_ascii(), "`Str::push` got `{byte}`: not ascii")
            }

            match &mut self.0 {
                UnsafeCow::Own(o) => o.push(byte),
                UnsafeCow::Ref(r) => {
                    let mut new = Vec::from(unsafe {r.as_ref()});
                    new.push(byte);
                    self.0 = UnsafeCow::Own(new)
                }
            }
        }

        #[inline]
        pub(crate) fn extend(&mut self, other: Self) {
            match &mut self.0 {
                UnsafeCow::Own(o) => o.extend_from_slice(other.as_bytes()),
                UnsafeCow::Ref(r) => {
                    let mut new = Vec::from(unsafe {r.as_ref()});
                    new.extend_from_slice(other.as_bytes());
                    self.0 = UnsafeCow::Own(new)
                }
            }
        }
    }

    unsafe impl Send for Str {}
    unsafe impl Sync for Str {}

    impl std::ops::Deref for Str {
        type Target = str;

        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            self.as_str()
        }
    }

    impl PartialEq for Str {
        fn eq(&self, other: &Self) -> bool {
            &**self == &**other
        }
    }
    impl PartialEq<str> for Str {
        #[inline(always)]
        fn eq(&self, other: &str) -> bool {
            &**self == other
        }
    }
    impl PartialEq<&str> for Str {
        #[inline]
        fn eq(&self, other: &&str) -> bool {
            &**self == *other
        }
    }

    impl std::fmt::Debug for Str {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&**self)
        }
    }

    impl From<&'static str> for Str {
        #[inline]
        fn from(s: &'static str) -> Self {
            Str(UnsafeCow::Ref(s.as_bytes().into()))
        }
    }
    impl From<String> for Str {
        #[inline]
        fn from(s: String) -> Self {
            Str(UnsafeCow::Own(s.into_bytes()))
        }
    }
    impl From<std::borrow::Cow<'static, str>> for Str {
        fn from(cow: std::borrow::Cow<'static, str>) -> Self {
            match cow {
                std::borrow::Cow::Borrowed(s) => Str::from(s),
                std::borrow::Cow::Owned(s) => Str::from(s)
            }
        }
    }
};

pub trait IntoStr: Sized {
    /// SAFETY: `self` is owned type or reference valid whenever the `Str` can be accessed
    unsafe fn into_str(self) -> Str;
}
const _: () = {
    impl IntoStr for &str {
        #[inline]
        unsafe fn into_str(self) -> Str {
            Str::unchecked_ref(self)
        }
    }

    impl IntoStr for String {
        #[inline]
        unsafe fn into_str(self) -> Str {
            Str::from(self)
        }
    }

    impl IntoStr for std::borrow::Cow<'_, str> {
        unsafe fn into_str(self) -> Str {
            match self {
                std::borrow::Cow::Borrowed(b) => Str::unchecked_ref(b),
                std::borrow::Cow::Owned(o) => Str::from(o)
            }
        }
    }
};
