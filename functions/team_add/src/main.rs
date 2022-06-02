use aws_sdk_dynamodb;
use aws_sdk_dynamodb::model::AttributeValue;
use lambda_http::request::RequestContext::ApiGatewayV2;
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};
use serde_json::Value;
use std::env;

use model;

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
    let query_string_parameters = request.query_string_parameters();
    let team_name_parameter = query_string_parameters.first("team_name").unwrap();
    let team_response = model::Team::from_name(dynamodb, team_name_parameter).await;

    match request.request_context() {
        ApiGatewayV2(req) => {
            let authorizer = req.authorizer.unwrap().lambda;
            let requesting_user_type = authorizer.get("user_type").unwrap();

            match request.body() {
                lambda_http::Body::Text(text) => {
                    let body: Value = serde_json::from_str(text).unwrap();
                    let user_email_json = body.get("user_email").unwrap().as_str().unwrap();
                    let user = model::User::from_email(dynamodb, user_email_json).await.unwrap();

                    match team_response {
                        Some(team) => {
                            if requesting_user_type == "SuperAdmin" {
                                dynamodb
                                    .put_item()
                                    .table_name(env::var("TABLE").unwrap())
                                    .item("PK", AttributeValue::S(format!("TEAM#{}", team.name)))
                                    .item("SK", AttributeValue::S(format!("USER#{}", user.id)))
                                    .item("GSI1PK", AttributeValue::S(format!("USER#{}", user.id)))
                                    .item(
                                        "GSI1SK",
                                        AttributeValue::S(format!("TEAM#{}", team.name)),
                                    )
                                    .send()
                                    .await
                                    .expect("Failed to add user to team");
                                Ok(Response::builder().status(200).body("success".into()).unwrap())
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
