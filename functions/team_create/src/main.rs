use aws_sdk_dynamodb;
use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::request::RequestContext::ApiGatewayV2;
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};
use serde_json::{json, Value};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let dynamodb = aws_sdk_dynamodb::Client::new(&shared_config);
    let func = service_fn(|req| handler(req, &dynamodb));
    lambda_http::run(func).await?;
    Ok(())
}

async fn handler(
    request: Request,
    dynamodb: &aws_sdk_dynamodb::Client,
) -> Result<Response<Body>, Error> {
    match request.request_context() {
        ApiGatewayV2(req) => {
            let authorizer = req.authorizer.unwrap().lambda;
            let user_id = authorizer.get("user_id").unwrap();
            let user_type = authorizer.get("user_type").unwrap();

            match request.body() {
                lambda_http::Body::Text(text) => {
                    let body: Value = serde_json::from_str(text).unwrap();
                    let team_name_json = body.get("team_name").and_then(|tn| tn.as_str());

                    match team_name_json {
                        Some(team_name) => {
                            if user_type == "SuperAdmin" {
                                dynamodb
                                    .put_item()
                                    .table_name(env::var("TABLE").unwrap())
                                    .item("PK", AttributeValue::S(format!("TEAM#{}", team_name)))
                                    .item("SK", AttributeValue::S(format!("TEAM#{}", team_name)))
                                    .send()
                                    .await
                                    .expect("Failed to create team");
                                Ok(Response::builder()
                                    .status(200)
                                    .body(
                                        json!({ "team_name": team_name, "admin": user_id })
                                            .to_string()
                                            .into(),
                                    )
                                    .unwrap())
                            } else {
                                Ok(Response::builder()
                                    .status(403)
                                    .body("Forbidden".into())
                                    .unwrap())
                            }
                        }
                        _ => {
                            Ok(Response::builder().status(400).body("bad request".into()).unwrap())
                        }
                    }
                }
                _ => Ok(Response::builder().status(400).body("bad request".into()).unwrap()),
            }
        }
        _ => Ok(Response::builder().status(400).body("bad request".into()).unwrap()),
    }
}
