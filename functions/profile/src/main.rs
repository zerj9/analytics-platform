use aws_sdk_dynamodb;
use lambda_http::request::RequestContext::ApiGatewayV2;
use lambda_http::{service_fn, Error, Request, RequestExt};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&shared_config);
    let func = service_fn(|req| handler(req, &client));
    lambda_http::run(func).await?;
    Ok(())
}

async fn handler(request: Request, _client: &aws_sdk_dynamodb::Client) -> Result<Value, Error> {
    match request.request_context() {
        ApiGatewayV2(req) => {
            let authorizer = req.authorizer.unwrap().lambda;

            Ok(json!({
                 "user_id": authorizer.get("user_id").unwrap(),
                 "email": authorizer.get("user_email").unwrap()
            }))
        }
        _ => Ok(json!({ "": "" })),
    }
}
