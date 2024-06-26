use crate::core::create_id;
use crate::data::Database;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Org {
    pub id: String,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Create {
    pub id: String,
    pub name: String,
    pub active: bool,
}

impl Org {
    pub async fn from_id<T: Database>(database: T, id: &str) -> Result<Self> {
        Ok(database.get_org_by_id(id).await.unwrap())
    }

    pub async fn create<T: Database>(database: T, org: &Create) -> Result<()> {
        let org_id = create_id(30).await;
        let db_resp = database
            .create_org(&Org {
                id: org_id,
                name: org.name.clone(),
                active: true,
            })
            .await;

        match db_resp {
            Ok(()) => Ok(()),
            Err(_) => Err(anyhow!("failed to create org")),
        }
    }

    pub async fn delete<T: Database>(database: T, id: &str) -> Result<()> {
        database.delete_org(id).await?;
        Ok(())
    }
}
