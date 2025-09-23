pub use crate::roller::croll::CrollResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CharacterInitiative {
    pub croll_result: CrollResult,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Battle {
    pub characters: Vec<CharacterInitiative>,
    pub current_position: usize,
}

impl Battle {
    pub fn new(characters: Vec<CharacterInitiative>) -> Self {
        let mut battle = Self {
            characters,
            current_position: 0,
        };
        battle.characters.sort();
        battle
    }

    pub fn next_round(&mut self) {
        self.current_position += 1;
        self.current_position %= self.characters.len();
    }

    pub fn previous_round(&mut self) {
        if self.current_position == 0 {
            self.current_position = self.characters.len() - 1;
        } else {
            self.current_position -= 1;
        }
        self.current_position %= self.characters.len();
    }

    pub fn add_characters(&mut self, characters: &[CharacterInitiative]) -> Result<(), String> {
        for character in characters {
            self.add_character(character)?;
        }
        Ok(())
    }

    pub fn add_character(&mut self, character: &CharacterInitiative) -> Result<(), String> {
        if self
            .characters
            .iter()
            .map(|el| &el.name)
            .any(|el| el.eq_ignore_ascii_case(&character.name))
        {
            return Err(format!("Character `{}` already in battle!", character.name));
        }
        self.characters.push(character.clone());
        self.characters.sort();
        Ok(())
    }

    pub fn remove_character(&mut self, name: &str) -> Result<(), String> {
        let position = self
            .characters
            .iter()
            .position(|c| c.name.eq(name))
            .ok_or(format!("Character `{}` not found.", name))?;
        self.characters.remove(position);
        Ok(())
    }
}
