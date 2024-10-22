<!--

<div align="center">
    <h1>whttp</h1>
    A new, opinionated implementation of HTTP types
</div>

<br>

* _efficient_
    * minimum memory copy & allocation in request parsing
    * pre-calculated fxhash for headers

* _battery included_
    * Server-Sent Events : `sse` feature
    * WebSocket : `ws` with `rt_*` feature
    * HTTP/1.1 parsing & writing : `http1` with `rt_*` feature
    * supported runtimes ( `rt_*` ) : `tokio`, `async-std`, `smol`, `glommio`

-->
