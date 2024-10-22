use crate::{Response, response::Body, io::Write};

pub enum Upgrade {
    None,

    #[cfg(feature="ws")]
    WebSocket(mews::WebSocket)
}

#[cfg_attr(not(feature="sse"), inline)]
pub async fn send(
    mut res: Response,
    conn: &mut (impl Write + Unpin)
) -> Result<Upgrade, std::io::Error> {
    let mut buf = [b"HTTP/1.1 ", res.status().message().as_bytes()].concat();
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

            while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
                let mut message = Vec::with_capacity(
                    /* capacity when `chunk` contains only one line */
                    "data: ".len() + chunk.len() + "\n\n".len()
                );
                for line in chunk.split('\n') {
                    message.extend_from_slice(b"data: ");
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




#[cfg(feature="DEBUG")]
#[cfg(test)]
#[tokio::test]
async fn test_send_response() {
    {
        let mut buf = Vec::<u8>::new();
        
    }
}
