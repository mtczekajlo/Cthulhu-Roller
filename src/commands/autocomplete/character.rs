use itertools::Itertools;
use poise::serenity_prelude::ResolvedValue;

use crate::bot_data::UserData;
use crate::character::{ADDITIONAL_SKILLS, SPECIALIZED_SKILLS, skill_map_wrapper};
use crate::locale::{LOCALE_PULP_ARCHETYPES, LOCALE_PULP_TALENTS, locale_text_by_tag_lang};
use crate::types::{ApplicationContext, AttributeMap, Context, SkillMap};

pub async fn autocomplete_my_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    let user_id = ctx.author().id;

    let mut characters: Vec<_> = data
        .users
        .get(&user_id.get())
        .into_iter()
        .flat_map(|user| user.characters.keys())
        .filter(move |name| name.to_lowercase().contains(&partial.to_lowercase()))
        .take(25)
        .cloned()
        .collect();
    characters.sort();
    characters
}

pub async fn autocomplete_additional_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let mut skills: Vec<_> = ADDITIONAL_SKILLS
        .iter()
        .map(|(tag, _)| locale_text_by_tag_lang(user_data.lang, *tag))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_specialized_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let mut skills: Vec<_> = SPECIALIZED_SKILLS
        .iter()
        .map(|(tag, _)| locale_text_by_tag_lang(user_data.lang, *tag))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut skills: Vec<_> = character
        .skills
        .values()
        .map(|skill| skill.name.get(user_data.lang))
        .sorted()
        .dedup()
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_custom_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut skills: Vec<_> = character
        .skills
        .values()
        .filter(|skill| !skill.default)
        .map(|skill| skill.name.get(user_data.lang))
        .sorted()
        .dedup()
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_skills_with_additional<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let additional_skills: SkillMap = ADDITIONAL_SKILLS
        .iter()
        .map(|el| skill_map_wrapper(el.0, el.1).unwrap())
        .collect();

    let mut full_map: SkillMap = character.skills.clone();
    full_map.extend(additional_skills);

    let mut skills: Vec<_> = full_map
        .values()
        .map(|skill| skill.name.get(user_data.lang))
        .sorted()
        .dedup()
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_improvable_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut skills: Vec<_> = character
        .skills
        .iter()
        .filter(|(_, skill)| skill.to_improve)
        .map(|(_, skill)| skill.name.get(user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_fight_skills<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
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
        .map(|(_, skill)| skill.name.get(user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    skills.sort();
    skills
}

pub async fn autocomplete_my_attributes<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut attributes: Vec<_> = AttributeMap::from(character.attributes.clone())
        .values()
        .map(|attribute| attribute.name.get(user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    attributes.sort();
    attributes
}

pub async fn autocomplete_my_items<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut items: Vec<_> = character
        .items
        .iter()
        .map(|item| item.name.clone())
        .filter(|name| name.contains(&partial.to_lowercase()))
        .collect();
    items.sort();
    items
}

pub async fn autocomplete_my_item_quantity<'a>(ctx: ApplicationContext<'a>, _: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    if let ResolvedValue::String(item_name) = ctx
        .args
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("item_name"))
        .unwrap()
        .value
    {
        let mut quantity: Vec<_> = character
            .items
            .iter()
            .filter(|&item| item.name.contains(item_name))
            .map(|item| item.quantity.to_string())
            .collect();
        quantity.sort();
        return quantity;
    }

    vec![]
}

pub async fn autocomplete_character_items<'a>(ctx: ApplicationContext<'a>, partial: &'a str) -> Vec<String> {
    if let ResolvedValue::String(character_name) = ctx
        .args
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("character_name"))
        .unwrap()
        .value
    {
        for user_data in ctx.data().data.read().await.users.values() {
            if let Some(character) = user_data.characters.get(character_name) {
                let mut items: Vec<_> = character
                    .items
                    .iter()
                    .map(|item| item.name.clone())
                    .filter(|name| name.contains(&partial.to_lowercase()))
                    .collect();
                items.sort();
                return items;
            }
        }
    }

    vec![]
}

pub async fn autocomplete_character_weapons<'a>(ctx: ApplicationContext<'a>, partial: &'a str) -> Vec<String> {
    let character_name = if let ResolvedValue::String(s) = ctx
        .args
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("character_name"))
        .unwrap()
        .value
    {
        s
    } else {
        ""
    };

    for user_data in ctx.data().data.read().await.users.values() {
        if let Some(character) = user_data.characters.get(character_name) {
            let mut weapons: Vec<_> = character
                .weapons
                .iter()
                .map(|weapon| weapon.name.clone())
                .filter(|name| name.to_ascii_lowercase().contains(&partial.to_lowercase()))
                .collect();
            weapons.sort();
            return weapons;
        }
    }

    vec![]
}

pub async fn autocomplete_my_weapons<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut weapons: Vec<_> = character
        .weapons
        .iter()
        .map(|weapon| weapon.name.clone())
        .filter(|name| name.to_ascii_lowercase().contains(&partial.to_lowercase()))
        .collect();
    weapons.sort();
    weapons
}

pub async fn autocomplete_my_weapon_attacks<'a>(ctx: ApplicationContext<'a>, _: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let weapon_name = if let ResolvedValue::String(s) = ctx
        .args
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("weapon_name"))
        .unwrap()
        .value
    {
        s
    } else {
        ""
    };

    character
        .weapons
        .iter()
        .find(|weapon| weapon.name.to_ascii_lowercase().contains(&weapon_name.to_lowercase()))
        .unwrap()
        .attacks
        .iter()
        .map(|a| a.count.to_string())
        .collect()
}

pub async fn autocomplete_any_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    let mut characters: Vec<_> = data
        .users
        .values()
        .flat_map(|user| user.characters.keys())
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .take(25)
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
        .take(25)
        .cloned()
        .collect();
    characters.sort();
    characters
}

pub async fn autocomplete_pulp_archetypes<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let mut archetypes: Vec<_> = LOCALE_PULP_ARCHETYPES
        .iter()
        .map(|(_, archetype)| archetype.get(user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    archetypes.sort();
    archetypes
}

pub async fn autocomplete_pulp_talents<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let mut talents: Vec<_> = LOCALE_PULP_TALENTS
        .iter()
        .map(|(_, archetype)| archetype.get(user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    talents.sort();
    talents
}

pub async fn autocomplete_my_pulp_talents<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let active = user_data.active_character.clone().unwrap_or_default();
    let character = user_data.characters.get(&active);

    if character.is_none() {
        return vec![];
    }

    let character = character.unwrap();

    let mut talents: Vec<_> = character
        .pulp_talents
        .iter()
        .filter(|&lt| lt.partial_match_ignore_case(&partial.to_lowercase()))
        .map(|name| name.get(user_data.lang))
        .collect();
    talents.sort();
    talents
}
