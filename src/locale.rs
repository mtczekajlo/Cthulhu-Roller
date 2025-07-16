use crate::types::{Error, LocaleTextMap, LocaleVec, SkillValMap};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum LocaleLang {
    #[default]
    English,
    Polski,
}

impl From<String> for LocaleLang {
    fn from(value: String) -> Self {
        match value.replace(' ', "").to_ascii_lowercase().as_str() {
            "pl" | "pol" | "polski" | "polish" => Self::Polski,
            _ => Self::English,
        }
    }
}

impl Display for LocaleLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LocaleLang::English => "english",
            LocaleLang::Polski => "polski",
        })
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Debug, Default, PartialOrd, Ord)]
pub struct LocaleText {
    pub en: String,
    pub pl: String,
}

impl LocaleText {
    pub fn new(en: &str, pl: &str) -> Self {
        Self {
            en: en.into(),
            pl: pl.into(),
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn new_single_lang(text: &str) -> Self {
        Self {
            en: text.into(),
            pl: text.into(),
        }
    }

    pub fn get(&self, language: &LocaleLang) -> String {
        match language {
            LocaleLang::English => self.en.clone(),
            LocaleLang::Polski => self.pl.clone(),
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn to_enum(&self) -> Option<LocaleTag> {
        LOCALE_TEXTS
            .iter()
            .find_map(|(tag, locale)| if self == locale { Some(*tag) } else { None })
    }

    #[cfg(feature = "character-sheet")]
    pub fn partial_match(&self, text: &str) -> bool {
        self.en.to_ascii_lowercase().contains(&text.to_ascii_lowercase())
            || self.pl.to_ascii_lowercase().contains(&text.to_ascii_lowercase())
    }

    #[cfg(feature = "character-sheet")]
    pub fn equals(&self, text: &str) -> bool {
        self.en.eq_ignore_ascii_case(text) || self.pl.eq_ignore_ascii_case(text)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum LocaleTag {
    Accounting,
    Agony,
    Anthropology,
    App,
    Appearance,
    Appraise,
    Archeology,
    ArtCraftAny,
    BodyHealed,
    Bonus,
    Build,
    CantSpendLuck,
    Charm,
    Climb,
    Con,
    Constitution,
    CreditRating,
    CriticalFailure,
    CriticalSuccess,
    CthulhuMythos,
    Damage,
    DamageBonus,
    Db,
    DeathInevitable,
    Dex,
    Dexterity,
    Dice,
    Disguise,
    Dodge,
    DriveAuto,
    Edu,
    Education,
    ElectricalRepair,
    ExtremeDamage,
    ExtremeSuccess,
    Failure,
    FastTalk,
    FightingBrawl,
    FirearmsHandgun,
    FirearmsRifleShotgun,
    FirstAid,
    GoneMad,
    HardSuccess,
    History,
    HitPoints,
    Hp,
    Impaling,
    IndefInsanity,
    Int,
    Intelligence,
    Intimidate,
    Items,
    Jump,
    KnockOut,
    LanguageOther,
    LanguageOwn,
    Law,
    LibraryUse,
    Listen,
    Locksmith,
    Luck,
    MajorWound,
    MajorWoundHealed,
    Malfunction,
    MaxSanitySet,
    MechanicalRepair,
    Medicine,
    MindHealed,
    MindShattered,
    Move,
    Mp,
    NaturalWorld,
    Navigate,
    NoItems,
    NoSuchSkill,
    NotMarked,
    Occult,
    OperateHeavyMachinery,
    Penalty,
    Persuade,
    Pilot,
    PointsTo,
    Pow,
    Power,
    Psychoanalysis,
    Psychology,
    Range,
    Ride,
    RollConCheckBlackOut,
    RollConCheckDie,
    RollIntCheck,
    Rolls,
    Sanity,
    ScienceAny,
    Siz,
    Size,
    Skill,
    SkillMarked,
    SleightOfHand,
    Sp,
    SpotHidden,
    Stealth,
    Str,
    Strength,
    Success,
    SurvivalAny,
    Swim,
    TempInsanity,
    TempInsanityThreat,
    Throw,
    Track,
    Unarmed,
    Weapon,
    WeaponJammed,
    Weapons,
    YouBlackOut,
    YouFell,
}

pub fn locale_text_entry(tag: LocaleTag, en: &'static str, pl: &'static str) -> (LocaleTag, LocaleText) {
    (tag, LocaleText::new(en, pl))
}

lazy_static! {
    static ref LOCALE_VEC: LocaleVec = vec![
        locale_text_entry(LocaleTag::Accounting, "Accounting", "Księgowość"),
        locale_text_entry(LocaleTag::Agony, "Agony!", "Agonia!"),
        locale_text_entry(LocaleTag::Anthropology, "Anthropology", "Antropologia"),
        locale_text_entry(LocaleTag::App, "APP", "WYG"),
        locale_text_entry(LocaleTag::Appearance, "Appearance", "Wygląd"),
        locale_text_entry(LocaleTag::Appraise, "Appraise", "Wycena"),
        locale_text_entry(LocaleTag::Archeology, "Archeology", "Archeologia"),
        locale_text_entry(LocaleTag::ArtCraftAny, "Art/Craft (any)", "Sztuka/Rzemiosło (dowolne)"),
        locale_text_entry(
            LocaleTag::BodyHealed,
            "your body healed enough to carry on...",
            "twoje ciało wyzdrowiało wystarczająco, by ruszyć dalej..."
        ),
        locale_text_entry(LocaleTag::Bonus, "➕ Bonus", "➕ Premiowe"),
        locale_text_entry(LocaleTag::Build, "Build", "Krzepa"),
        locale_text_entry(
            LocaleTag::CantSpendLuck,
            "Not enough Luck points to spend!",
            "Niewystarczająca ilość punktów Szczęścia!"
        ),
        locale_text_entry(LocaleTag::Charm, "Charm", "Urok Osobisty"),
        locale_text_entry(LocaleTag::Climb, "Climb", "Wspinaczka"),
        locale_text_entry(LocaleTag::Con, "CON", "KON"),
        locale_text_entry(LocaleTag::Constitution, "Constitution", "Kondycja"),
        locale_text_entry(LocaleTag::CreditRating, "Credit Rating", "Majętność"),
        locale_text_entry(
            LocaleTag::CriticalFailure,
            "🐙🐙🐙 CRITICAL FAILURE 🐙🐙🐙",
            "🐙🐙🐙 KRYTYCZNA PORAŻKA 🐙🐙🐙"
        ),
        locale_text_entry(
            LocaleTag::CriticalSuccess,
            "✨✨✨ CRITICAL SUCCESS ✨✨✨",
            "✨✨✨ KRYTYCZNY SUKCES ✨✨✨"
        ),
        locale_text_entry(LocaleTag::CthulhuMythos, "Cthulhu Mythos", "Mity Cthulhu"),
        locale_text_entry(LocaleTag::Damage, "Damage", "Obrażenia"),
        locale_text_entry(LocaleTag::DamageBonus, "Damage Bonus", "Modyfikator Obrażeń"),
        locale_text_entry(LocaleTag::Db, "DB", "MO"),
        locale_text_entry(
            LocaleTag::DeathInevitable,
            "Death is inevitable.",
            "Śmierć jest nieunikniona."
        ),
        locale_text_entry(LocaleTag::Dex, "DEX", "ZR"),
        locale_text_entry(LocaleTag::Dexterity, "Dexterity", "Zręczność"),
        locale_text_entry(LocaleTag::Dice, "dice", "kości"),
        locale_text_entry(LocaleTag::Disguise, "Disguise", "Charakteryzacja"),
        locale_text_entry(LocaleTag::Dodge, "Dodge", "Unik"),
        locale_text_entry(LocaleTag::DriveAuto, "Drive Auto", "Prowadzenie Samochodu"),
        locale_text_entry(LocaleTag::Edu, "EDU", "WYK"),
        locale_text_entry(LocaleTag::Education, "Education", "Wykształcenie"),
        locale_text_entry(LocaleTag::ElectricalRepair, "Electrical Repair", "Elektryka"),
        locale_text_entry(LocaleTag::ExtremeDamage, "extreme damage", "ekstremalne obrażenia"),
        locale_text_entry(
            LocaleTag::ExtremeSuccess,
            "⭐⭐⭐ Extreme Success",
            "⭐⭐⭐ Ekstremalny Sukces"
        ),
        locale_text_entry(LocaleTag::Failure, "❌ Failure", "❌ Porażka"),
        locale_text_entry(LocaleTag::FastTalk, "Fast Talk", "Gadanina"),
        locale_text_entry(LocaleTag::FightingBrawl, "Fighting (Brawl)", "Walka Wręcz (Bijatyka)"),
        locale_text_entry(LocaleTag::FirearmsHandgun, "Firearms (Handgun)", "Broń Palna (Krótka)"),
        locale_text_entry(
            LocaleTag::FirearmsRifleShotgun,
            "Firearms (Rifle/Shotgun)",
            "Broń Palna (Karabin/Strzelba)"
        ),
        locale_text_entry(LocaleTag::FirstAid, "First Aid", "Pierwsza Pomoc"),
        locale_text_entry(LocaleTag::GoneMad, "'s gone mad!", " ma atak szaleństwa!"),
        locale_text_entry(LocaleTag::HardSuccess, "⭐⭐ Hard Success", "⭐⭐ Trudny Sukces"),
        locale_text_entry(LocaleTag::History, "History", "Historia"),
        locale_text_entry(LocaleTag::HitPoints, "Hit Points", "Punkty Wytrzymałości"),
        locale_text_entry(LocaleTag::Hp, "HP", "PW"),
        locale_text_entry(LocaleTag::Impaling, "impaling", "ostra"),
        locale_text_entry(
            LocaleTag::IndefInsanity,
            "Indefinite insanity.",
            "Czasowa niepoczytalność."
        ),
        locale_text_entry(LocaleTag::Int, "INT", "INT"),
        locale_text_entry(LocaleTag::Intelligence, "Intelligence", "Inteligencja"),
        locale_text_entry(LocaleTag::Intimidate, "Intimidate", "Zastraszanie"),
        locale_text_entry(LocaleTag::Items, "Items", "Przedmioty"),
        locale_text_entry(LocaleTag::Jump, "Jump", "Skakanie"),
        locale_text_entry(LocaleTag::KnockOut, "Knock Out!", "Nokaut!"),
        locale_text_entry(LocaleTag::LanguageOther, "Language (other)", "Język (obcy)"),
        locale_text_entry(LocaleTag::LanguageOwn, "Language (own)", "Język (ojczysty)"),
        locale_text_entry(LocaleTag::Law, "Law", "Prawo"),
        locale_text_entry(LocaleTag::LibraryUse, "Library Use", "Korzystanie z Bibliotek"),
        locale_text_entry(LocaleTag::Listen, "Listen", "Nasłuchiwanie"),
        locale_text_entry(LocaleTag::Locksmith, "Locksmith", "Ślusarstwo"),
        locale_text_entry(LocaleTag::Luck, "Luck", "Szczęście"),
        locale_text_entry(LocaleTag::MajorWound, "Major wound!", "Ciężka rana!"),
        locale_text_entry(
            LocaleTag::MajorWoundHealed,
            "Major wound healed!",
            "Ciężka rana wyleczona!"
        ),
        locale_text_entry(LocaleTag::Malfunction, "Malfunction", "Zawodność"),
        locale_text_entry(
            LocaleTag::MaxSanitySet,
            "Set max sanity to",
            "Ustawiono maksymalną Poczytalność na"
        ),
        locale_text_entry(LocaleTag::MechanicalRepair, "Mechanical Repair", "Mechanika"),
        locale_text_entry(LocaleTag::Medicine, "Medicine", "Medycyna"),
        locale_text_entry(
            LocaleTag::MindHealed,
            "your mind healed enough to carry on...",
            "twój umysł wyzdrowiał wystarczająco, by ruszyć dalej..."
        ),
        locale_text_entry(
            LocaleTag::MindShattered,
            "Your mind has been irreversibly shattered.",
            "Twój umysł został nieodwracalnie strzaskany."
        ),
        locale_text_entry(LocaleTag::Move, "Move", "Ruch"),
        locale_text_entry(LocaleTag::Mp, "Magic Points", "Punkty Magii"),
        locale_text_entry(LocaleTag::NaturalWorld, "Natural World", "Wiedza o Naturze"),
        locale_text_entry(LocaleTag::Navigate, "Navigate", "Nawigacja"),
        locale_text_entry(LocaleTag::NoItems, "no items", "brak przedmiotów"),
        locale_text_entry(
            LocaleTag::NotMarked,
            "is not marked to be improved",
            "nie jest oznaczone do rozwinięcia"
        ),
        locale_text_entry(LocaleTag::Occult, "Occult", "Okultyzm"),
        locale_text_entry(
            LocaleTag::OperateHeavyMachinery,
            "Operate Heavy Machinery",
            "Obsługa Ciężkiego Sprzętu"
        ),
        locale_text_entry(LocaleTag::Penalty, "➖ Penalty", "➖ Karne"),
        locale_text_entry(LocaleTag::Persuade, "Persuade", "Perswazja"),
        locale_text_entry(LocaleTag::Pilot, "Pilot (any)", "Pilotowanie (dowolne)"),
        locale_text_entry(LocaleTag::PointsTo, "pts to", "pkt do"),
        locale_text_entry(LocaleTag::Pow, "POW", "MOC"),
        locale_text_entry(LocaleTag::Power, "Power", "Moc"),
        locale_text_entry(LocaleTag::Psychoanalysis, "Psychoanalysis", "Psychoanaliza"),
        locale_text_entry(LocaleTag::Psychology, "Psychology", "Psychologia"),
        locale_text_entry(LocaleTag::Range, "Range", "Zasięg"),
        locale_text_entry(LocaleTag::Ride, "Ride", "Jeździectwo"),
        locale_text_entry(
            LocaleTag::RollConCheckBlackOut,
            "Roll a **CON** test not to blackout.",
            "Rzuć test **KON**, aby nie stracić przytomności."
        ),
        locale_text_entry(
            LocaleTag::RollConCheckDie,
            "Roll a **CON** test not to **die**.\nYou'll be rolling this every round until someone helps you...",
            "Rzuć test **KON**, aby nie **umrzeć**.\nBędziesz rzucał co rundę dopóki ktoś Ci nie pomoże..."
        ),
        locale_text_entry(
            LocaleTag::RollIntCheck,
            "Roll an **INT** check if you **really** understood what just happened...",
            "Rzuć test **INT**, aby sprawdzić czy **naprawdę** pojąłeś co się właśnie stało..."
        ),
        locale_text_entry(LocaleTag::Rolls, "rolls", "rzuty"),
        locale_text_entry(LocaleTag::Sanity, "Sanity", "Poczytalność"),
        locale_text_entry(LocaleTag::ScienceAny, "Science (any)", "Nauka (dowolna)"),
        locale_text_entry(LocaleTag::Siz, "SIZ", "BC"),
        locale_text_entry(LocaleTag::Size, "Size", "Budowa Ciała"),
        locale_text_entry(LocaleTag::Skill, "Skill", "Umiejętność"),
        locale_text_entry(
            LocaleTag::SkillMarked,
            "Skill marked to improve.",
            "Umiejętność oznaczona do rozwinięcia."
        ),
        locale_text_entry(LocaleTag::SleightOfHand, "Sleight of Hand", "Zwinne Palce"),
        locale_text_entry(LocaleTag::Sp, "SP", "PP"),
        locale_text_entry(LocaleTag::SpotHidden, "Spot Hidden", "Spostrzegawczość"),
        locale_text_entry(LocaleTag::Stealth, "Stealth", "Ukrywanie"),
        locale_text_entry(LocaleTag::Str, "STR", "S"),
        locale_text_entry(LocaleTag::Strength, "Strength", "Siła"),
        locale_text_entry(LocaleTag::Success, "⭐ Success", "⭐ Sukces"),
        locale_text_entry(LocaleTag::SurvivalAny, "Survival (any)", "Sztuka Przetrwania (dowolna)"),
        locale_text_entry(LocaleTag::Swim, "Swim", "Pływanie"),
        locale_text_entry(LocaleTag::TempInsanity, "Temporal insanity!", "Atak szaleństwa!"),
        locale_text_entry(
            LocaleTag::TempInsanityThreat,
            "Temporal insanity threat!",
            "Ryzyko ataku szaleństwa!"
        ),
        locale_text_entry(LocaleTag::Throw, "Throw", "Rzucanie"),
        locale_text_entry(LocaleTag::Track, "Track", "Tropienie"),
        locale_text_entry(LocaleTag::Unarmed, "Unarmed", "Nieuzbrojony"),
        locale_text_entry(LocaleTag::Weapon, "Weapon", "Broń"),
        locale_text_entry(LocaleTag::WeaponJammed, "Weapon has jammed!", "Broń się zacięła!"),
        locale_text_entry(LocaleTag::Weapons, "Weapons", "Broń"),
        locale_text_entry(LocaleTag::YouBlackOut, "You black out", "Tracisz przytomność."),
        locale_text_entry(LocaleTag::YouFell, "You fell.", "Upadasz."),
    ];
    static ref LOCALE_TEXTS: LocaleTextMap = LOCALE_VEC.iter().cloned().collect();
    pub static ref DEFAULT_SKILLS: SkillValMap = vec![
        (LocaleTag::Accounting, 5),
        (LocaleTag::Anthropology, 1),
        (LocaleTag::Appraise, 5),
        (LocaleTag::Archeology, 1),
        (LocaleTag::ArtCraftAny, 5),
        (LocaleTag::Charm, 15),
        (LocaleTag::Climb, 20),
        (LocaleTag::Disguise, 5),
        (LocaleTag::Dodge, 0),
        (LocaleTag::DriveAuto, 20),
        (LocaleTag::ElectricalRepair, 10),
        (LocaleTag::FastTalk, 5),
        (LocaleTag::FightingBrawl, 25),
        (LocaleTag::FirearmsHandgun, 20),
        (LocaleTag::FirearmsRifleShotgun, 25),
        (LocaleTag::FirstAid, 30),
        (LocaleTag::History, 5),
        (LocaleTag::Intimidate, 15),
        (LocaleTag::Jump, 20),
        (LocaleTag::LanguageOther, 1),
        (LocaleTag::LanguageOwn, 0),
        (LocaleTag::Law, 5),
        (LocaleTag::LibraryUse, 20),
        (LocaleTag::Listen, 20),
        (LocaleTag::Locksmith, 1),
        (LocaleTag::MechanicalRepair, 10),
        (LocaleTag::Medicine, 1),
        (LocaleTag::NaturalWorld, 10),
        (LocaleTag::Navigate, 10),
        (LocaleTag::Occult, 5),
        (LocaleTag::Persuade, 10),
        (LocaleTag::Pilot, 1),
        (LocaleTag::Psychoanalysis, 1),
        (LocaleTag::Psychology, 10),
        (LocaleTag::Ride, 5),
        (LocaleTag::ScienceAny, 1),
        (LocaleTag::SleightOfHand, 10),
        (LocaleTag::SpotHidden, 25),
        (LocaleTag::Stealth, 20),
        (LocaleTag::SurvivalAny, 10),
        (LocaleTag::Swim, 10),
        (LocaleTag::Throw, 10),
        (LocaleTag::Track, 10),
    ]
    .into_iter()
    .collect();
    pub static ref DEFAULT_CONST_SKILLS: SkillValMap =
        vec![(LocaleTag::CreditRating, 0), (LocaleTag::CthulhuMythos, 0),]
            .into_iter()
            .collect();
}

pub fn locale_text(tag: &LocaleTag) -> Result<&LocaleText, Error> {
    Ok(LOCALE_TEXTS.get(tag).ok_or("Missing entry in LOCALE_TEXTS")?)
}

pub fn locale_text_lang(lang: &LocaleLang, tag: &LocaleTag) -> Result<String, Error> {
    Ok(locale_text(tag)?.get(lang))
}
