use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use uuid::Uuid;

use crate::auth_session::AuthSession;
use crate::session::Session;

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub email: String,
}

impl User {
    pub async fn from_email(dynamodb: &aws_sdk_dynamodb::Client, email: &str) -> Option<User> {
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

    pub async fn from_id(dynamodb: &aws_sdk_dynamodb::Client, id: &str) -> Option<User> {
        let resp = dynamodb
            .get_item()
            .table_name(env::var("TABLE").unwrap())
            .key("PK", AttributeValue::S(format!("USER#{}", id)))
            .key("SK", AttributeValue::S(format!("USER#{}", id)))
            .send()
            .await
            .expect("Query failed: Get user by id");

        match resp.item {
            None => None,
            Some(item) => Some(User {
                id: item.get("PK").unwrap().as_s().unwrap().strip_prefix("USER#").unwrap().into(),
                email: item
                    .get("GSI1PK")
                    .unwrap()
                    .as_s()
                    .unwrap()
                    .strip_prefix("EMAIL#")
                    .unwrap()
                    .into(),
            }),
        }
    }

    pub async fn create_auth_session(
        &self,
        dynamodb: &aws_sdk_dynamodb::Client,
    ) -> Result<String, ()> {
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

    pub async fn create_session(
        &self,
        dynamodb: &aws_sdk_dynamodb::Client,
        session_seconds: Option<i64>,
    ) -> Session {
        println!("Creating session for {}", self.email);
        let session_seconds = session_seconds.unwrap_or(28800);
        let session_id = Uuid::new_v4();
        let csrf_token = Uuid::new_v4();
        let session_expiry = Utc::now() + Duration::seconds(session_seconds);
        dynamodb
            .put_item()
            .table_name(env::var("TABLE").unwrap())
            .item("PK", AttributeValue::S(format!("USER#{}", self.id)))
            .item("SK", AttributeValue::S(format!("SESSION#{}", session_id)))
            .item("GSI1PK", AttributeValue::S(format!("SESSION#{}", session_id)))
            .item("GSI1SK", AttributeValue::S(format!("SESSION#{}", session_id)))
            .item("csrf_token", AttributeValue::S(format!("{}", csrf_token)))
            .item("expiry", AttributeValue::N(session_expiry.timestamp().to_string()))
            .send()
            .await
            .expect("Failed to create session");
        Session {
            id: session_id.to_string(),
            user_id: self.id.to_owned(),
            csrf_token: csrf_token.to_string(),
            expiry: session_expiry,
        }
    }

    pub async fn get_auth_sessions(
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
