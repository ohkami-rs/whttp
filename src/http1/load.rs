use crate::{Request, Status, request::parse, io::Read};
use crate::header::ContentLength;
use std::{pin::Pin, io::ErrorKind, str::FromStr as _};

const PAYLOAD_LIMIT: usize = 1 << 32;

pub async fn load(
    mut req: Pin<&mut Request>,
    conn: &mut (impl Read + Unpin)
) -> Result<Option<()>, Status> {
    let buf = parse::buf(req.as_mut());

    match conn.read(&mut **buf).await {
        Err(e) => return match e.kind() {
            ErrorKind::ConnectionReset => Ok(None),
            _ => Err((|_err| {
                #[cfg(feature="DEBUG")] {eprintln!("failed to load Request: {_err}")}
                Status::InternalServerError
            })(e))
        },
        Ok(0) => return Ok(None),
        _ => ()
    }

    let mut r = byte_reader::Reader::new(unsafe {
        // lifetime trick.
        // SAFETY: `req.buf` is immutable after `parse::buf`
        std::mem::transmute(buf.as_slice())
    });

    /* from here, SAFETY of `parse::*`: just self-referencing bytes of `req.buf` */

    unsafe {parse::method(&mut req, r.read_while(|&b| b != b' '))}?;

    r.next_if(|&b| b == b' ').ok_or(Status::BadRequest)?;

    unsafe {parse::path(&mut req, r.read_while(|&b| !matches!(b, b' '|b'?')))}?;

    if r.next_if(|&b| b == b'?').is_some() {
        unsafe {parse::query(&mut req, r.read_while(|&b| b != b' '))}?;
    }

    r.next_if(|&b| b == b' ').ok_or(Status::BadRequest)?;

    r.consume("HTTP/1.1\r\n").ok_or(Status::HTTPVersionNotSupported)?;

    while r.consume("\r\n").is_none() {
        let name = r.read_while(|&b| b != b':');
        r.consume(": ").ok_or(Status::BadRequest)?;
        let value = r.read_while(|&b| b != b'\r');
        r.consume("\r\n").ok_or(Status::BadRequest)?;
        unsafe {parse::header(&mut req, name, value)}?;
    }

    match req.header(ContentLength).map(usize::from_str).transpose().map_err(|_| Status::BadRequest)? {
        None | Some(0) => (),
        Some(PAYLOAD_LIMIT..) => return Err(Status::PayloadTooLarge),
        Some(n) => load_body(req, conn, r.remaining(), n).await?
    }

    Ok(Some(()))
}

#[inline]
async fn load_body(
    mut req:        Pin<&mut Request>,
    conn:           &mut (impl Read + Unpin),
    remaining_buf:  &[u8],
    content_length: usize,
) -> Result<(), Status> {
    let remaining_buf_len = remaining_buf.len();

    if remaining_buf_len == 0 || remaining_buf[0] == 0 {
        #[cfg(feature="DEBUG")] {println!("\n[load_body] case: remaining_buf.is_empty() || remaining_buf[0] == 0\n")}

        let mut body = vec![0; content_length];
        conn.read_exact(&mut body).await.map_err(|_| Status::InternalServerError)?;
        parse::body_own(&mut req, body);

    } else if content_length <= remaining_buf_len {
        #[cfg(feature="DEBUG")] {println!("\n[load_body] case: starts_at + size <= BUF_SIZE\n")}

        let body = unsafe {remaining_buf.get_unchecked(..content_length)};
        unsafe {parse::body_ref(&mut req, body)}

    } else {
        #[cfg(feature="DEBUG")] {println!("\n[load_body] case: else\n")}

        let mut body = vec![0; content_length];
        unsafe {body.get_unchecked_mut(..remaining_buf_len)}.copy_from_slice(remaining_buf);
        conn.read_exact(unsafe {body.get_unchecked_mut(remaining_buf_len..)}).await.map_err(|_| Status::InternalServerError)?;
        parse::body_own(&mut req, body);
    }

    Ok(())
}




#[cfg(all(feature="DEBUG",feature="rt_tokio"))]
#[cfg(test)]
#[tokio::test]
async fn test_load_request() {
    use crate::header::*;

    let mut req = parse::new();
    let mut req = Pin::new(&mut req);

    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Ok(None));
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            GET /HTTP/2\r\n\
            \r\n\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Err(Status::BadRequest));
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            GET / HTTP/2\r\n\
            \r\n\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Err(Status::HTTPVersionNotSupported));
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            GET / HTTP/1.1\r\n\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Err(Status::BadRequest));
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            GET / HTTP/1.1\r\n\
            \r\n\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Ok(Some(())));
        assert_eq!(*req, Request::GET("/"));
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            GET / HTTP/1.1\r\n\
            Host: http://127.0.0.1:3000\r\n\
            \r\n\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Ok(Some(())));
        assert_eq!(*req, Request::GET("/").with(Host, "http://127.0.0.1:3000"));
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            POST /api/users HTTP/1.1\r\n\
            Host: http://127.0.0.1:3000\r\n\
            Content-Type: application/json\r\n\
            Content-Length: 24\r\n\
            \r\n\
            {\"name\":\"whttp\",\"age\":0}\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Ok(Some(())));
        assert_eq!(*req,
            Request::POST("/api/users")
            .with(Host, "http://127.0.0.1:3000")
            .with_body("application/json", "{\"name\":\"whttp\",\"age\":0}")
        );
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            POST /api/users HTTP/1.1\r\n\
            Host: http://127.0.0.1:3000\r\n\
            Content-Type: application/json\r\n\
            Content-Length: 22\r\n\
            \r\n\
            {\"name\":\"whttp\",\"age\":0}\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Ok(Some(())));
        assert_eq!(*req,
            Request::POST("/api/users")
            .with(Host, "http://127.0.0.1:3000")
            .with_body("application/json", "{\"name\":\"whttp\",\"age\":")
        );
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            POST /api/users HTTP/1.1\r\n\
            host: http://127.0.0.1:3000\r\n\
            content-type: application/json\r\n\
            content-length: 24\r\n\
            \r\n\
            {\"name\":\"whttp\",\"age\":0}\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Ok(Some(())));
        assert_eq!(*req,
            Request::POST("/api/users")
            .with(Host, "http://127.0.0.1:3000")
            .with_body("application/json", "{\"name\":\"whttp\",\"age\":0}")
        );
    }
    {
        let mut case: &[u8] = {parse::clear(&mut req); b"\
            POST /api/users HTTP/1.1\r\n\
            host: http://127.0.0.1:3000\r\n\
            content-type: application/json\r\n\
            content-Length: 24\r\n\
            \r\n\
            {\"name\":\"whttp\",\"age\":0}\
        "};
        assert_eq!(load(req.as_mut(), &mut case).await, Ok(Some(())));
        assert_eq!(*req,
            Request::POST("/api/users")
            .with(Host, "http://127.0.0.1:3000")
            .with_body("application/json", "{\"name\":\"whttp\",\"age\":0}")
        );
    }
}
