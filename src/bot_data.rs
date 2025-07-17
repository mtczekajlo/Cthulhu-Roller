#[cfg(feature = "character-sheet")]
use crate::character::Character;
use crate::{locale::*, types::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use tokio::{fs, sync::RwLock};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserData {
    #[cfg(feature = "character-sheet")]
    pub characters: HashMap<String, Character>,
    #[cfg(feature = "character-sheet")]
    pub active_character: Option<String>,
    pub lang: LocaleLang,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InnerData {
    pub users: UsersHashMap,
    pub max_characters_per_user: usize,
    pub gm_role_name: String,
}

impl InnerData {
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub async fn from_json(path: &str) -> Result<InnerData, Error> {
        Ok(serde_json::from_str::<InnerData>(
            fs::read_to_string(path).await?.as_str(),
        )?)
    }
}

pub struct Data {
    pub data: RwLock<InnerData>,
    pub save_path: String,
}

impl Data {
    const DB_JSON: &str = "db.json";
    const DB_BACKUP_JSON: &str = "db-backup.json";

    pub async fn load_from_file() -> Self {
        match InnerData::from_json(Self::DB_JSON).await {
            Ok(inner_data) => Data {
                data: RwLock::new(inner_data),
                save_path: Self::DB_JSON.to_string(),
            },
            Err(_) => {
                eprintln!("No save file found, starting fresh.");
                Data::empty(Self::DB_JSON)
            }
        }
    }

    pub fn empty(path: &str) -> Self {
        Data {
            data: RwLock::new(InnerData {
                users: HashMap::new(),
                max_characters_per_user: 5,
                gm_role_name: "GM".into(),
            }),
            save_path: path.to_string(),
        }
    }

    async fn save_to_file(&self, path: &str) -> Result<(), Error> {
        let r = self.data.read().await.to_json();
        if let Err(e) = &r {
            eprintln!("{e:?}");
        }
        let r = fs::write(path, r?).await;
        if let Err(e) = &r {
            eprintln!("{e:?}");
        }
        Ok(())
    }

    pub fn get_db_path(&self) -> String {
        self.save_path.clone()
    }

    pub async fn save(&self) -> Result<(), Error> {
        self.save_to_file(&self.save_path).await
    }

    pub async fn quicksave(&self) -> Result<(), Error> {
        self.save_to_file(Self::DB_BACKUP_JSON).await
    }

    pub async fn quickload(&self) -> Result<(), Error> {
        let mut inner_data = self.data.write().await;
        inner_data.clone_from(&InnerData::from_json(Self::DB_BACKUP_JSON).await?);
        Ok(())
    }
}

impl Display for InnerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            serde_json::to_string_pretty(&self)
                .map_err(|_| std::fmt::Error)?
                .as_str(),
        )
    }
}
