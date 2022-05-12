use aws_sdk_dynamodb;
use lambda_http::{service_fn, Body, Error, IntoResponse, Request, Response};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&shared_config);
    let func = service_fn(|req| handler(req, &client));
    lambda_http::run(func).await?;
    Ok(())
}

async fn handler(
    _event: Request,
    _client: &aws_sdk_dynamodb::Client,
) -> Result<Response<Body>, Error> {
    Ok(format!("profile endpoint called").into_response())
}
