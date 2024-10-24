use whttp::{Request, Response, http1};
use whttp::header::{ContentType, Date};
use whttp::util::IMFfixdate;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind("localhost:3000").await?;

    println!("Started serving at localhost:3000");
    println!("Try\n\
        $ curl -v http://localhost:3000\n\
        $ curl -v http://localhost:3000 http://localhost:3000\n\
        $ curl -v http://localhost:3000 -H 'Content-Type: text/plain' -d '{{YOUR NAME}}'\n\
    ");

    while let Ok((mut conn, addr)) = listener.accept().await {
        println!("accepcted {addr}\n");

        let mut req = http1::init();
        let mut req = std::pin::Pin::new(&mut req);

        while let Ok(Some(())) = http1::load(
            req.as_mut(), &mut conn
        ).await {
            println!("req = {req:?}");

            let res = handle(&req).await;
            println!("res = {res:?}");

            http1::send(res, &mut conn).await?;
            println!();
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
