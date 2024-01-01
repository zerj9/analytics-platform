use crate::core::{Email, User};
use crate::data::{Database, UserStore};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue as AV;
use aws_sdk_dynamodb::Client;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Dynamodb {
    pub client: Client,
    pub table_name: &'static str,
}

impl Database for Dynamodb {}

fn split_at_hash(input: &str) -> &str {
    input.split_once('#').unwrap().1
}

impl From<HashMap<String, AV>> for User {
    fn from(value: HashMap<String, AV>) -> Self {
        let user = User {
            id: split_at_hash(value.get("PK").unwrap().as_s().unwrap()).to_string(),
            email: split_at_hash(value.get("GSI1PK").unwrap().as_s().unwrap()).to_string(),
            first_name: value.get("first_name").unwrap().as_s().unwrap().to_string(),
            last_name: value.get("last_name").unwrap().as_s().unwrap().to_string(),
            is_active: *value.get("is_active").unwrap().as_bool().unwrap(),
            r#type: value.get("user_type").unwrap().as_s().unwrap().to_string(),
        };
        user
    }
}

impl From<HashMap<String, AV>> for Email {
    fn from(value: HashMap<String, AV>) -> Self {
        Email {
            email: split_at_hash(value.get("PK").unwrap().as_s().unwrap()).to_string(),
            user_id: split_at_hash(value.get("GSI1PK").unwrap().as_s().unwrap()).to_string(),
        }
    }
}

#[async_trait]
impl UserStore for Dynamodb {
    async fn create_user(&self, user: &User) -> Result<()> {
        // Create the item to insert
        let mut item = std::collections::HashMap::new();
        let key = format!("{}{}", "USER#", user.id);
        let email = format!("{}{}", "EMAIL#", user.email);
        let r#type = format!("{}{}", "USERTYPE#", user.r#type);

        item.insert(String::from("PK"), AV::S(key.clone()));
        item.insert(String::from("SK"), AV::S(key));
        item.insert(String::from("GSI1PK"), AV::S(email.clone()));
        item.insert(String::from("GSI1SK"), AV::S(email));
        item.insert(String::from("GSI2PK"), AV::S(r#type.clone()));
        item.insert(String::from("GSI2SK"), AV::S(r#type));
        item.insert(String::from("first_name"), AV::S(user.first_name.clone()));
        item.insert(String::from("last_name"), AV::S(user.last_name.clone()));
        item.insert(String::from("user_type"), AV::S(user.r#type.clone()));

        self.client
            .put_item()
            .table_name(self.table_name)
            .set_item(Some(item))
            .send()
            .await?;
        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let email_key = format!("EMAIL#{email}");
        match self
            .client
            .get_item()
            .table_name(self.table_name)
            .key("PK", AV::S(email_key))
            .send()
            .await
        {
            Ok(response) => {
                let email_record: Email = response.item.unwrap().into();
                UserStore::get_user_by_id(self, &email_record.user_id).await
            }
            Err(_e) => Err(anyhow!("email not found")),
        }
    }

    async fn get_user_by_id(&self, id: &str) -> Result<User> {
        let key = format!("USER#{id}");
        match self
            .client
            .get_item()
            .table_name(self.table_name)
            .key("PK", AV::S(key.into()))
            .send()
            .await
        {
            Ok(response) => Ok(response.item.unwrap().into()),
            Err(_e) => Err(anyhow!("user not found")),
        }
    }
}
