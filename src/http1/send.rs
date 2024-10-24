use crate::{Response, Status, response::Body, io::Write};
use crate::header::{ContentLength, SetCookie};

pub enum Upgrade {
    None,

    #[cfg(feature="ws")]
    WebSocket(mews::WebSocket)
}
const _: () = {
    impl std::fmt::Debug for Upgrade {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::None => f.write_str("{no upgrade}"),

                #[cfg(feature="ws")]
                Self::WebSocket(_) => f.write_str("{upgrade to WebSocket}"),
            }
        }
    }
    impl PartialEq for Upgrade {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Upgrade::None, Upgrade::None) => true,

                #[cfg(feature="ws")]
                (Upgrade::WebSocket(_), Upgrade::WebSocket(_)) => true,

                #[allow(unreachable_patterns)]
                _ => false
            }
        }
    }
};

#[cfg_attr(not(feature="sse"), inline)]
pub async fn send(
    mut res: Response,
    conn: &mut (impl Write + Unpin)
) -> Result<Upgrade, std::io::Error> {
    if res.header(ContentLength).is_none()
    && res.body().is_none()
    && res.status() != Status::NoContent {
        res.set(ContentLength, "0");
    }

    let mut buf = [
        b"HTTP/1.1 ", res.status().message().as_bytes(), b"\r\n"
    ].concat();
    if let Some(set_cookie) = res.take(SetCookie) {
        for set_cookie in set_cookie.split(',') {
            buf.extend_from_slice(b"Set-Cookie: ");
            buf.extend_from_slice(set_cookie.as_bytes());
            buf.push(b'\r'); buf.push(b'\n');
        }
    }
    for (h, v) in res.headers().iter() {
        buf.extend_from_slice(h.as_bytes());
        buf.push(b':'); buf.push(b' ');
        buf.extend_from_slice(v.as_bytes());
        buf.push(b'\r'); buf.push(b'\n');
    }; buf.push(b'\r'); buf.push(b'\n');

    match res.take_body() {
        None => {
            conn.write_all(&buf).await?;
            conn.flush().await?;

            Ok(Upgrade::None)
        }

        Some(Body::Payload(payload)) => {
            buf.extend_from_slice(&payload);

            conn.write_all(&buf).await?;
            conn.flush().await?;

            Ok(Upgrade::None)
        }

        #[cfg(feature="sse")]
        Some(Body::Stream(mut stream)) => {
            conn.write_all(&buf).await?;
            conn.flush().await?;

            while let Some(chunk) = next(&mut stream).await {
                let mut message = Vec::with_capacity(
                    /* capacity when `chunk` contains only one line */
                    "data: ".len() + chunk.len() + "\n\n".len()
                );
                for line in chunk.split('\n') {
                    message.extend_from_slice("data: ".as_bytes());
                    message.extend_from_slice(line.as_bytes());
                    message.push(b'\n');
                }; message.push(b'\n');

                let size_hex = hexized_bytes(message.len());
                let size_hex = &size_hex[size_hex.iter().position(|&b| b != b'0').unwrap_or(0)..];

                let mut chunk = Vec::with_capacity(
                    size_hex.len() + "\r\n".len() + message.len() + "\r\n".len()
                );
                chunk.extend_from_slice(size_hex);
                chunk.push(b'\r'); chunk.push(b'\n');
                chunk.extend_from_slice(&message);
                chunk.push(b'\r'); chunk.push(b'\n');

                conn.write_all(&chunk).await?;
                conn.flush().await?;
            }

            Ok(Upgrade::None)
        }

        #[cfg(feature="ws")]
        Some(Body::WebSocket(ws)) => {
            conn.write_all(&buf).await?;
            conn.flush().await?;

            Ok(Upgrade::WebSocket(ws))
        }
    }
}

#[cfg(feature="sse")]
#[inline]
fn hexized_bytes(n: usize) -> [u8; size_of::<usize>() * 2] {
    unsafe {// SAFETY: mapping u8 -> u8 u8
        std::mem::transmute::<_, [u8; size_of::<usize>() * 2]>(
            n.to_be_bytes().map(|byte| [byte>>4, byte&0b1111])
        ).map(|h| h + match h {
            0..=9   => b'0'-0,
            10..=15 => b'a'-10,
            _ => std::hint::unreachable_unchecked()
        })
    }
}

#[cfg(feature="sse")]
#[inline]
fn next<S: futures_core::Stream>(stream: &mut S) -> impl std::future::Future<Output = Option<S::Item>> + '_ {
    struct Next<'stream, S>(&'stream mut S);
    impl<'stream, S: futures_core::Stream> std::future::Future for Next<'stream, S> {
        type Output = Option<S::Item>;

        #[inline]
        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            unsafe {self.map_unchecked_mut(|pin| &mut *pin.0)}.poll_next(cx)
        }
    }

    Next(stream)
}




#[cfg(feature="sse")]
#[cfg(test)]
#[test]
fn test_hexized_bytes() {
    assert_eq!(hexized_bytes(0), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0'
    ]);
    assert_eq!(hexized_bytes(1), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'1'
    ]);
    assert_eq!(hexized_bytes(9), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'9'
    ]);
    assert_eq!(hexized_bytes(10), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'a'
    ]);
    assert_eq!(hexized_bytes(15), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'f'
    ]);
    assert_eq!(hexized_bytes(16), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'1',b'0'
    ]);
    assert_eq!(hexized_bytes(17), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'1',b'1'
    ]);
    assert_eq!(hexized_bytes(31), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'1',b'f'
    ]);
    assert_eq!(hexized_bytes(32), [
        b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'0',b'2',b'0'
    ]);
}

#[cfg(all(feature="DEBUG",feature="rt_tokio"))]
#[cfg(test)]
#[tokio::test]
async fn test_send_response() {
    use crate::header::*;

    #[cfg(feature="sse")]
    use futures_util::{stream, StreamExt};

    #[cfg(feature="ws")]
    use mews::{WebSocketContext, Connection};

    macro_rules! assert_bytes {
        ($left:expr, $right:expr) => {{
            let _: &[u8] = &$left;
            let _: &[u8] = $right;

            if let (Ok(left), Ok(right)) = (
                std::str::from_utf8(&$left),
                std::str::from_utf8($right)
            ) {
                assert_eq!(left, right)
            } else {
                assert_eq!($left, $right)
            }
        }};
    }

    {
        let mut buf = Vec::<u8>::new();
        assert_eq!(send(Response::OK(), &mut buf).await.unwrap(), Upgrade::None);
        assert_bytes!(buf, b"\
            HTTP/1.1 200 OK\r\n\
            \r\n\
        ");
    }
    {
        let mut buf = Vec::<u8>::new();
        assert_eq!(send(
            Response::OK()
            .with_text("Hello, world!"),
        &mut buf).await.unwrap(), Upgrade::None);
        assert_bytes!(buf, b"\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 13\r\n\
            Content-Type: text/plain; charset=UTF-8\r\n\
            \r\n\
            Hello, world!\
        ");
    }
    {
        let mut buf = Vec::<u8>::new();
        assert_eq!(send(
            Response::NotFound()
            .with(Origin, "https://ohkami.rs")
            .with_html("<h1>Not Found</h1><p>no resource was found for your request.</p>"),
        &mut buf).await.unwrap(), Upgrade::None);
        assert_bytes!(buf, b"\
            HTTP/1.1 404 Not Found\r\n\
            Content-Length: 64\r\n\
            Content-Type: text/html; charset=UTF-8\r\n\
            Origin: https://ohkami.rs\r\n\
            \r\n\
            <h1>Not Found</h1><p>no resource was found for your request.</p>\
        ");
    }
    {
        let mut buf = Vec::<u8>::new();
        assert_eq!(send(
            Response::NotFound()
            .with(Date, "Sun, 06 Nov 1994 08:49:37 GMT")
            .with_html("<h1>Not Found</h1><p>no resource was found for your request.</p>"),
        &mut buf).await.unwrap(), Upgrade::None);
        assert_bytes!(buf, b"\
            HTTP/1.1 404 Not Found\r\n\
            Content-Length: 64\r\n\
            Content-Type: text/html; charset=UTF-8\r\n\
            Date: Sun, 06 Nov 1994 08:49:37 GMT\r\n\
            \r\n\
            <h1>Not Found</h1><p>no resource was found for your request.</p>\
        ");
    }
    #[cfg(feature="sse")] {
        let stream = stream::repeat("Hello!".to_string()).take(3);

        let mut buf = Vec::<u8>::new();
        assert_eq!(send(
            Response::OK()
            .with(Date, "Sun, 06 Nov 1994 08:49:37 GMT")
            .with_stream(stream),
        &mut buf).await.unwrap(), Upgrade::None);
        assert_bytes!(buf, b"\
            HTTP/1.1 200 OK\r\n\
            Cache-Control: no-cache, must-revalidate\r\n\
            Transfer-Encoding: chunked\r\n\
            Content-Type: text/event-stream\r\n\
            Date: Sun, 06 Nov 1994 08:49:37 GMT\r\n\
            \r\n\
            e\r\n\
            data: Hello!\n\n\r\n\
            e\r\n\
            data: Hello!\n\n\r\n\
            e\r\n\
            data: Hello!\n\n\r\n\
        ");
    }
    #[cfg(feature="ws")] {
        let websocket = || WebSocketContext::new("a-sec-websocket-key").on_upgrade(|_: Connection| async {});

        let mut buf = Vec::<u8>::new();
        assert_eq!(send(
            Response::OK()
            .with(Date, "Sun, 06 Nov 1994 08:49:37 GMT")
            .with_websocket(websocket().0, websocket().1),
        &mut buf).await.unwrap(), Upgrade::WebSocket(websocket().1));
        assert_bytes!(buf, format!("\
            HTTP/1.1 101 Switching Protocols\r\n\
            Connection: Upgrade\r\n\
            Upgrade: websocket\r\n\
            Sec-WebSocket-Accept: {sign:}\r\n\
            Date: Sun, 06 Nov 1994 08:49:37 GMT\r\n\
            \r\n\
        ", sign=(websocket().0)).as_bytes());
    }
}
