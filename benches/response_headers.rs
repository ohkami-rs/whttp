#![feature(test)]
extern crate test;

use test::black_box;
// const fn black_box<T>(t: T) -> T {t}

#[bench] fn intert_http(b: &mut test::Bencher) {
    use ::http::{HeaderMap, HeaderName, HeaderValue, header};
    use std::mem::{MaybeUninit, replace};

    let mut h = MaybeUninit::new(HeaderMap::new());
    b.iter(|| -> HeaderMap {
        let mut h = unsafe {replace(&mut h, MaybeUninit::uninit()).assume_init()};
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
        h.insert(HeaderName::from_static("x-myapp-data"),  black_box(HeaderValue::from_static("excellent")));
        h.insert(HeaderName::from_static("something"),     black_box(HeaderValue::from_static("anything")));
        h
    });
}

#[bench] fn insert_whttp(b: &mut test::Bencher) {
    use ::whttp::{Headers, Header, header};
    use std::mem::{MaybeUninit, replace};

    const X_MYAPP_DATA: &Header = &Header::new("x-myapp-data");
    const SOMETHING:    &Header = &Header::new("something");

    let mut h = MaybeUninit::new(Headers::new());
    b.iter(|| -> Headers {
        let mut h = unsafe {replace(&mut h, MaybeUninit::uninit()).assume_init()};
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
        h
    });
}
