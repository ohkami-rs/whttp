#![feature(test)]
extern crate test;

use test::black_box;
// const fn black_box<T>(t: T) -> T {t}

mod __http__ {
    use super::*;
    pub use ::http::*;

    pub const X_MYAPP_DATA: HeaderName = HeaderName::from_static("x-myapp-data");
    pub const SOMETHING:    HeaderName = HeaderName::from_static("something");

    #[inline(always)]
    pub fn init(h: &mut HeaderMap) {
        h.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, black_box(HeaderValue::from_static("true")));
        h.insert(header::ACCESS_CONTROL_ALLOW_HEADERS,     black_box(HeaderValue::from_static("X-Custom-Header,Upgrade-Insecure-Requests")));
        h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN,      black_box(HeaderValue::from_static("https://foo.bar.org")));
        h.insert(header::ACCESS_CONTROL_ALLOW_METHODS,     black_box(HeaderValue::from_static("POST,GET,OPTIONS,DELETE")));
        h.insert(header::ACCESS_CONTROL_MAX_AGE,           black_box(HeaderValue::from_static("86400")));
        h.insert(header::VARY,                             black_box(HeaderValue::from_static("Origin")));
        h.insert(header::CONNECTION,                       black_box(HeaderValue::from_static("Keep-Alive")));
        h.insert(header::DATE,                             black_box(HeaderValue::from_static("Wed, 21 Oct 2015 07:28:00 GMT")));
        h.insert(header::REFERRER_POLICY,                  black_box(HeaderValue::from_static("same-origin")));
        h.insert(header::X_FRAME_OPTIONS,                  black_box(HeaderValue::from_static("DENY")));
        h.insert(X_MYAPP_DATA,                             black_box(HeaderValue::from_static("excellent")));
        h.insert(SOMETHING,                                black_box(HeaderValue::from_static("anything")));
    }

    #[inline(always)]
    pub fn initialized() -> HeaderMap {
        let mut h = HeaderMap::new();
        init(&mut h);
        h
    }
}

mod __whttp__ {
    use super::*;
    pub use ::whttp::*;

    pub const X_MYAPP_DATA: &Header = &Header::def("x-myapp-data");
    pub const SOMETHING:    &Header = &Header::def("something");

    #[inline(always)]
    pub fn init(h: &mut Headers) {
        h
            .set(header::AccessControlAllowCredentials, black_box("true"))
            .set(header::AccessControlAllowHeaders,     black_box("X-Custom-Header,Upgrade-Insecure-Requests"))
            .set(header::AccessControlAllowOrigin,      black_box("https://foo.bar.org"))
            .set(header::AccessControlAllowMethods,     black_box("POST,GET,OPTIONS,DELETE"))
            .set(header::AccessControlMaxAge,           black_box("86400"))
            .set(header::Vary,                          black_box("Origin"))
            .set(header::Connection,                    black_box("Keep-Alive"))
            .set(header::Date,                          black_box("Wed, 21 Oct 2015 07:28:00 GMT"))
            .set(header::ReferrerPolicy,                black_box("same-origin"))
            .set(header::XFrameOptions,                 black_box("DENY"))
            .set(X_MYAPP_DATA,                          black_box("excellent"))
            .set(SOMETHING,                             black_box("anything"))
        ;
    }

    #[inline(always)]
    pub fn initialized() -> Headers {
        let mut h = Headers::new();
        init(&mut h);
        h
    }
}

#[bench] fn new_http(b: &mut test::Bencher) {
    use __http__::*;

    b.iter(|| -> HeaderMap {
        HeaderMap::new()
    });
}
#[bench] fn new_whttp(b: &mut test::Bencher) {
    use __whttp__::*;

    b.iter(|| -> Headers {
        Headers::new()
    });
}

#[bench] fn initialize_http(b: &mut test::Bencher) {
    use __http__::*;

    b.iter(|| -> HeaderMap {
        initialized()
    });
}
#[bench] fn initialize_whttp(b: &mut test::Bencher) {
    use __whttp__::*;

    b.iter(|| -> Headers {
        initialized()
    });
}

#[bench] fn insert_http(b: &mut test::Bencher) {
    use __http__::*;

    let mut h = HeaderMap::new();
    b.iter(|| {
        h.clear();
        init(&mut h);
    });
}
#[bench] fn insert_whttp(b: &mut test::Bencher) {
    use __whttp__::*;

    let mut h = Headers::new();
    b.iter(|| {
        h.clear();
        init(&mut h);
    });
}

#[bench] fn remove_http(b: &mut test::Bencher) {
    use __http__::*;

    let mut h = initialized();
    b.iter(|| {
        h.remove(header::ACCESS_CONTROL_ALLOW_CREDENTIALS);
        h.remove(header::ACCESS_CONTROL_ALLOW_HEADERS);
        h.remove(header::ACCESS_CONTROL_ALLOW_METHODS);
        h.remove(header::ACCESS_CONTROL_ALLOW_ORIGIN);
        h.remove(header::ACCESS_CONTROL_MAX_AGE);
        h.remove(X_MYAPP_DATA);
    });
}
#[bench] fn remove_whttp(b: &mut test::Bencher) {
    use __whttp__::*;

    let mut h = initialized();
    b.iter(|| {
        h.remove(header::AccessControlAllowCredentials);
        h.remove(header::AccessControlAllowHeaders);
        h.remove(header::AccessControlAllowMethods);
        h.remove(header::AccessControlAllowOrigin);
        h.remove(header::AccessControlMaxAge);
        h.remove(X_MYAPP_DATA);
    });
}
