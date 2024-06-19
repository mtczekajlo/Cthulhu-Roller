# Cthulhu Roller

**Cthulhu Roller** is a Call of Cthulhu RPG 7E dice roller bot for Discord.

## Installation

[shuttle.rs quick start](<https://docs.shuttle.rs/getting-started/quick-start>)

## Usage

### Croll

Call of Cthulhu 7E skill test roller with bonus and penalty dice being resolved automatically for easier adding circumstances of the roll, for example: test you firearms skill test of threshold `70`, you've been aiming entire previous round (`bonus`), target is really big (`bonus`) but moving fast (`penalty`) so you can roll `70bbp`.

Syntax: `<threshold>` `<bonus die>/<penalty die>...`

Examples:

`50`, `50p`, `70bb`, `20bbppp`

```text
/croll 60ppb
```

![croll](docs/croll.png)

### Roll

Generic dice roller with multiplier and modifier (modifier is not multiplied).

Syntax: `<optional number of dice>` `d/k` `<sides>` `<optional multiplier>` `<optional modifier>`

Examples:

`2d4`, `3k6`, `24k6+10`, `12d8x3`, `6d6x6+6`

```text
/roll 3d6x5+1
```

![roll](docs/roll.png)
