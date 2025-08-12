#[cfg(feature = "character-sheet")]
use crate::character::Character;
use crate::{locale::LocaleLang, roller::battle::Battle, types::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, sync::Arc};
use tokio::{fs, sync::RwLock};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserData {
    #[cfg(feature = "character-sheet")]
    pub characters: HashMap<String, Character>,
    #[cfg(feature = "character-sheet")]
    pub active_character: Option<String>,
    pub lang: LocaleLang,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    pub battle: Option<Battle>,
    pub gm_role_name: String,
    pub max_characters_per_user: usize,
    pub users: UsersHashMap,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            battle: None,
            gm_role_name: "GM".into(),
            max_characters_per_user: 3,
            users: HashMap::new(),
        }
    }
}

impl Data {
    const DB_JSON: &str = "db.json";
    const DB_BACKUP_JSON: &str = "db-backup.json";

    pub fn get_db_path(&self) -> &'static str {
        Self::DB_JSON
    }

    pub async fn save(&self) -> Result<(), Error> {
        self.save_to_file(Self::DB_JSON).await
    }

    pub async fn load() -> Self {
        fs::try_exists(Self::DB_JSON).await.expect("No database file!");
        match Self::load_from_file(Self::DB_JSON).await {
            Ok(d) => {
                println!("Loaded database.");
                d
            }
            Err(e) => {
                eprintln!("Data loading error: {e}");
                let invalid_db_name = format!(
                    "db-invalid-{}.json",
                    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                );
                match fs::copy(Self::DB_JSON, &invalid_db_name).await {
                    Ok(_) => {
                        eprintln!("Saved invalid db as {invalid_db_name}")
                    }
                    Err(e) => eprintln!("{e}"),
                }
                eprintln!("Creating new database.");
                let d = Data::default();
                let _ = d.save().await;
                d
            }
        }
    }

    pub async fn quicksave(&self) -> Result<(), Error> {
        self.save_to_file(Self::DB_BACKUP_JSON).await
    }

    pub async fn quickload(&mut self) -> Result<(), Error> {
        self.save().await?;
        (*self).clone_from(&Data::from_json(Self::DB_BACKUP_JSON).await?);
        Ok(())
    }

    async fn load_from_file(path: &str) -> Result<Self, Error> {
        let json = fs::read_to_string(path).await?;
        Data::from_json(&json).await
    }

    async fn save_to_file(&self, path: &str) -> Result<(), Error> {
        let string = self.to_json()?;
        fs::write(path, string).await?;
        Ok(())
    }

    fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    async fn from_json(json: &str) -> Result<Data, Error> {
        Ok(serde_json::from_str::<Data>(json)?)
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            serde_json::to_string_pretty(&self)
                .map_err(|_| std::fmt::Error)?
                .as_str(),
        )
    }
}

pub struct ContextData {
    pub data: Arc<RwLock<Data>>,
}
