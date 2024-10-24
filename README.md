<div align="center">
    <h1>whttp</h1>
    A new, opinionated implementation of HTTP types for Rust
</div>

<br>

<div align="right">
    <a href="https://github.com/ohkami-rs/whttp/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/whttp.svg" /></a>
    <a href="https://github.com/ohkami-rs/whttp/actions"><img alt="CI status" src="https://github.com/ohkami-rs/whttp/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/whttp"><img alt="crates.io" src="https://img.shields.io/crates/v/whttp" /></a>
</div>

## What's advantage over http crate？

### fast, efficient

* swiss table (by hashbrown) and pre-calculated fxhash for `Headers`
* pre-matching standard headers before hashing in parsing
* `Request` construction with zero or least copy from parsing buffer and very minimum allocation
* size of `Request` is *128* and size of `Response` is *64*
* [micro benchmarks](https://github.com/ohkami-rs/whttp/blob/main/benches)

### batteries included

* consistent and clear API
* builtin support for Cookie, Set-Cookie, IMF-fixdate header values and JSON response body
* Server-Sent Events on `sse` feature
* WebSocket on `ws` & `rt_*` feature
* HTTP/1.1 parsing & writing on `http1` & `rt_*` feature
* supported runtimes ( `rt_*` ) : `tokio`, `async-std`, `smol`, `glommio`

## [Example](https://github.com/ohkami-rs/whttp/blob/main/example)

```toml
[dependencies]
whttp = { version = "0.1", features = ["http1", "rt_tokio"] }
tokio = { version = "1", features = ["full"] }
```
```rust
use whttp::{Request, Response, http1};
use whttp::header::{ContentType, Date};
use whttp::util::IMFfixdate;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind("localhost:3000").await?;

    while let Ok((mut conn, addr)) = listener.accept().await {
        let mut req = http1::init();
        let mut req = std::pin::Pin::new(&mut req);

        while let Ok(Some(())) = http1::load(
            req.as_mut(), &mut conn
        ).await {
            let res = handle(&req).await;
            http1::send(res, &mut conn).await?;
        }
    }

    Ok(())
}

async fn handle(req: &Request) -> Response {
    if !(req.header(ContentType)
            .is_some_and(|ct| ct.starts_with("text/plain"))
    ) {
        return Response::BadRequest()
            .with(Date, IMFfixdate::now())
            .with_text("expected text payload")
    }

    let name = std::str::from_utf8(req.body().unwrap()).unwrap();

    Response::OK()
        .with(Date, IMFfixdate::now())
        .with_text(format!("Hello, {name}!"))
}
```

## LICENSE

whttp is licensed under MIT LICENSE ( [LICENSE](https://github.com/ohkami-rs/whttp/blob/main/LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT) ).
