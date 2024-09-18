#[derive(Clone)]
pub(crate) enum Str {
    Ref(std::ptr::NonNull<str>),
    Own(String),
}

unsafe impl Send for Str {}
unsafe impl Sync for Str {}
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