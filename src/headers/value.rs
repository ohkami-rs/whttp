use crate::Str;
use std::ptr::NonNull;

/// HTTP header value.
/// 
/// ## Note
/// RFC allows non UTF-8 bytes for HTTP header, but `whttp` doesn't.
#[derive(Clone)]
pub struct Value(Str);

pub struct InvalidValue;
impl std::error::Error for InvalidValue {}
impl std::fmt::Debug for InvalidValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid header value")
    }
}
impl std::fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid header value")
    }
}

#[inline(always)]
const fn available(byte: &u8) -> bool {
    match byte {
        b'\t' | 32..=126 => true,
        _ => false
    }
}

#[inline(always)]
fn valid(bytes: &[u8]) -> bool {
    for b in bytes {
        if !available(b) {return false}
    }
    true
}

const fn const_valid(bytes: &[u8]) -> bool {
    {
        let mut i = 0;
        while i < bytes.len() {
            if !available(&bytes[i]) {return false}
            i += 1
        }
    }
    true
}

const _/* trait impls */: () = {
    impl std::ops::Deref for Value {
        type Target = str;

        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            match &self.0 {
                // SAFETY: `Value` constructors' SAFETY
                Str::Ref(r) => unsafe {r.as_ref()},
                Str::Own(o) => o.as_str()
            }
        }
    }

    impl PartialEq for Value {
        fn eq(&self, other: &Self) -> bool {
            &**self == &**other
        }
    }
    impl PartialEq<str> for Value {
        #[inline(always)]
        fn eq(&self, other: &str) -> bool {
            &**self == other
        }
    }
    impl PartialEq<&str> for Value {
        #[inline]
        fn eq(&self, other: &&str) -> bool {
            &**self == *other
        }
    }

    impl std::fmt::Debug for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&**self)
        }
    }

    impl From<&'static str> for Value {
        #[inline]
        fn from(s: &'static str) -> Self {
            if !valid(s.as_bytes()) {panic!("invalid header value")}
            Self(Str::Ref(s.into()))
        }
    }
    impl From<String> for Value {
        #[inline]
        fn from(s: String) -> Self {
            if !valid(s.as_bytes()) {panic!("invalid header value")}
            Self(Str::Own(s))
        }
    }
    impl From<std::borrow::Cow<'static, str>> for Value {
        fn from(cow: std::borrow::Cow<'static, str>) -> Self {
            match cow {
                std::borrow::Cow::Borrowed(s) => Value::from(s),
                std::borrow::Cow::Owned(s) => Value::from(s)
            }
        }
    }
};

impl Value {
    pub const fn new(value: &'static str) -> Self {
        if !const_valid(value.as_bytes()) {panic!("invalid header value")}
        // SAFETY: 'static reference is always valid
        Self(Str::Ref(unsafe {NonNull::new_unchecked(value as *const str as *mut str)}))
    }
}

impl Value {
    /// SAFETY: `bytes` is valid reference whenever returned `Value` can be accessed
    #[inline]
    pub(crate) unsafe fn parse(bytes: &[u8]) -> Result<Self, InvalidValue> {
        if valid(bytes) {
            // SAFETY: `valid(bytes)` returned true
            let bytes = unsafe {std::str::from_utf8_unchecked(bytes)};
            // SAFETY: function SAFETY
            Ok(Self(Str::Ref(unsafe {NonNull::new_unchecked(bytes as *const str as *mut str)})))
        } else {
            Err(InvalidValue)
        }
    }

    pub(crate) fn append(&mut self, another: Value) {
        match &mut self.0 {
            Str::Own(s) => {
                s.push(',');
                s.push_str(&another);
            }
            Str::Ref(s) => {
                // SAFETY: `Value` constructors' SAFETY
                let mut s = String::from(unsafe {s.as_ref()});
                s.push(',');
                s.push_str(&another);
                self.0 = Str::Own(s)
            }
        }
    }
}
