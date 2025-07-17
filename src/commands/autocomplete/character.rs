use crate::types::{AttributeMap, Context};

pub async fn autocomplete_my_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    let user_id = ctx.author().id;

    let mut characters: Vec<_> = data
        .users
        .get(&user_id.get())
        .into_iter()
        .flat_map(|user| user.characters.keys())
        .filter(move |name| name.to_lowercase().contains(&partial.to_lowercase()))
        .take(25) // Discord's limit
        .cloned()
        .collect();
    characters.sort();
    characters
}

pub async fn autocomplete_my_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);

    if user_data.is_none() {
        eprintln!("No user_data");
        return vec![];
    }

    let user_data = user_data.unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        eprintln!("No character");
        return vec![];
    }

    let character = character.unwrap();

    let mut skills: Vec<_> = character
        .skills
        .values()
        .map(|skill| skill.name.get(&user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_improvable_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);

    if user_data.is_none() {
        eprintln!("No user_data");
        return vec![];
    }

    let user_data = user_data.unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        eprintln!("No character");
        return vec![];
    }

    let character = character.unwrap();

    let mut skills: Vec<_> = character
        .skills
        .iter()
        .filter(|(_, skill)| skill.to_improve)
        .map(|(_, skill)| skill.name.get(&user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_fight_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);

    if user_data.is_none() {
        eprintln!("No user_data");
        return vec![];
    }

    let user_data = user_data.unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        eprintln!("No character");
        return vec![];
    }

    let character = character.unwrap();

    let mut skills: Vec<_> = character
        .skills
        .iter()
        .filter(|(_, v)| {
            v.name
                .en
                .to_lowercase()
                .contains(&"Fighting".to_string().to_lowercase())
                || v.name
                    .en
                    .to_lowercase()
                    .contains(&"Firearms".to_string().to_lowercase())
        })
        .map(|(_, skill)| skill.name.get(&user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_attributes<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);

    if user_data.is_none() {
        return vec![];
    }

    let user_data = user_data.unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut attributes: Vec<_> = AttributeMap::from(character.attributes.clone())
        .values()
        .map(|attribute| attribute.name.get(&user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    attributes.sort();
    attributes
}

pub async fn autocomplete_my_equipment<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);

    if user_data.is_none() {
        eprintln!("No user_data");
        return vec![];
    }

    let user_data = user_data.unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        eprintln!("No character");
        return vec![];
    }

    let character = character.unwrap();

    let mut weapons: Vec<_> = character
        .items
        .iter()
        .filter(|&name| name.contains(&partial.to_lowercase()))
        .cloned()
        .collect();
    weapons.sort();
    weapons
}

pub async fn autocomplete_my_weapons<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);

    if user_data.is_none() {
        eprintln!("No user_data");
        return vec![];
    }

    let user_data = user_data.unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        eprintln!("No character");
        return vec![];
    }

    let character = character.unwrap();

    let mut weapons: Vec<_> = character
        .weapons
        .iter()
        .map(|weapon| weapon.name.clone())
        .filter(|name| name.partial_match(&partial.to_lowercase()))
        .map(|name| name.get(&user_data.lang))
        .collect();
    weapons.sort();
    weapons
}

pub async fn autocomplete_any_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    let mut characters: Vec<_> = data
        .users
        .values()
        .flat_map(|user| user.characters.keys())
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .take(25) // Discord's limit
        .cloned()
        .collect();
    characters.sort();
    characters
}

pub async fn autocomplete_any_active_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    let mut characters: Vec<_> = data
        .users
        .values()
        .filter_map(move |user| user.active_character.as_ref())
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .take(25) // Discord's limit
        .cloned()
        .collect();
    characters.sort();
    characters
}
