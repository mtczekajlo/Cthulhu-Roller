use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CharacterVariable {
    pub current: i32,
    pub initial: i32,
    pub start: i32,
    pub max: i32,
}

impl CharacterVariable {
    pub fn new(start: i32, max: i32) -> Self {
        Self {
            current: start,
            initial: start,
            start,
            max,
        }
    }

    pub fn modify(&mut self, delta: i32) {
        self.current = (self.current + delta).clamp(0, self.max);
    }

    pub fn update_initial(&mut self) {
        self.initial = self.current;
    }

    pub fn reset(&mut self) {
        self.current = self.start;
        self.initial = self.start;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Character {
    pub name: String,
    pub hp: CharacterVariable,
    pub sanity: CharacterVariable,
    pub luck: CharacterVariable,
    pub major_wound: bool,
    pub dead: bool,
    pub fragile_mind: bool,
    pub insane: bool,
}

impl Character {
    pub fn new(name: String, hp: i32, sanity: i32, luck: i32) -> Self {
        Character {
            name,
            hp: CharacterVariable::new(hp, hp),
            sanity: CharacterVariable::new(sanity, 99),
            luck: CharacterVariable::new(luck, 99),
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.dead = false;
        self.major_wound = false;
        self.insane = false;
        self.fragile_mind = false;
        self.hp.reset();
        self.sanity.reset();
        self.luck.reset();
    }

    pub fn name(&self) -> String {
        format!("**`{}`**", self.name)
    }

    pub fn status_hp(&self) -> String {
        let mut comment: Option<&str> = None;
        if self.dead {
            comment = Some(" 💀");
        } else if self.major_wound {
            comment = Some(" 🤕");
        }

        format!(
            "❤️: **{}**/{}{}",
            self.hp.current,
            self.hp.max,
            comment.unwrap_or_default(),
        )
    }

    pub fn status_sanity(&self) -> String {
        let mut comment: Option<&str> = None;
        if self.insane {
            comment = Some(" 🤯")
        } else if self.fragile_mind {
            comment = Some(" 😨")
        }

        format!(
            "🧠: **{}**{}",
            self.sanity.current,
            comment.unwrap_or_default(),
        )
    }

    pub fn status_luck(&self) -> String {
        format!("🍀: **{}**/{}", self.luck.current, self.luck.initial,)
    }

    pub fn status(&self) -> String {
        format!(
            "{}\n{}\n{}",
            self.status_hp(),
            self.status_sanity(),
            self.status_luck()
        )
    }

    pub fn status_oneline(&self) -> String {
        format!(
            "{} : {} | {} | {}",
            self.name(),
            self.status_hp(),
            self.status_sanity(),
            self.status_luck()
        )
    }

    pub fn status_named(&self) -> String {
        format!("{}\n{}", self.name(), self.status())
    }
}
