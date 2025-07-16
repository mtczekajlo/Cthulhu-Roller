pub const CROLL_HELP: &str = r#"Bonus (`+`) and penalty (`-`) dice are being resolved automatically for easier adding circumstances of the roll, for example: test you firearms skill test of threshold 70, you've been aiming entire previous round (bonus), target is really big (bonus) but moving fast (penalty) so you can roll `70++-`.

Syntax: `<threshold>` `<optional modifier dice symbols>...`

Examples:
`30+`, `20--`, `50`, `50+`, `50-`, `70++`, `20++---`

`/croll 60--+` results with:

```text
Success
38
Rolls: [ 30 ] [ 20 ] [ 8 ]
8 points to Hard Success
Threshold: 60 / 30 / 12
Penalty dice: 1
Query: "60--+"
```"#;

pub const ROLL_HELP: &str = r#"Syntax: `<optional number of dice>` `d/k` `<sides>` `<optional multiplier>` `<optional modifier>`

Examples:
`2d4`, `3k6`, `24k6+10`, `12d8x3`, `4k12*2`, `6d6x6+6`

`/roll 3d6x5+1` results with:
```
71
Rolls: ( [ 5 ] [ 6 ] [ 3 ] ) x 5 + 1
Query: "3d6x5+1"
```"#;

pub const IMPROVE_HELP: &str = r#"Syntax: `<threshold>`

`/improve 60` results with:

```text
Success
67
Threshold: 60
Query: "60"
```"#;

pub const INITIATIVE_HELP: &str = r#"Initiative order is defined by: dexterity test success level, dexterity value and lowest roll value.

Bonus and penalty dice are being resolved automatically for easier adding circumstances of the roll, for example: you gain bonus die for initiative roll for being prepared (armed) at the beginning of fight. (see `/croll` command)

Syntax: `<character_name> <dexterity> <character_name> <dexterity> ...`

For example `/initiative Anna 50 Brian 60 Celine 60 Douglas 70 Emma 50 Frank 50 George 50` results with:
```
Initiative
1. Douglas (Success) [Dex:70 Roll:62]
2. Celine (Success) [Dex:60 Roll:36]
3. Brian (Success) [Dex:60 Roll:43]
4. Emma (Success) [Dex:50 Roll:50]
5. Anna (Failure) [Dex:50 Roll:51]
6. Frank (Failure) [Dex:50 Roll:65]
7. George (Failure) [Dex:50 Roll:91]
Query: "Anna 50 Brian 60 Celine 60 Douglas 70 Emma 50 Frank 50 George 50"
```"#;

pub const LEVELS_HELP: &str = r#"Syntax: `<threshold>`

`/levels 50` results with:
```
50 / 25 / 10
Threshold: 50
Query: "50"
```"#;
