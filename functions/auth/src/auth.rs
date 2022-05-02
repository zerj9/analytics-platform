use aws_sdk_dynamodb;
use lambda_http::{Body, IntoResponse, Response};
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

impl User {
    async fn from_email(dynamodb: &aws_sdk_dynamodb::Client, email: &str) -> Option<User> {
        let resp = dynamodb
            .query()
            .table_name(env::var("TABLE").unwrap())
            .index_name("GSI1")
            .key_condition_expression("GSI1PK = :email and GSI1SK = :email")
            .expression_attribute_values(":email", AttributeValue::S(format!("EMAIL#{}", email)))
            .expression_attribute_values(":U", AttributeValue::S("USER#".into()))
            .filter_expression("begins_with(PK, :U) and begins_with(SK, :U)")
            .send()
            .await
            .expect("Query failed: Get user by email");

        match resp.count {
            1 => {
                let item = &resp
                    .items
                    .expect("User item could not be accessed in response")[0];
                Some(User {
                    id: item
                        .get("PK")
                        .expect("PK attribute not found in User item")
                        .as_s()
                        .unwrap()
                        .strip_prefix("USER#")
                        .expect("Failed to parse PK: USER# attribute")
                        .into(),
                    email: item
                        .get("GSI1PK")
                        .expect("GSI1PK attribute not found in User item")
                        .as_s()
                        .unwrap()
                        .strip_prefix("EMAIL#")
                        .expect("Failed to parse GSI1PK: EMAIL# attribute")
                        .into(),
                })
            }
            _ => None,
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

pub async fn login(dynamodb: &aws_sdk_dynamodb::Client, email: &str) -> Response<Body> {
    let user = User::from_email(dynamodb, &email).await;
    match user {
        None => Response::builder()
            .status(400)
            .body("user not found".into())
            .expect(""),
        Some(user) => {
            println!("{:?}", user.email);
            // TODO: Create AUTH_SESSION in db
            // TODO: Send email with code
            format!("found user").into_response()
        }
    }
}

pub async fn authenticate(
    dynamodb: &aws_sdk_dynamodb::Client,
    email: &str,
    auth_session_id: &str,
    code: &str,
) -> Response<Body> {
    let user = User::from_email(dynamodb, email).await;
    match user {
        None => Response::builder()
            .status(400)
            .body("user not found".into())
            .expect(""),
        Some(user) => {
            // Get AUTH_SESSION from db
            // Validate code from request
            // Create SESSION in db and delete AUTH_SESSION
            println!("{:?}", user.email);
            format!(
                "Authenticate request for {} with auth session {} and code {}",
                email, auth_session_id, code
            )
            .into_response()
        }
    }
}
