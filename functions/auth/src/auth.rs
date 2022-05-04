use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{Duration, Utc};
use lambda_http::{Body, IntoResponse, Response};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
// use serde_json::json;
use std::collections::hash_map::HashMap;
use std::env;
use uuid::Uuid;

#[derive(Debug)]
struct User {
    id: String,
    email: String,
}

#[derive(Debug)]
struct AuthSession {
    id: String,
    user_id: String,
    code: String,
    expiry: i64,
}

impl AuthSession {
    async fn delete(&self, dynamodb: &aws_sdk_dynamodb::Client) -> Result<(), ()> {
        dynamodb
            .delete_item()
            .table_name(env::var("TABLE").unwrap())
            .key("PK", AttributeValue::S(format!("USER#{}", self.user_id)))
            .key("SK", AttributeValue::S(format!("AUTHSESSION#{}", self.id)))
            .send()
            .await
            .expect("AuthSession::delete failed");
        // TODO: throw error if record doesn't exist
        Ok(())
    }
}

impl From<HashMap<String, AttributeValue>> for AuthSession {
    fn from(item: HashMap<String, AttributeValue>) -> Self {
        AuthSession {
            id: item
                .get("SK")
                .expect("SK attribute not found in AUTHSESSION item")
                .as_s()
                .unwrap()
                .strip_prefix("AUTHSESSION#")
                .expect("Failed to parse SK: AUTHSESSION# attribute")
                .into(),
            user_id: item
                .get("PK")
                .expect("PK attribute not found in AUTHSESSION item")
                .as_s()
                .unwrap()
                .strip_prefix("USER#")
                .expect("Failed to parse PK: USER# attribute")
                .into(),
            code: item
                .get("code")
                .expect("code attribute not found in AUTHSESSION item")
                .as_s()
                .unwrap()
                .into(),
            expiry: item
                .get("expiry")
                .expect("expiry attribute not found in AUTHSESSION item")
                .as_n()
                .unwrap()
                .parse::<i64>()
                .expect("Failed to parse expiry string to i64")
                .into(),
        }
    }
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
            .item("GSI1PK", AttributeValue::S(format!("USER#{}", self.id)))
            .item("GSI1SK", AttributeValue::S(format!("USER#{}", self.id)))
            .item("code", AttributeValue::S(format!("{}", code)))
            .item("expiry", AttributeValue::N(ts_plus_5_minutes.to_string()))
            .send()
            .await
            .expect("Failed to create session");
        Ok(session_id.to_string())
    }

    async fn get_auth_sessions(
        &self,
        dynamodb: &aws_sdk_dynamodb::Client,
    ) -> Option<Vec<AuthSession>> {
        let resp = dynamodb
            .query()
            .table_name(env::var("TABLE").unwrap())
            .index_name("GSI1")
            .key_condition_expression("GSI1PK = :user_id and GSI1SK = :user_id")
            .expression_attribute_values(":user_id", AttributeValue::S(format!("USER#{}", self.id)))
            .expression_attribute_values(":U", AttributeValue::S("USER#".into()))
            .expression_attribute_values(":AS", AttributeValue::S("AUTHSESSION#".into()))
            .filter_expression("begins_with(PK, :U) and begins_with(SK, :AS)")
            .send()
            .await
            .expect("get_auth_sessions_by_email: db request failed");

        if resp.count == 0 {
            None
        } else {
            let mut auth_sessions: Vec<AuthSession> = Vec::new();
            for item in resp.items.unwrap() {
                let auth_session_item: AuthSession = item.into();
                auth_sessions.push(auth_session_item);
            }
            Some(auth_sessions)
        }
    }
}

pub async fn login(dynamodb: &aws_sdk_dynamodb::Client, email: &str) -> Response<Body> {
    let user = User::from_email(dynamodb, &email).await;
    println!("User record found by email: {:?}", user);
    match user {
        None => format!("login request success").into_response(),
        Some(user) => {
            let auth_session_id = user.create_auth_session(dynamodb).await;
            println!("AUTHSESSION#{:?} created for {}", auth_session_id, user.id);
            // TODO: Send email with code
            format!("login request success").into_response()
        }
    }
}

pub async fn authenticate(
    dynamodb: &aws_sdk_dynamodb::Client,
    email: &str,
    code: &str,
) -> Response<Body> {
    let user_record = User::from_email(dynamodb, email).await;
    // TODO: Add check to see if user is active

    match user_record {
        None => format!("Authentication failed").into_response(),
        Some(user) => {
            let auth_sessions_response = user.get_auth_sessions(dynamodb).await;

            println!("auth_sessions: {:?}", auth_sessions_response);

            match auth_sessions_response {
                Some(auth_sessions) => {
                    let matched_auth_session =
                        auth_sessions.iter().find(|&auth_session| auth_session.code == code);
                    match matched_auth_session {
                        Some(auth_session) => {
                            auth_session.delete(dynamodb).await.expect("AUTHSESSION delete failed");
                            // Create SESSION in db
                            format!("Authenticate request for {} with code {}", email, code)
                                .into_response()
                        }
                        None => format!("Authentication failed").into_response(),
                    }
                }
                None => format!("Authentication failed").into_response(),
            }
        }
    }
}
