mod auth;
use aws_sdk_dynamodb;
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&shared_config);
    let func = service_fn(|req| handler(req, &client));
    lambda_http::run(func).await?;
    Ok(())
}

async fn handler(
    event: Request,
    client: &aws_sdk_dynamodb::Client,
) -> Result<Response<Body>, Error> {
    Ok(match event.query_string_parameters().first("type") {
        Some("login") => auth::login(&client, event).await,
        Some("authenticate") => auth::authenticate(&client, event).await,
        _ => Response::builder()
            .status(400)
            .body("type parameter missing".into())
            .expect("failed to render response"),
    })
}
