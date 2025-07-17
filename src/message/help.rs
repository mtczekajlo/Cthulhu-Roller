pub const CROLL_HELP: &str = r#"Bonus (`+`) and penalty (`-`) dice are being resolved automatically for easier adding circumstances of the roll, for example: test you firearms skill test of threshold `70`, you've been aiming entire previous round (bonus), target is really big (bonus) but moving fast (penalty) so you can roll `70++-`.

Syntax: `<threshold><optional modifier dice symbols>`

Examples: `30+` `20--` `50` `50+` `50-` `70++` `20++---`"#;

pub const ROLL_HELP: &str = r#"Syntax: `<optional number of dice>` `d/k` `<sides>` `<optional multiplier>` `<optional modifier>`

Examples: `2d4` `3k6` `24k6+10` `12d8x3` `4k12*2` `6d6x6+6` `d4-2` `k8+k4` `1k6+1k4+1`"#;

pub const IMPROVE_HELP: &str = r#"Syntax: `<threshold>`

Examples: `40` `60`"#;

pub const INITIATIVE_HELP: &str = r#"Initiative order is defined by: dexterity test success level, dexterity value and lowest roll value.

Bonus and penalty dice are being resolved automatically for easier adding circumstances of the roll, for example: you gain bonus die for initiative roll for being prepared (armed) at the beginning of fight. (see `/croll` command)

Syntax: `<character_name>` `<dexterity>` `<character_name>` `<dexterity>` (and so on)

Example: `/initiative Anna 50+ Brian 60- Celine 60 Douglas 70 Emma 50 Frank 50 George 50`"#;

pub const LEVELS_HELP: &str = r#"Syntax: `<threshold>`

Examples: `40` `60`"#;
