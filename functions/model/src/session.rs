use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::env;

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub csrf_token: String,
    pub expiry: DateTime<Utc>,
}

impl Session {
    pub async fn from_id(dynamodb: &aws_sdk_dynamodb::Client, id: &str) -> Option<Session> {
        let resp = dynamodb
            .query()
            .table_name(env::var("TABLE").unwrap())
            .index_name("GSI1")
            .key_condition_expression("GSI1PK = :session_id and GSI1SK = :session_id")
            .expression_attribute_values(
                ":session_id",
                AttributeValue::S(format!("SESSION#{}", id)),
            )
            .expression_attribute_values(":U", AttributeValue::S("USER#".into()))
            .expression_attribute_values(":S", AttributeValue::S("SESSION#".into()))
            .filter_expression("begins_with(PK, :U) and begins_with(SK, :S)")
            .send()
            .await
            .expect("Query failed: Get session by id");

        match resp.count {
            1 => {
                let item = &resp.items.expect("User item could not be accessed in response")[0];
                let naive_expiry = NaiveDateTime::from_timestamp(
                    item.get("expiry")
                        .expect("expiry attribute not found in SESSION item")
                        .as_n()
                        .unwrap()
                        .parse()
                        .unwrap(),
                    0,
                );
                let expiry_dt: DateTime<Utc> = DateTime::from_utc(naive_expiry, Utc);

                if expiry_dt > Utc::now() {
                    None
                } else {
                    Some(Session {
                        id: item
                            .get("SK")
                            .unwrap()
                            .as_s()
                            .unwrap()
                            .strip_prefix("SESSION#")
                            .expect("Failed to parse SK: SESSION# attribute")
                            .into(),
                        user_id: item
                            .get("PK")
                            .unwrap()
                            .as_s()
                            .unwrap()
                            .strip_prefix("USER#")
                            .expect("Failed to parse PK: USER# attribute")
                            .into(),
                        csrf_token: item.get("csrf_token").unwrap().as_s().unwrap().into(),
                        expiry: DateTime::from_utc(naive_expiry, Utc),
                    })
                }
            }
            _ => None,
        }
    }
}
