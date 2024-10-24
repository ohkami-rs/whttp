use crate::{headers::Value, util::IntoStr};
use std::borrow::Cow;
use percent_encoding::{utf8_percent_encode, percent_decode_str, NON_ALPHANUMERIC};

/*=====================================================*/

/// `Cookie` helper
/// 
/// *example.rs*
/// ```
/// # use whttp::{Request, Response, util::cookie};
/// # use whttp::header::SetCookie;
/// # 
/// fn get_cookies(req: &Request) ->
///     Option<impl Iterator<Item = cookie<'_>>>
/// {
///     req.cookies()
/// }
/// 
/// fn add_cookie(res: &mut Response) {
///     res.append(SetCookie,
///         cookie::set("token", "abcxyz")
///             .Secure()
///             .HttpOnly()
///     );
/// }
/// ```
#[allow(non_camel_case_types)]
pub struct cookie<'req> {
    name:  &'req str,
    value: &'req str
}

impl<'req> cookie<'req> {
    pub(crate) fn parse(cookies: &'req str) -> impl Iterator<Item = Self> {
        cookies.split("; ").flat_map(|cookie|
            cookie.split_once('=').map(|(name, value)|
                cookie { name, value }))
    }
}

impl<'req> cookie<'req> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn value(&self) -> Cow<'_, str> {
        percent_decode_str(self.value).decode_utf8()
            .map_err(|_| self.value).expect("non UTF-8 Cookie value")
    }
    pub fn value_bytes(&self) -> Cow<'_, [u8]> {
        percent_decode_str(self.value).into()
    }
}

/*=====================================================*/

impl cookie<'static> {
    pub fn set(name: &str, value: &str) -> setcookie {
        setcookie::new(name, value)
    }

    pub fn set_encoded(name: &str, value: &str) -> setcookie {
        setcookie::encoded(name, value)
    }
}

/// `Set-Cookie` helper
/// 
/// *example.rs*
/// ```
/// # use whttp::{Headers, util::cookie};
/// # use whttp::header::SetCookie;
/// # 
/// fn add_cookie(headers: &mut Headers) {
///     headers
///         .append(SetCookie,
///             cookie::set("token", "abcxyz")
///             .Secure()
///             .HttpOnly()
///         );
/// }
/// ```
#[allow(non_camel_case_types)]
pub struct setcookie(String);

const _: () = {
    impl Into<Value> for setcookie {
        #[inline]
        fn into(self) -> Value {
            Value::from(self.0)
        }
    }

    impl std::ops::Deref for setcookie {
        type Target = str;
        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
};

fn valid_name(name: &str) -> bool {
    name.bytes().all(|b| matches!(b,
                   // 0 ..=31 are controls
                   // 32 is ' '
        | 33       // 34 is '"'
        | 35..=39  // 40..=41 are '(' ')'
        | 42..=43  // 44 is ','
        | 45..=46  // 47 is '/'
        | 48..=57  // 58..=64 are ':' ';' '<' '=' '>' '?' '@'
        | 65..=90  // 91..=93 are '[' '\' ']'
        | 94..=122 // 123 is '{'
        | 124      // 125 is '}'
        | 126      // 127 is DEL (control)
    ))
}
fn valid_value(value: &str) -> bool {
    value.bytes().all(|b| b.is_ascii() && !!!(
        b.is_ascii_control() ||
        b.is_ascii_whitespace() ||
        matches!(b, b'"' | b',' | b';' | b'\\')
    ))
}
fn quoted_content(value: &str) -> Option<&str> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Some(&value[1..value.len()-1])
    } else {
        None
    }
}

impl setcookie {
    pub fn new(name: &str, value: &str) -> Self {
        assert!(valid_name(name), "\
            `{name}` can't be a Set-Cookie name: it must be ascii and not be controls, spaces or separators\
            (https://httpwg.org/specs/rfc6265.html#sane-set-setcookie-syntax) \
        ");

        let (value, quoted) = match quoted_content(value) {
            Some(q) => (q, true),
            None    => (value, false)
        };
        assert!(valid_value(value), "\
            `{value}` can't be a Set-Cookie value: it must be ascii and not be controls, whitespaces or `\"` `,` `;` `\\` \
            (https://httpwg.org/specs/rfc6265.html#sane-set-setcookie-syntax) \
        ");

        Self(if quoted {
            [name, "=\"", value, "\""].concat()
        } else {
            [name, "=", value].concat()
        })
    }

    pub fn encoded(name: &str, value: &str) -> Self {
        let value = utf8_percent_encode(
            quoted_content(value).unwrap_or(value),
            NON_ALPHANUMERIC
        );
        Self::new(name, &Cow::from(value))
    }
}

#[allow(non_snake_case)]
impl setcookie {
    pub fn Expires(mut self, Expires: super::IMFfixdate) -> Self {
        self.0.push_str("; Expires=");
        self.0.push_str(&Expires);
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
