use super::{Request, Method};
use crate::header::ContentLength;
use crate::headers::{Header, Headers};
use crate::unsafe_cow::Str;
use ::std::borrow::Cow;
use ::percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

pub trait BuildRequest {
    fn build(self, req: Request) -> Request;
}
const _: () = {
    impl BuildRequest for () {
        fn build(self, req: Request) -> Request {
            req
        }
    }

    impl<F: FnOnce(RequestBuilder)->RequestBuilder> BuildRequest for F {
        fn build(self, req: Request) -> Request {
            self(RequestBuilder(req)).0
        }
    }
};

pub struct RequestBuilder(Request);

macro_rules! builder {
    ($( $method:ident )*) => {
        #[cfg(test)]
        fn __assert_exhausted__(m: Method) {
            match m {$(Method::$method => (),)*}
        }

        impl Request {$(
            #[allow(non_snake_case)]
            pub fn $method(path: impl Into<Cow<'static, str>>, builder: impl BuildRequest) -> Self {
                builder.build(Request {
                    __buf__: None,
                    method:  Method::$method,
                    path:    Str::from(path.into()),
                    query:   None,
                    headers: Headers::new(),
                    body:    None,
                })
            }
        )*}
    };
}
builder! { GET PUT POST PATCH DELETE HEAD OPTIONS CONNECT }

impl RequestBuilder {
    pub fn header(mut self, header: Header, value: impl Into<Cow<'static, str>>) -> Self {
        self.0.headers.insert(header, Into::<Cow<'static, str>>::into(value));
        self
    }

    pub fn query(mut self, key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        let value: Cow<'static, str> = value.into();
        let query: String = {
            let key = utf8_percent_encode(key, NON_ALPHANUMERIC);
            let value = utf8_percent_encode(&value, NON_ALPHANUMERIC);
            let mut query = String::with_capacity(key.size_hint().0 + 1 + value.size_hint().0);
            query.push_str(&Cow::<str>::from(key));
            unsafe {query.as_mut_vec().push(b'=');}
            query.push_str(&Cow::<str>::from(value));
            query
        };
        match &mut self.0.query {
            None => unsafe {self.0.set_query(query)}
            Some(existing) => {
                existing.push(b'&');
                existing.extend(Str::from(query));
            }
        }
        self
    }

    pub fn body(mut self, body: impl Into<Cow<'static, [u8]>>) -> Self {
        let body: Cow<'static, [u8]> = body.into();
        self.0.set(ContentLength, body.len());
        unsafe {self.0.set_body(body)}
        self
    }
}
