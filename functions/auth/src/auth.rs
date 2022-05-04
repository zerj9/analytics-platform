use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{Duration, Utc};
use lambda_http::{Body, IntoResponse, Response};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::json;
use std::env;
use uuid::Uuid;

#[derive(Debug)]
struct User {
    id: String,
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
                let item = &resp.items.expect("User item could not be accessed in response")[0];
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

    async fn create_auth_session(&self, dynamodb: &aws_sdk_dynamodb::Client) -> Result<String, ()> {
        println!("Creating auth session for {}", self.email);
        let session_id = Uuid::new_v4();
        let code: String = (0..8).map(|_| thread_rng().sample(Alphanumeric) as char).collect(); // Fix this at some point?
        let code = code.to_uppercase();
        let ts_plus_5_minutes = (Utc::now() + Duration::minutes(5)).timestamp();
        dynamodb
            .put_item()
            .table_name(env::var("TABLE").unwrap())
            .item("PK", AttributeValue::S(format!("USER#{}", self.id)))
            .item("SK", AttributeValue::S(format!("AUTHSESSION#{}", session_id)))
            .item("GSI1PK", AttributeValue::S(format!("AUTHSESSION#{}", session_id)))
            .item("GSI1SK", AttributeValue::S(format!("AUTHSESSION#{}", session_id)))
            .item("code", AttributeValue::S(format!("{}", code)))
            .item("expiry", AttributeValue::N(ts_plus_5_minutes.to_string()))
            .send()
            .await
            .expect("Failed to create session");
        Ok(session_id.to_string())
    }
}

pub async fn login(dynamodb: &aws_sdk_dynamodb::Client, email: &str) -> Response<Body> {
    let user = User::from_email(dynamodb, &email).await;
    println!("User record found by email: {:?}", user);
    match user {
        None => Response::builder().status(400).body("user not found".into()).unwrap(),
        Some(user) => {
            // TODO: Create AUTH_SESSION in db
            let auth_session_id = user.create_auth_session(dynamodb).await;
            // TODO: Send email with code
            json!({ "auth_session_id": auth_session_id.unwrap() }).into_response()
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
        None => Response::builder().status(400).body("user not found".into()).expect(""),
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
