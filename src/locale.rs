use crate::types::{LocaleEntryMap, LocaleVec};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub enum LocaleLang {
    #[default]
    English,
    Polski,
}

impl From<&str> for LocaleLang {
    fn from(value: &str) -> Self {
        match value.replace(' ', "").to_ascii_lowercase().as_str() {
            "pl" | "pol" | "polski" => Self::Polski,
            _ => Self::default(),
        }
    }
}

impl From<String> for LocaleLang {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for LocaleLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LocaleLang::Polski => "polski",
            LocaleLang::English => "english",
        })
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Debug, Default, PartialOrd, Ord)]
pub struct LocaleEntry {
    pub pl: String,
    pub en: String,
}

impl LocaleEntry {
    pub fn new(en: &str, pl: &str) -> Self {
        Self {
            pl: pl.into(),
            en: en.into(),
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn new_single_lang(text: &str) -> Self {
        Self {
            pl: text.into(),
            en: text.into(),
        }
    }

    pub fn get(&self, lang: LocaleLang) -> String {
        match lang {
            LocaleLang::Polski => self.pl.clone(),
            LocaleLang::English => self.en.clone(),
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn to_enum(&self) -> Option<LocaleTag> {
        LOCALE_TEXTS
            .iter()
            .find_map(|(tag, locale)| if self == locale { Some(*tag) } else { None })
    }

    #[cfg(feature = "character-sheet")]
    pub fn partial_match_ignore_case(&self, text: &str) -> bool {
        self.pl.to_ascii_lowercase().contains(&text.to_ascii_lowercase())
            || self.en.to_ascii_lowercase().contains(&text.to_ascii_lowercase())
    }

    pub fn equals_ignore_case(&self, text: &str) -> bool {
        self.pl.eq_ignore_ascii_case(text) || self.en.eq_ignore_ascii_case(text)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum LocaleTag {
    Accounting,
    Adventurer,
    Agony,
    Alert,
    Ammo,
    AnimalCompanion,
    AnimalHandling,
    Anthropology,
    App,
    Appearance,
    Appraise,
    ArcaneInsight,
    Archeology,
    ArtCraft,
    ArtCraftAny,
    Artillery,
    Attacks,
    BeadyEye,
    Beefcake,
    BodyHealed,
    BodyWounded,
    Bonus,
    BonusDieToFirstAction,
    BonVivant,
    Build,
    CantRemoveDefaultWeapon,
    CantSpendLuck,
    Characteristic,
    CharacterNotFound,
    Charm,
    Climb,
    ColdBlooded,
    ComesBackWithFullStrength,
    ComputerUse,
    Con,
    Constitution,
    CoreCharacteristic,
    CreditRating,
    CriticalFailure,
    CriticalSuccess,
    CthulhuMythos,
    Damage,
    DamageBonus,
    Db,
    DeathInevitable,
    Demolitions,
    Dex,
    Dexterity,
    Dice,
    Disguise,
    Diving,
    Dodge,
    Dreamer,
    DriveAuto,
    Edu,
    Education,
    Egghead,
    ElectricalRepair,
    Electronics,
    Endurance,
    Explorer,
    ExtremeDamage,
    ExtremeSuccess,
    Failure,
    FastLoad,
    FastTalk,
    FemmeFatale,
    Fight,
    FightEnd,
    Fighting,
    FightingBrawl,
    Firearms,
    FirearmsHandgun,
    FirearmsRifleShotgun,
    FirstAid,
    FleetFooted,
    From,
    Gadget,
    GoneMad,
    GreaseMonkey,
    Handy,
    HardBoiled,
    Hardened,
    HardSuccess,
    Harlequin,
    HeavyHitter,
    History,
    HitPoints,
    Hp,
    Hunter,
    Hypnosis,
    Impaling,
    InClip,
    IndefInsanity,
    Int,
    Intelligence,
    Intimidate,
    IronLiver,
    Item,
    Items,
    Jump,
    KeenHearing,
    KeenVision,
    KnockOut,
    LanguageAnyOther,
    LanguageOther,
    LanguageOwn,
    Law,
    LibraryUse,
    Linguist,
    Listen,
    Locksmith,
    Lore,
    LosesFirstRound,
    Luck,
    Lucky,
    MagicPoints,
    MajorWound,
    MajorWoundHealed,
    Malfunction,
    ManeuverBuildError,
    MasterOfDisguise,
    MaxSanitySet,
    MechanicalRepair,
    Medicine,
    MindHealed,
    MindShattered,
    Move,
    Mp,
    Mystic,
    MythosKnowledge,
    Name,
    NaturalWorld,
    Navigate,
    NightVision,
    Nimble,
    NoCharacters,
    NoCharacterSelected,
    NoItems,
    NoSuchItem,
    NoSuchSkill,
    NoSuchWeapon,
    NotMarked,
    Occult,
    OperateHeavyMachinery,
    Outmaneuver,
    Outsider,
    Pcs,
    Penalty,
    Persuade,
    PhotographicMemory,
    Pilot,
    PilotAny,
    PointsTo,
    Pow,
    Power,
    PowerLifter,
    PrepareForTheConsequences,
    PsychicPower,
    Psychoanalysis,
    Psychology,
    PulpArchetype,
    PulpTalents,
    PushRoll,
    QuickDraw,
    QuickHealer,
    QuickStudy,
    Range,
    RapidAttack,
    RapidFire,
    ReadLips,
    Received,
    Reloaded,
    Resilient,
    Resourceful,
    Result,
    Results,
    Ride,
    Rogue,
    RollConCheckBlackOut,
    RollConCheckDie,
    RollIntCheck,
    Rolls,
    Rounds,
    Sanity,
    Scary,
    Scholar,
    Science,
    ScienceAny,
    Seeker,
    SetLanguageTo,
    Shadow,
    SharpWitted,
    Sidekick,
    Siz,
    Size,
    Skill,
    SkillMarked,
    SkillUnmarked,
    SleightOfHand,
    SmoothTalker,
    SorryTooManyCharacters,
    Sp,
    SpotHidden,
    Steadfast,
    Stealth,
    StoutConstitution,
    Str,
    Strength,
    StrongWilled,
    Success,
    Survival,
    SurvivalAny,
    Swashbuckler,
    Swim,
    TempInsanity,
    TempInsanityThreat,
    ThrillSeeker,
    Throw,
    ToughGuy,
    Track,
    TwoFisted,
    Unarmed,
    Value,
    Weapon,
    WeaponJammed,
    Weapons,
    WeirdScience,
    YouBlackOut,
    YouFell,
    YouGotLuckyThisTIme,
}

pub fn locale_entry(tag: LocaleTag, en: &'static str, pl: &'static str) -> (LocaleTag, LocaleEntry) {
    (tag, LocaleEntry::new(en, pl))
}

lazy_static! {
    pub static ref LOCALE_ATTRIBUTES: LocaleVec = vec![
        locale_entry(LocaleTag::Appearance, "Appearance", "Wygląd"),
        locale_entry(LocaleTag::Constitution, "Constitution", "Kondycja"),
        locale_entry(LocaleTag::Dexterity, "Dexterity", "Zręczność"),
        locale_entry(LocaleTag::Education, "Education", "Wykształcenie"),
        locale_entry(LocaleTag::Intelligence, "Intelligence", "Inteligencja"),
        locale_entry(LocaleTag::Power, "Power", "Moc"),
        locale_entry(LocaleTag::Size, "Size", "Budowa Ciała"),
        locale_entry(LocaleTag::Strength, "Strength", "Siła"),
    ];
    static ref LOCALE_ATTRIBUTES_SHORT: LocaleVec = vec![
        locale_entry(LocaleTag::App, "APP", "WYG"),
        locale_entry(LocaleTag::Con, "CON", "KON"),
        locale_entry(LocaleTag::Dex, "DEX", "ZR"),
        locale_entry(LocaleTag::Edu, "EDU", "WYK"),
        locale_entry(LocaleTag::Int, "INT", "INT"),
        locale_entry(LocaleTag::Pow, "POW", "MOC"),
        locale_entry(LocaleTag::Siz, "SIZ", "BC"),
        locale_entry(LocaleTag::Str, "STR", "S"),
    ];
    static ref LOCALE_CHARACTER_STATS: LocaleVec = vec![
        locale_entry(LocaleTag::Build, "Build", "Krzepa"),
        locale_entry(LocaleTag::DamageBonus, "Damage Bonus", "Modyfikator Obrażeń"),
        locale_entry(LocaleTag::Db, "DB", "MO"),
        locale_entry(LocaleTag::HitPoints, "Hit Points", "Punkty Wytrzymałości"),
        locale_entry(LocaleTag::Hp, "HP", "PW"),
        locale_entry(LocaleTag::Luck, "Luck", "Szczęście"),
        locale_entry(LocaleTag::MagicPoints, "Magic Points", "Punkty Magii"),
        locale_entry(LocaleTag::Move, "Move", "Ruch"),
        locale_entry(LocaleTag::Mp, "MP", "PM"),
        locale_entry(LocaleTag::Sanity, "Sanity", "Poczytalność"),
        locale_entry(LocaleTag::Skill, "Skill", "Umiejętność"),
        locale_entry(LocaleTag::Sp, "SP", "PP"),
    ];
    static ref LOCALE_SKILLS: LocaleVec = vec![
        locale_entry(LocaleTag::Accounting, "Accounting", "Księgowość"),
        locale_entry(LocaleTag::AnimalHandling, "Animal Handling", "Tresura Zwierząt"),
        locale_entry(LocaleTag::Anthropology, "Anthropology", "Antropologia"),
        locale_entry(LocaleTag::Appraise, "Appraise", "Wycena"),
        locale_entry(LocaleTag::Archeology, "Archeology", "Archeologia"),
        locale_entry(LocaleTag::ArtCraft, "Art/Craft", "Sztuka/Rzemiosło"),
        locale_entry(LocaleTag::ArtCraftAny, "Art/Craft (any)", "Sztuka/Rzemiosło (dowolne)"),
        locale_entry(LocaleTag::Artillery, "Artillery", "Broń Artyleryjska"),
        locale_entry(LocaleTag::Charm, "Charm", "Urok Osobisty"),
        locale_entry(LocaleTag::Climb, "Climb", "Wspinaczka"),
        locale_entry(LocaleTag::ComputerUse, "Computer Use", "Korzystanie z Komputerów"),
        locale_entry(LocaleTag::CreditRating, "Credit Rating", "Majętność"),
        locale_entry(LocaleTag::CthulhuMythos, "Cthulhu Mythos", "Mity Cthulhu"),
        locale_entry(LocaleTag::Demolitions, "Demolitions", "Materiały Wybuchowe"),
        locale_entry(LocaleTag::Disguise, "Disguise", "Charakteryzacja"),
        locale_entry(LocaleTag::Diving, "Diving", "Nurkowanie"),
        locale_entry(LocaleTag::Dodge, "Dodge", "Unik"),
        locale_entry(LocaleTag::DriveAuto, "Drive Auto", "Prowadzenie Samochodu"),
        locale_entry(LocaleTag::ElectricalRepair, "Electrical Repair", "Elektryka"),
        locale_entry(LocaleTag::Electronics, "Electronics", "Elektronika"),
        locale_entry(LocaleTag::FastTalk, "Fast Talk", "Gadanina"),
        locale_entry(LocaleTag::Fighting, "Fighting", "Walka Wręcz"),
        locale_entry(LocaleTag::FightingBrawl, "Fighting (Brawl)", "Walka Wręcz (Bijatyka)"),
        locale_entry(LocaleTag::Firearms, "Firearms", "Broń Palna"),
        locale_entry(LocaleTag::FirearmsHandgun, "Firearms (Handgun)", "Broń Palna (Krótka)"),
        locale_entry(
            LocaleTag::FirearmsRifleShotgun,
            "Firearms (Rifle/Shotgun)",
            "Broń Palna (Karabin/Strzelba)"
        ),
        locale_entry(LocaleTag::FirstAid, "First Aid", "Pierwsza Pomoc"),
        locale_entry(LocaleTag::History, "History", "Historia"),
        locale_entry(LocaleTag::Hypnosis, "Hypnosis", "Hipnoza"),
        locale_entry(LocaleTag::Intimidate, "Intimidate", "Zastraszanie"),
        locale_entry(LocaleTag::Jump, "Jump", "Skakanie"),
        locale_entry(
            LocaleTag::LanguageAnyOther,
            "Language (any other)",
            "Język Obcy (dowolny)"
        ),
        locale_entry(LocaleTag::LanguageOther, "Language (other)", "Język Obcy"),
        locale_entry(LocaleTag::LanguageOwn, "Language (own)", "Język Ojczysty"),
        locale_entry(LocaleTag::Law, "Law", "Prawo"),
        locale_entry(LocaleTag::LibraryUse, "Library Use", "Korzystanie z Bibliotek"),
        locale_entry(LocaleTag::Listen, "Listen", "Nasłuchiwanie"),
        locale_entry(LocaleTag::Locksmith, "Locksmith", "Ślusarstwo"),
        locale_entry(LocaleTag::MechanicalRepair, "Mechanical Repair", "Mechanika"),
        locale_entry(LocaleTag::Medicine, "Medicine", "Medycyna"),
        locale_entry(LocaleTag::NaturalWorld, "Natural World", "Wiedza o Naturze"),
        locale_entry(LocaleTag::Navigate, "Navigate", "Nawigacja"),
        locale_entry(LocaleTag::Occult, "Occult", "Okultyzm"),
        locale_entry(
            LocaleTag::OperateHeavyMachinery,
            "Operate Heavy Machinery",
            "Obsługa Ciężkiego Sprzętu"
        ),
        locale_entry(LocaleTag::Persuade, "Persuade", "Perswazja"),
        locale_entry(LocaleTag::Pilot, "Pilot", "Pilotowanie"),
        locale_entry(LocaleTag::PilotAny, "Pilot (any)", "Pilotowanie (dowolne)"),
        locale_entry(LocaleTag::Psychoanalysis, "Psychoanalysis", "Psychoanaliza"),
        locale_entry(LocaleTag::Psychology, "Psychology", "Psychologia"),
        locale_entry(LocaleTag::ReadLips, "Read Lips", "Czytanie z Ruchu Warg"),
        locale_entry(LocaleTag::Ride, "Ride", "Jeździectwo"),
        locale_entry(LocaleTag::Science, "Science", "Nauka"),
        locale_entry(LocaleTag::ScienceAny, "Science (any)", "Nauka (dowolna)"),
        locale_entry(LocaleTag::SleightOfHand, "Sleight of Hand", "Zręczne Palce"),
        locale_entry(LocaleTag::SpotHidden, "Spot Hidden", "Spostrzegawczość"),
        locale_entry(LocaleTag::Stealth, "Stealth", "Ukrywanie"),
        locale_entry(LocaleTag::Survival, "Survival", "Sztuka Przetrwania"),
        locale_entry(LocaleTag::SurvivalAny, "Survival (any)", "Sztuka Przetrwania (dowolna)"),
        locale_entry(LocaleTag::Swim, "Swim", "Pływanie"),
        locale_entry(LocaleTag::Throw, "Throw", "Rzucanie"),
        locale_entry(LocaleTag::Track, "Track", "Tropienie"),
    ];
    pub static ref LOCALE_PULP_ARCHETYPES: LocaleVec = vec![
        locale_entry(LocaleTag::Adventurer, "Adventurer", "Awanturnik"),
        locale_entry(LocaleTag::Beefcake, "Beefcake", "Osiłek"),
        locale_entry(LocaleTag::BonVivant, "Bon Vivant", "Bon Vivant"),
        locale_entry(LocaleTag::ColdBlooded, "Cold Blooded", "Bezwzględny"),
        locale_entry(LocaleTag::Dreamer, "Dreamer", "Marzyciel"),
        locale_entry(LocaleTag::Egghead, "Egghead", "Mózgowiec"),
        locale_entry(LocaleTag::Explorer, "Explorer", "Odkrywca"),
        locale_entry(LocaleTag::FemmeFatale, "Femme Fatale", "Femme Fatale"),
        locale_entry(LocaleTag::GreaseMonkey, "Grease Monkey", "Złota Rączka"),
        locale_entry(LocaleTag::HardBoiled, "Hard Boiled", "Twarda Sztuka"),
        locale_entry(LocaleTag::Harlequin, "Harlequin", "Harlequin"),
        locale_entry(LocaleTag::Hunter, "Hunter", "Łowca"),
        locale_entry(LocaleTag::Mystic, "Mystic", "Mistyk"),
        locale_entry(LocaleTag::Outsider, "Outsider", "Autsajder"),
        locale_entry(LocaleTag::Rogue, "Rogue", "Buntownik"),
        locale_entry(LocaleTag::Scholar, "Scholar", "Uczony"),
        locale_entry(LocaleTag::Seeker, "Seeker", "Poszukiwacz Prawdy"),
        locale_entry(LocaleTag::Sidekick, "Sidekick", "Pomagier"),
        locale_entry(LocaleTag::Steadfast, "Steadfast", "Bojownik o Sprawę"),
        locale_entry(LocaleTag::Swashbuckler, "Swashbuckler", "Zawadiaka"),
        locale_entry(LocaleTag::ThrillSeeker, "Thrill Seeker", "Śmiałek"),
        locale_entry(LocaleTag::TwoFisted, "Two-Fisted", "Zabijaka"),
    ];
    pub static ref LOCALE_PULP_TALENTS: LocaleVec = vec![
        locale_entry(LocaleTag::Alert, "Alert", "Czujność"),
        locale_entry(LocaleTag::AnimalCompanion, "Animal Companion", "Zwierzęcy Towarzysz"),
        locale_entry(LocaleTag::ArcaneInsight, "Arcane Insight", "Magiczna Intuicja"),
        locale_entry(LocaleTag::BeadyEye, "Beady Eye", "Celne Oko"),
        locale_entry(LocaleTag::Endurance, "Endurance", "Żelazna Kondycja"),
        locale_entry(LocaleTag::FastLoad, "Fast Load", "Szybkie Przeładowanie"),
        locale_entry(LocaleTag::FleetFooted, "Fleet Footed", "Szybkonogi"),
        locale_entry(LocaleTag::Gadget, "Gadget", "Gadżet"),
        locale_entry(LocaleTag::Handy, "Handy", "Majsterkowicz"),
        locale_entry(LocaleTag::Hardened, "Hardened", "Hart Ducha"),
        locale_entry(LocaleTag::HeavyHitter, "Heavy Hitter", "Ciężka Ręka"),
        locale_entry(LocaleTag::IronLiver, "Iron Liver", "Mocna Głowa"),
        locale_entry(LocaleTag::KeenHearing, "Keen Hearing", "Czuły Słuch"),
        locale_entry(LocaleTag::KeenVision, "Keen Vision", "Bystry Wzrok"),
        locale_entry(LocaleTag::Linguist, "Linguist", "Lingwista"),
        locale_entry(LocaleTag::Lore, "Lore", "Wiedza Tajemna"),
        locale_entry(LocaleTag::Lucky, "Lucky", "Szczęściarz"),
        locale_entry(LocaleTag::MasterOfDisguise, "Master of Disguise", "Mistrz Kamuflażu"),
        locale_entry(LocaleTag::MythosKnowledge, "Mythos Knowledge", "Znajomość Mitów"),
        locale_entry(LocaleTag::NightVision, "Night Vision", "Widzenie w Ciemności"),
        locale_entry(LocaleTag::Nimble, "Nimble", "Niezwykła Zwinność"),
        locale_entry(LocaleTag::Outmaneuver, "Outmaneuver", "Pewna Postawa"),
        locale_entry(
            LocaleTag::PhotographicMemory,
            "Photographic Memory",
            "Fotograficzna Pamięć"
        ),
        locale_entry(LocaleTag::PowerLifter, "Power Lifter", "Ciężarowiec"),
        locale_entry(LocaleTag::PsychicPower, "Psychic Power", "Moc Parapsychiczna"),
        locale_entry(LocaleTag::QuickDraw, "Quick Draw", "Szybkie Dobywanie"),
        locale_entry(LocaleTag::QuickHealer, "Quick Healer", "Szybka Regeneracja"),
        locale_entry(LocaleTag::QuickStudy, "Quick Study", "Pojętny Uczeń"),
        locale_entry(LocaleTag::RapidAttack, "Rapid Attack", "Szybki Atak"),
        locale_entry(LocaleTag::RapidFire, "Rapid Fire", "Rewolwerowiec"),
        locale_entry(LocaleTag::Resilient, "Resilient", "Nerwy ze Stali"),
        locale_entry(LocaleTag::Resourceful, "Resourceful", "Zaradność"),
        locale_entry(LocaleTag::Scary, "Scary", "Zakapior"),
        locale_entry(LocaleTag::Shadow, "Shadow", "Cień"),
        locale_entry(LocaleTag::SharpWitted, "Sharp Witted", "Bystry Umysł"),
        locale_entry(LocaleTag::SmoothTalker, "Smooth Talker", "Bajerant"),
        locale_entry(LocaleTag::StoutConstitution, "Stout Constitution", "Zahartowany"),
        locale_entry(LocaleTag::StrongWilled, "Strong Willed", "Silna Wola"),
        locale_entry(LocaleTag::ToughGuy, "Tough Guy", "Twardziel"),
        locale_entry(LocaleTag::WeirdScience, "Weird Science", "Szalona Nauka"),
    ];
    static ref LOCALE_VEC: LocaleVec = vec![
        locale_entry(LocaleTag::Agony, "Agony!", "Agonia!"),
        locale_entry(LocaleTag::Ammo, "Ammo", "Amunicja"),
        locale_entry(LocaleTag::Attacks, "Attacks", "Ataki"),
        locale_entry(
            LocaleTag::BodyHealed,
            "your body healed enough to carry on...",
            "twoje ciało wyzdrowiało wystarczająco, by ruszyć dalej..."
        ),
        locale_entry(
            LocaleTag::BodyWounded,
            "your body has been heavily wounded...",
            "twoje ciało zostało ciężko zranione..."
        ),
        locale_entry(LocaleTag::Bonus, "➕ Bonus", "➕ Premiowe"),
        locale_entry(
            LocaleTag::BonusDieToFirstAction,
            "bonus die to first action",
            "kość premiowa do pierwszej akcji"
        ),
        locale_entry(
            LocaleTag::CantRemoveDefaultWeapon,
            "Can't remove default weapon.",
            "Nie można usunąć domyślnej broni."
        ),
        locale_entry(
            LocaleTag::CantSpendLuck,
            "Not enough Luck points!",
            "Niewystarczająca ilość punktów Szczęścia!"
        ),
        locale_entry(LocaleTag::Characteristic, "Characteristic", "Cecha"),
        locale_entry(
            LocaleTag::CharacterNotFound,
            "No such character",
            "Nie ma takiej postaci"
        ),
        locale_entry(
            LocaleTag::ComesBackWithFullStrength,
            "Comes back with full strength. 💪",
            "Powraca w pełni sił. 💪"
        ),
        locale_entry(LocaleTag::CoreCharacteristic, "Core Characteristic", "Cecha Podstawowa"),
        locale_entry(
            LocaleTag::CriticalFailure,
            "🐙🐙🐙 CRITICAL FAILURE 🐙🐙🐙",
            "🐙🐙🐙 KRYTYCZNA PORAŻKA 🐙🐙🐙"
        ),
        locale_entry(
            LocaleTag::CriticalSuccess,
            "✨✨✨ CRITICAL SUCCESS ✨✨✨",
            "✨✨✨ KRYTYCZNY SUKCES ✨✨✨"
        ),
        locale_entry(LocaleTag::Damage, "Damage", "Obrażenia"),
        locale_entry(
            LocaleTag::DeathInevitable,
            "Death is inevitable.",
            "Śmierć jest nieunikniona."
        ),
        locale_entry(LocaleTag::Dice, "dice", "kości"),
        locale_entry(LocaleTag::ExtremeDamage, "extreme damage", "ekstremalne obrażenia"),
        locale_entry(
            LocaleTag::ExtremeSuccess,
            "⭐⭐⭐ Extreme Success",
            "⭐⭐⭐ Ekstremalny Sukces"
        ),
        locale_entry(LocaleTag::Failure, "❌ Failure", "❌ Porażka"),
        locale_entry(LocaleTag::Fight, "Fight", "Walka"),
        locale_entry(LocaleTag::FightEnd, "The end of the fight! 🎉", "Koniec walki! 🎉"),
        locale_entry(LocaleTag::From, "from", "od"),
        locale_entry(LocaleTag::GoneMad, "has gone mad!", "ma atak szaleństwa!"),
        locale_entry(LocaleTag::HardSuccess, "⭐⭐ Hard Success", "⭐⭐ Trudny Sukces"),
        locale_entry(LocaleTag::Impaling, "impaling", "ostra"),
        locale_entry(LocaleTag::InClip, "in clip", "w magazynku"),
        locale_entry(
            LocaleTag::IndefInsanity,
            "Indefinite insanity.",
            "Czasowa niepoczytalność."
        ),
        locale_entry(LocaleTag::Item, "item", "przedmiot"),
        locale_entry(LocaleTag::Items, "Items", "Przedmioty"),
        locale_entry(LocaleTag::KnockOut, "Knock Out!", "Nokaut!"),
        locale_entry(LocaleTag::LosesFirstRound, "loses first round", "traci pierwszą turę"),
        locale_entry(LocaleTag::MajorWound, "Major wound!", "Ciężka rana!"),
        locale_entry(
            LocaleTag::MajorWoundHealed,
            "Major wound healed!",
            "Ciężka rana wyleczona!"
        ),
        locale_entry(LocaleTag::Malfunction, "Malfunction", "Zawodność"),
        locale_entry(
            LocaleTag::ManeuverBuildError,
            "Build difference is 3 or more! Maneuver is impossible.",
            "Różnica Krzepy wynosi 3 lub więcej! Manewr niemożliwy."
        ),
        locale_entry(
            LocaleTag::MaxSanitySet,
            "Set max sanity to",
            "Ustawiono maksymalną Poczytalność na"
        ),
        locale_entry(
            LocaleTag::MindHealed,
            "your mind healed enough to carry on...",
            "twój umysł wyzdrowiał wystarczająco, by ruszyć dalej..."
        ),
        locale_entry(
            LocaleTag::MindShattered,
            "Your mind has been irreversibly shattered.",
            "Twój umysł został nieodwracalnie strzaskany."
        ),
        locale_entry(LocaleTag::Name, "Name", "Nazwa"),
        locale_entry(
            LocaleTag::NoCharacters,
            "You have no characters yet.",
            "Nie masz jeszcze żadnej postaci."
        ),
        locale_entry(
            LocaleTag::NoCharacterSelected,
            "No character selected",
            "Nie wybrano żadnej postaci"
        ),
        locale_entry(LocaleTag::NoItems, "no items", "brak przedmiotów"),
        locale_entry(LocaleTag::NoSuchItem, "has no such item", "nie ma takiego przedmiotu"),
        locale_entry(LocaleTag::NoSuchWeapon, "has no such weapon", "nie ma takiej broni"),
        locale_entry(
            LocaleTag::NotMarked,
            "is not marked to be improved",
            "nie jest oznaczone do rozwinięcia"
        ),
        locale_entry(LocaleTag::Pcs, "pcs", "szt"),
        locale_entry(LocaleTag::Penalty, "➖ Penalty", "➖ Karne"),
        locale_entry(LocaleTag::PointsTo, "pts to", "pkt do"),
        locale_entry(
            LocaleTag::PrepareForTheConsequences,
            "Prepare for the consequences...",
            "Przygotuj się na konsekwencje..."
        ),
        locale_entry(LocaleTag::PulpArchetype, "Pulp Archetype", "Pulpowy Archetyp"),
        locale_entry(LocaleTag::PulpTalents, "Pulp Talents", "Pulpowe Talenty"),
        locale_entry(LocaleTag::PushRoll, "🥊 Push", "🥊 Forsuj"),
        locale_entry(LocaleTag::Range, "Range", "Zasięg"),
        locale_entry(LocaleTag::Received, "received", "otrzymano"),
        locale_entry(LocaleTag::Reloaded, "Reloaded", "Przeładowano"),
        locale_entry(LocaleTag::Result, "Result", "Wynik"),
        locale_entry(LocaleTag::Results, "Results", "Wyniki"),
        locale_entry(
            LocaleTag::RollConCheckBlackOut,
            "Roll a **CON** test not to blackout.",
            "Rzuć test **KON**, aby nie stracić przytomności."
        ),
        locale_entry(
            LocaleTag::RollConCheckDie,
            "Roll a **CON** test not to **die**.\nYou'll be rolling this every round until someone helps you...",
            "Rzuć test **KON**, aby nie **umrzeć**.\nBędziesz rzucał co rundę dopóki ktoś Ci nie pomoże..."
        ),
        locale_entry(
            LocaleTag::RollIntCheck,
            "Roll an **INT** check if you **really** understood what just happened...",
            "Rzuć test **INT**, aby sprawdzić czy **naprawdę** pojąłeś co się właśnie stało..."
        ),
        locale_entry(LocaleTag::Rolls, "🎲", "🎲"),
        locale_entry(LocaleTag::Rounds, "rounds", "pocisków"),
        locale_entry(LocaleTag::SetLanguageTo, "Set language to", "Ustawiono język na"),
        locale_entry(
            LocaleTag::SkillMarked,
            "Skill marked to improve.",
            "Umiejętność oznaczona do rozwinięcia."
        ),
        locale_entry(
            LocaleTag::SkillUnmarked,
            "Skill unmarked from improve.",
            "Umiejętność odznaczona z rozwinięcia."
        ),
        locale_entry(
            LocaleTag::SorryTooManyCharacters,
            "Sorry, you have too many characters already.",
            "Wybacz, masz już za dużo postaci."
        ),
        locale_entry(LocaleTag::Success, "⭐ Success", "⭐ Sukces"),
        locale_entry(LocaleTag::TempInsanity, "Temporal insanity!", "Atak szaleństwa!"),
        locale_entry(
            LocaleTag::TempInsanityThreat,
            "Temporal insanity threat!",
            "Ryzyko ataku szaleństwa!"
        ),
        locale_entry(LocaleTag::Unarmed, "Unarmed", "Nieuzbrojony"),
        locale_entry(LocaleTag::Value, "Value", "Wartość"),
        locale_entry(LocaleTag::Weapon, "weapon", "broń"),
        locale_entry(LocaleTag::WeaponJammed, "Weapon has jammed!", "Broń się zacięła!"),
        locale_entry(LocaleTag::Weapons, "Weapons", "Broń"),
        locale_entry(LocaleTag::YouBlackOut, "You black out", "Tracisz przytomność."),
        locale_entry(LocaleTag::YouFell, "You fell.", "Upadasz."),
        locale_entry(
            LocaleTag::YouGotLuckyThisTIme,
            "You got lucky this time...",
            "Tym razem ci się poszczęściło..."
        ),
    ]
    .into_iter()
    .chain(LOCALE_ATTRIBUTES_SHORT.clone().into_iter())
    .chain(LOCALE_ATTRIBUTES.clone().into_iter())
    .chain(LOCALE_CHARACTER_STATS.clone().into_iter())
    .chain(LOCALE_PULP_ARCHETYPES.clone().into_iter())
    .chain(LOCALE_PULP_TALENTS.clone().into_iter())
    .chain(LOCALE_SKILLS.clone().into_iter())
    .collect();
    static ref LOCALE_TEXTS: LocaleEntryMap = LOCALE_VEC.iter().cloned().collect();
}

pub fn locale_entry_by_tag<'a>(tag: LocaleTag) -> &'a LocaleEntry {
    LOCALE_TEXTS
        .get(&tag)
        .unwrap_or_else(|| panic!("Missing LOCALE_TEXTS entry for tag: {tag:?}"))
}

#[cfg(feature = "character-sheet")]
pub fn locale_entry_by_str(string: &str) -> Option<&LocaleEntry> {
    let find_result = LOCALE_TEXTS.iter().find(|(_, text)| text.equals_ignore_case(string));
    if let Some(pair) = find_result {
        Some(pair.1)
    } else {
        None
    }
}

pub fn locale_tag_by_str(string: &str) -> Option<LocaleTag> {
    let find_result = LOCALE_TEXTS.iter().find(|(_, text)| text.equals_ignore_case(string));
    find_result.map(|pair| *pair.0)
}

pub fn locale_text_by_tag_lang(lang: LocaleLang, tag: LocaleTag) -> String {
    locale_entry_by_tag(tag).get(lang)
}
