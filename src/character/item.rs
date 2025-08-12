use crate::locale::{LocaleLang, LocaleTag, locale_entry_by_tag};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Item {
    pub name: String,
    pub quantity: i32,
    pub unit: Option<String>,
}

pub enum ItemError {
    CantRemove(i32),
}

impl ItemError {
    pub fn to_string(&self, lang: LocaleLang) -> String {
        match (self, lang) {
            (ItemError::CantRemove(q), LocaleLang::Polski) => {
                format!("Nie można usunąć {q} jednostek.")
            }
            (ItemError::CantRemove(q), LocaleLang::English) => format!("Can't remove {q} units."),
        }
    }
}

impl Item {
    pub fn new(name: &str, quantity: i32, unit: Option<&str>) -> Self {
        Self {
            name: name.into(),
            quantity,
            unit: unit.map(|u| u.into()),
        }
    }

    pub fn add(&mut self, quantity: i32) {
        self.quantity += quantity;
    }

    pub fn remove(&mut self, mut quantity: i32) -> Result<Item, ItemError> {
        if self.quantity < quantity {
            quantity = self.quantity
        }
        self.quantity -= quantity;

        let mut removed_item = self.clone();
        removed_item.quantity = quantity;

        Ok(removed_item)
    }

    pub fn to_string(&self, lang: LocaleLang) -> String {
        let mut out = self.name.clone();
        if self.quantity != 1 {
            out.push_str(
                format!(
                    " [{} {}]",
                    self.quantity,
                    self.unit
                        .clone()
                        .unwrap_or(locale_entry_by_tag(LocaleTag::Pcs).get(lang))
                )
                .as_str(),
            );
        }
        out
    }
}
