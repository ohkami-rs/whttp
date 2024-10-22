use super::Response;
use crate::{header, util::IntoStr};
use std::borrow::Cow;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

impl Response {
    pub fn with_setcookie()

    #[inline]
    pub fn set_cookie(
        &mut self,
        name: &str,
        value: &str,
        directives: impl FnOnce(SetCookie) -> SetCookie
    ) -> &mut Self {
        self.set(header::SetCookie, directives(SetCookie::new(name, value)).0);
        self
    }
}

pub struct SetCookie(String);
#[allow(non_snake_case)]
impl SetCookie {
    #[inline]
    fn new(name: &str, value: &str) -> Self {
        /*
            matches!(b, 0..=127)  &&
            !b.is_ascii_control() &&
            !matches!(b, /* separators */
                b'(' | b')' | b'<' | b'>'  | b'@' |
                b',' | b';' | b':' | b'\\' | b'"' |
                b'/' | b'[' | b']' | b'?'  | b'=' |
                b'{' | b'}' | b' ' | b'\t'
            )
        */
        assert!(name.bytes().all(|b| matches!(b,
                       // 0 ..=31 are controls / 32 is ' '
            | 33       // 34 is '"'
            | 35..=39  // 40..=41 are '(' ')'
            | 42..=43  // 44 is ','
            | 45..=46  // 47 is '/'
            | 48..=57  // 58..=64 are ':' ';' '<' '=' '>' '?' '@'
            | 65..=90  // 91..=93 are '[' '\' ']'
            | 94..=122 // 123 is '{'
            | 124      // 125 is '}'
            | 126      // 127 is DEL
        )), "`{name}` can't be Set-Cookie name: it must be ascii and not be controls or separators (https://httpwg.org/specs/rfc6265.html#sane-set-cookie-syntax)");

        let value = Cow::from(percent_encode(value.as_bytes(), NON_ALPHANUMERIC));

        Self([name, "=", &value].concat())
    }

    pub fn Expires(mut self, Expires: impl IntoStr) -> Self {
        self.0.push_str("; Expires=");
        self.0.push_str(&Expires.into_str());
        self
    }
    pub fn MaxAge(mut self, MaxAge: u64) -> Self {
        self.0.push_str("; Max-Age=");
        self.0.push_str(&MaxAge.to_string());
        self
    }
    pub fn Domain(mut self, Domain: impl IntoStr) -> Self {
        self.0.push_str("; Domain=");
        self.0.push_str(&Domain.into_str());
        self
    }
    pub fn Path(mut self, Path: impl IntoStr) -> Self {
        self.0.push_str("; Path=");
        self.0.push_str(&Path.into_str());
        self
    }
    pub fn Secure(mut self) -> Self {
        self.0.push_str("; Secure");
        self
    }
    pub fn HttpOnly(mut self) -> Self {
        self.0.push_str("; HttpOnly");
        self
    }
    pub fn SameSiteStrict(mut self) -> Self {
        self.0.push_str("; SameSite=Strict");
        self
    }
    pub fn SameSiteLax(mut self) -> Self {
        self.0.push_str("; SameSite=Lax");
        self
    }
    pub fn SameSiteNone(mut self) -> Self {
        self.0.push_str("; SameSite=None");
        self
    }
}
