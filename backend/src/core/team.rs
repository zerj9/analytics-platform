use crate::core::create_id;
use crate::data::Database;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Create {
    pub name: String,
}

impl Team {
    pub async fn create<T: Database>(database: T, team: &Create) -> Result<()> {
        let id = create_id(30).await;
        let db_resp = database
            .create_team(&Team {
                id,
                name: team.name.clone(),
                active: true,
            })
            .await;

        match db_resp {
            Ok(()) => Ok(()),
            Err(_) => Err(anyhow!("failed to create team")),
        }
    }

    pub async fn from_id<T: Database>(database: T, id: &str) -> Result<Self> {
        Ok(database.get_team_by_id(id).await.unwrap())
    }
}
