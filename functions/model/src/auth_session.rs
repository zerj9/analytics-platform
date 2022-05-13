use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::hash_map::HashMap;
use std::env;

#[derive(Debug)]
pub struct AuthSession {
    pub id: String,
    pub user_id: String,
    pub code: String,
    pub expiry: DateTime<Utc>,
}

impl AuthSession {
    // TODO: Move logic to create_session function. Use db transaction in that function
    pub async fn delete(&self, dynamodb: &aws_sdk_dynamodb::Client) -> () {
        dynamodb
            .delete_item()
            .table_name(env::var("TABLE").unwrap())
            .key("PK", AttributeValue::S(format!("USER#{}", self.user_id)))
            .key("SK", AttributeValue::S(format!("AUTHSESSION#{}", self.id)))
            .send()
            .await
            .expect("AuthSession::delete failed");
        // TODO: throw error if record doesn't exist
    }
}

impl From<HashMap<String, AttributeValue>> for AuthSession {
    fn from(item: HashMap<String, AttributeValue>) -> Self {
        let naive_expiry = NaiveDateTime::from_timestamp(
            item.get("expiry")
                .expect("expiry attribute not found in AUTHSESSION item")
                .as_n()
                .unwrap()
                .parse()
                .unwrap(),
            0,
        );
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
            expiry: DateTime::from_utc(naive_expiry, Utc),
        }
    }
}
