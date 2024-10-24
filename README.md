<div align="center">
    <h1>whttp</h1>
    A new, opinionated implementation of HTTP types for Rust
</div>

<br>

<div align="right">
    <a href="https://github.com/ohkami-rs/whttp/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/ohkami.svg" /></a>
    <a href="https://github.com/ohkami-rs/whttp/actions"><img alt="CI status" src="https://github.com/ohkami-rs/whttp/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/whttp"><img alt="crates.io" src="https://img.shields.io/crates/v/whttp" /></a>
</div>

<br>

## What's advantage over http crate？

### fast, efficient

* swiss table (by hashbrown) and pre-calculated fxhash for `Headers`
* pre-matching standard headers before hashing in parsing
* `Request` construction with zero or least copy from parsing buffer and very minimum allocation
* size of `Request` is *128* and size of `Response` is *64*

### batteries included

* consistent and clear API
* builtin support for Cookie, Set-Cookie, IMF-fixdate header values and JSON response body
* Server-Sent Events on `sse` feature
* WebSocket on `ws` & `rt_*` feature
* HTTP/1.1 parsing & writing on `http1` & `rt_*` feature
* supported runtimes ( `rt_*` ) : `tokio`, `async-std`, `smol`, `glommio`

<br>

## Example


