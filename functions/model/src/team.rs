#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub admins: Option<Vec<String>>,
    pub users: Option<Vec<String>>,
}

impl Team {
    pub async from_name(dynamodb: &aws_sdk_dynamodb::Client, name: &str) -> Option<Team> {
        let resp = dynamodb
            .get_item()
            .table_name(env::var("TABLE").unwrap())
            .key("PK", AttributeValue::S(format!("TEAM#{}", name)))
            .key("SK", AttributeValue::S(format!("TEAM#{}", name)))
            .send()
            .await
            .expect("Query failed: Get team by name");

        match resp.item {
            None => None,
            Some(item) => Some(Team {
                name: item.get("PK").unwrap().as_s().unwrap().strip_prefix("TEAM#").unwrap().into(),
                name: item
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
}
