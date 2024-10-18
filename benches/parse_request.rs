#![feature(test)]
extern crate test;

use test::black_box;
// const fn black_box<T>(t: T) -> T {t}

fn incoming_http1_large() -> Vec<u8> {
    black_box(From::<&[u8; 1165]>::from(b"\
        POST /api/v2/status?type=library&lang=rust HTTP/1.1\r\n\
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
        Forwarded: for=192.0.2.60; proto=http; by=203.0.113.43\r\n\
        Upgrade-Insecure-Requests: 1\r\n\
        From: webmaster@example.org\r\n\
        X-MyApp-Data: example-custom-header-value\r\n\
        Some-Custom-Header: strawberry\r\n\
        Cookie: PHPSESSID=298zf09hf012fh2; csrftoken=u32t4o3tb3gg43; _gat=1\r\n\
        Cache-Control: no-cache\r\n\
        Content-Type: application/json\r\n\
        Content-Length: 337\r\n\
        \r\n\
        {
            \"name\": \"whttp\",
            \"author\": \"kanarus\",
            \"owner\": {
                \"type\": \"organization\",
                \"name\": \"ohkami-rs\"
            },
            \"description\": \"a new, opinionated implementation of HTTP types\",
            \"license\": \"MIT\",
            \"category\": \"web-programming\"
        }\
    "))
}

fn incoming_http1_small() -> Vec<u8> {
    black_box(From::<&[u8; 326]>::from(b"\
        GET /api/hello HTTP/1.1\r\n\
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

#[inline(always)]
fn httpcrate_parse_http1(incoming: &[u8]) -> ::http::Request<Vec<u8>> {
    use ::http::{Request, Method, Uri, HeaderName, HeaderValue, header};

    let mut r = ::byte_reader::Reader::new(&incoming);

    let mut req = Request::new(Vec::new());

    *req.method_mut() = Method::from_bytes(r.read_while(|&b| b != b' ')).unwrap();
    r.consume(" ").unwrap();

    *req.uri_mut() = Uri::from_maybe_shared(r.read_while(|&b| b != b' ').to_vec()).unwrap();
    r.consume(" ").unwrap();

    r.consume("HTTP/1.1\r\n").unwrap();

    while r.consume("\r\n").is_none() {
        let key_bytes = r.read_while(|&b| b != b':');
        r.consume(": ").unwrap();
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(key_bytes),
            HeaderValue::from_bytes(r.read_while(|&b| b != b'\r'))
        ) {
            req.headers_mut().append(name, value);
        }
        r.consume("\r\n");
    }

    match req.headers().get(header::CONTENT_LENGTH) {
        None => (),
        Some(v) => {
            let n = v.as_bytes().iter().fold(0, |n, &b| 10*n + (b-b'0') as usize);
            if n != 0 {
                *req.body_mut() = Vec::from(&r.remaining()[..n])
            }
        }
    }

    req
}
#[inline(always)]
fn whttp_parse_http1<'incoming>(req: &mut std::pin::Pin<&mut whttp::Request>, incoming: &'incoming [u8]) -> ::byte_reader::Reader<'incoming> {
    use whttp::{request::parse, header};

    parse::clear(req);
    {
        let size = usize::min(incoming.len(), parse::BUF_SIZE);
        parse::buf(req.as_mut())[..size].copy_from_slice(&incoming[..size]);
    }

    let mut r = ::byte_reader::Reader::new(unsafe {
        let buf = parse::buf(req.as_mut());
        std::slice::from_raw_parts(buf.as_ptr(), buf.len())
    });

    parse::method(req, r.read_while(|&b| b != b' ')).unwrap();
    r.consume(" ").unwrap();

    unsafe {parse::path(req, r.read_while(|&b| b != b' '))}.unwrap();
    r.consume(" ").unwrap();

    r.consume("HTTP/1.1\r\n");

    while r.consume("\r\n").is_none() {
        let name_bytes = r.read_while(|&b| b != b':');
        r.consume(": ").unwrap();
        let value_bytes = r.read_while(|&b| b != b'\r');
        unsafe {parse::header(req, name_bytes, value_bytes)}.unwrap();
        r.consume("\r\n");
    }

    if let Some(n @ 1..) = req
        .header(header::ContentLength)
        .map(|v| v.bytes().fold(0, |n, b| 10*n + (b-b'0') as usize))
    {
        assert_eq!(r.index + n, incoming.len());
        req.set_body(Vec::from(&incoming[r.index..(r.index + n)]));
    }

    r
}

#[bench] fn parse_http1_large_httpcrate(b: &mut test::Bencher) {
    let incoming = incoming_http1_large();
    b.iter(|| -> ::http::Request<Vec<u8>> {httpcrate_parse_http1(&incoming)});
}
#[bench] fn parse_http1_small_httpcrate(b: &mut test::Bencher) {
    let incoming = incoming_http1_small();
    b.iter(|| -> ::http::Request<Vec<u8>> {httpcrate_parse_http1(&incoming)});
}

#[bench] fn parse_http1_large_whttp(b: &mut test::Bencher) {
    let incoming = incoming_http1_large();
    let mut req = whttp::request::parse::new();
    let mut req = std::pin::Pin::new(&mut req);
    b.iter(|| -> ::byte_reader::Reader {whttp_parse_http1(&mut req, &incoming)});
}
#[bench] fn parse_http1_small_whttp(b: &mut test::Bencher) {
    let incoming = incoming_http1_small();
    let mut req = whttp::request::parse::new();
    let mut req = std::pin::Pin::new(&mut req);
    b.iter(|| -> ::byte_reader::Reader {whttp_parse_http1(&mut req, &incoming)});
}
