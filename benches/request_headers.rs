#![feature(test)]
extern crate test;

use test::black_box;
// const fn black_box<T>(t: T) -> T {t}

fn incoming_http1_large() -> Vec<u8> {
    black_box(From::<&[u8; 819]>::from(b"\
        Accept-Language: fr-CH, fr;q=0.9, en;q=0.8, de;q=0.7, *;q=0.5\r\n\
        Authorization: Bearer dummy-authorization-token-sample\r\n\
        Date: Wed, 21 Oct 2015 07:28:00 GMT\r\n\
        Host: localhost:7777\r\n\
        Origin: localhost:3333\r\n\
        Referer: https://developer.mozilla.org/ja/docs/Web/JavaScript\r\n\
        Referrer-Policy: no-referrer\r\n\
        Via: HTTP/1.1 GWA\r\n\
        User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion\r\n\
        Transfer-Encoding: identity\r\n\
        Connection: upgrade\r\n\
        Upgrade: a_protocol/1, example ,another_protocol/2.2\r\n\
        Forwarded: for=192.0.2.60; proto=http; by=203.0.113.43\r\n\
        Upgrade-Insecure-Requests: 1\r\n\
        From: webmaster@example.org\r\n\
        X-MyApp-Data: example-custom-header-value\r\n\
        Some-Custom-Header: strawberry\r\n\
        Expect: 100-continue\r\n\
        Cookie: PHPSESSID=298zf09hf012fh2; csrftoken=u32t4o3tb3gg43; _gat=1\r\n\
        Cache-Control: no-cache\r\n\
        \r\n\
    "))
}

fn incoming_http1_small() -> Vec<u8> {
    black_box(From::<&[u8; 301]>::from(b"\
        Authorization: Bearer dummy-authorization-token-sample\r\n\
        Host: localhost:7777\r\n\
        Origin: localhost:3333\r\n\
        User-Agent: Mozilla/5.0 (platform; rv:geckoversion) Gecko/geckotrail Firefox/firefoxversion\r\n\
        From: webmaster@example.org\r\n\
        X-MyApp-Data: example-custom-header-value\r\n\
        Some-Custom-Header: strawberry\r\n\
        \r\n\
    "))
}

#[bench] fn parse_http1_headers_large_httpcrate(b: &mut test::Bencher) {
    use ::http::{HeaderMap, HeaderName, HeaderValue};

    let incoming = incoming_http1_large();

    b.iter(|| -> HeaderMap {
        let mut h = HeaderMap::new();
        let mut r = ::byte_reader::Reader::new(&incoming);
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|&b| b != b':');
            r.consume(": ").unwrap();
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(key_bytes),
                HeaderValue::from_bytes(r.read_while(|&b| b != b'\r'))
            ) {
                h.append(name, value);
            }
            r.consume("\r\n");
        }
        h
    });
}

#[bench] fn parse_http1_headers_large_whttp(b: &mut test::Bencher) {
    use whttp::{Headers, Header, Value};

    let incoming = incoming_http1_large();

    b.iter(|| -> Headers {
        let mut h = Headers::new();
        let mut r = ::byte_reader::Reader::new(&incoming);
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|&b| b != b':');
            r.consume(": ").unwrap();
            if let (Ok(name), Ok(value)) = (
                unsafe {Header::parse_mainly_standard(key_bytes)},
                unsafe {Value::parse(r.read_while(|&b| b != b'\r'))}
            ) {
                h.append(&name, value);
            }
            r.consume("\r\n");
        }
        h
    });
}

#[bench] fn parse_http1_headers_small_httpcrate(b: &mut test::Bencher) {
    use ::http::{HeaderMap, HeaderName, HeaderValue};

    let incoming = incoming_http1_small();

    b.iter(|| -> HeaderMap {
        let mut h = HeaderMap::new();
        let mut r = ::byte_reader::Reader::new(&incoming);
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_until(":");
            r.consume(": ").unwrap();
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(key_bytes),
                HeaderValue::from_bytes(r.read_until("\r\n"))
            ) {
                h.append(name, value);
            }
            r.consume("\r\n");
        }
        h
    });
}

#[bench] fn parse_http1_headers_small_whttp(b: &mut test::Bencher) {
    use whttp::{Headers, Header, Value};

    let incoming = incoming_http1_small();

    b.iter(|| -> Headers {
        let mut h = Headers::new();
        let mut r = ::byte_reader::Reader::new(&incoming);
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_until(":");
            r.consume(": ").unwrap();
            if let (Ok(name), Ok(value)) = (
                unsafe {Header::parse_mainly_standard(key_bytes)},
                unsafe {Value::parse(r.read_until("\r\n"))}
            ) {
                h.push(name, value);
            }
            r.consume("\r\n");
        }
        h
    });
}
