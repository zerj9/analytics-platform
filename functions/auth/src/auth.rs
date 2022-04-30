use aws_sdk_dynamodb;
use lambda_http::{Body, IntoResponse, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct UserRecord {
    #[serde(rename = "PK")]
    email: String,
}

#[derive(Debug)]
struct User {
    email: String,
}

impl From<UserRecord> for User {
    fn from(record: UserRecord) -> Self {
        Self {
            email: record.email.strip_prefix("EMAIL#").unwrap().to_string(),
        }
    }
}

impl User {
    async fn from_email(dynamodb: &aws_sdk_dynamodb::Client, email: &str) -> Option<User> {
        dynamodb
            .get_item()
            .table_name(env::var("TABLE").unwrap())
            .key(
                "PK",
                aws_sdk_dynamodb::model::AttributeValue::S(String::from(format!(
                    "EMAIL#{}",
                    email
                ))),
            )
            .key(
                "SK",
                aws_sdk_dynamodb::model::AttributeValue::S(String::from(format!(
                    "EMAIL#{}",
                    email
                ))),
            )
            .send()
            .await
            .expect("DB Call Failed")
            .item
            .and_then(|user_item| {
                serde_dynamo::from_item(user_item)
                    .expect("dynamodb to UserRecord conversion failed")
            })
            .and_then(|user_record: UserRecord| Some(User::from(user_record)))
    }
}

pub async fn login(dynamodb: &aws_sdk_dynamodb::Client, event: Request) -> Response<Body> {
    match event.query_string_parameters().first("email") {
        None => missing_parameter_response("email"),
        Some(email_query_param) => {
            let user = User::from_email(dynamodb, email_query_param).await;
            match user {
                None => Response::builder()
                    .status(400)
                    .body("user not found".into())
                    .expect(""),
                Some(user) => {
                    println!("{:?}", user.email);
                    format!("found user").into_response()
                }
            }
        }
    }
}

pub async fn authenticate(dynamodb: &aws_sdk_dynamodb::Client, event: Request) -> Response<Body> {
    match event.query_string_parameters().first("email") {
        None => missing_parameter_response("email"),
        Some(email_query_param) => {
            let user = User::from_email(dynamodb, email_query_param).await;
            match user {
                None => Response::builder()
                    .status(400)
                    .body("user not found".into())
                    .expect(""),
                Some(user) => {
                    println!("{:?}", user.email);
                    format!("found user").into_response()
                }
            }
        }
    }
}

fn missing_parameter_response(parameter: &str) -> Response<Body> {
    Response::builder()
        .status(400)
        .body(format!("{} parameter missing", parameter).into())
        .expect("failed to render response")
}
