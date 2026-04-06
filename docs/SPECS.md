# ROGUE: DUNGEON EXPLORATION GAME

Behavioral Specification v1.0 — D. Olmisani — April 2026

---

## Abstract

This document specifies the behavior of the Rogue dungeon exploration game as derived from the
rogue-libc5-ncurses reference implementation (Rogue 5.3-clone, patchlevel 1, ported by Alan Cox).
It is intended to serve as the authoritative behavioral reference for re-implementations, including
the Rusted Rogue Rust port.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT",
"RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in
RFC 2119.

## Table of Contents

1. [Introduction](#1-introduction)
2. [Display and Coordinate System](#2-display-and-coordinate-system)
3. [Player Character](#3-player-character)
4. [Game Loop and Turn Structure](#4-game-loop-and-turn-structure)
5. [Dungeon Generation](#5-dungeon-generation)
   - [5.1. Room Layout](#51-room-layout)
   - [5.2. Room Types](#52-room-types)
   - [5.3. Passages and Connections](#53-passages-and-connections)
   - [5.4. Mazes](#54-mazes)
   - [5.5. Level Content Placement](#55-level-content-placement)
6. [Movement and Navigation](#6-movement-and-navigation)
   - [6.1. Single Move](#61-single-move)
   - [6.2. Run Mode](#62-run-mode)
   - [6.3. Visibility](#63-visibility)
7. [Combat System](#7-combat-system)
   - [7.1. Melee Combat](#71-melee-combat)
   - [7.2. Ranged Combat (Throwing)](#72-ranged-combat-throwing)
   - [7.3. Special Monster Attacks](#73-special-monster-attacks)
   - [7.4. Death and Victory Conditions](#74-death-and-victory-conditions)
8. [Items](#8-items)
   - [8.1. Weapons](#81-weapons)
   - [8.2. Armor](#82-armor)
   - [8.3. Potions](#83-potions)
   - [8.4. Scrolls](#84-scrolls)
   - [8.5. Wands / Staves](#85-wands--staves)
   - [8.6. Rings](#86-rings)
   - [8.7. Food](#87-food)
   - [8.8. Gold](#88-gold)
   - [8.9. Amulet of Yendor](#89-amulet-of-yendor)
9. [Inventory and Pack](#9-inventory-and-pack)
10. [Monsters](#10-monsters)
    - [10.1. Monster Roster](#101-monster-roster)
    - [10.2. Monster Flags](#102-monster-flags)
    - [10.3. Monster AI](#103-monster-ai)
    - [10.4. Special Monster Behaviors](#104-special-monster-behaviors)
11. [Traps](#11-traps)
12. [Status Effects](#12-status-effects)
13. [Hunger System](#13-hunger-system)
14. [Experience and Leveling](#14-experience-and-leveling)
15. [Scoring](#15-scoring)
16. [Save and Restore](#16-save-and-restore)
17. [Command Reference](#17-command-reference)
18. [Options (ROGUEOPTS)](#18-options-rogueopts)
19. [Constants Reference](#19-constants-reference)
20. [Wizard Mode](#20-wizard-mode)

---

## 1. Introduction

Rogue is a turn-based, single-player dungeon exploration game displayed on a text terminal. The
player controls a character represented by the glyph `@` who must descend through a randomly
generated dungeon, retrieve the Amulet of Yendor from dungeon level 26, and return to the surface.

The game ends in one of three ways:

1. The player character's hit points reach zero (death).
2. The player quits voluntarily (death with `Q`).
3. The player returns to the surface carrying the Amulet (victory).

At game end the player's gold total is recorded in a shared top-ten score file.

## 2. Display and Coordinate System

The display MUST occupy a terminal of at least 24 rows by 80 columns (DROWS=24, DCOLS=80). Row 0
is the message line. Rows 1 through 22 are the dungeon play area. Row 23 is the status bar.

**Message paging.** After each turn, one or more messages may be generated (hit results, miss
results, inventory events, system notices).  All queued messages are shown one at a time on
row 0.  While more than one message is waiting, the current message is followed by an
inverted-colour `--More--` prompt; all game input is suspended until the player presses
`Space` to advance to the next message.  When the queue contains only one (or zero) messages
no prompt is shown and input is accepted normally.

Coordinate origin (0,0) is at the top-left corner of the screen. Rows increase downward; columns
increase rightward.

The minimum usable row for the dungeon is MIN_ROW = 1.

Tile glyphs on the dungeon layer are (in bitfield priority order):

| Glyph | Description                                       |
|-------|---------------------------------------------------|
| `@`   | Player character                                  |
| A–Z   | Monster (uppercase letter unique to monster type) |
| `!`   | Potion                                            |
| `?`   | Scroll                                            |
| `)`   | Weapon                                            |
| `]`   | Armor                                             |
| `/`   | Wand or staff                                     |
| `=`   | Ring                                              |
| `%`   | Stairs                                            |
| `*`   | Gold pile                                         |
| `:`   | Food                                              |
| `,`   | Amulet of Yendor                                  |
| `^`   | Trap (visible)                                    |
| `+`   | Door                                              |
| `#`   | Tunnel / passage floor                            |
| `.`   | Room floor                                        |
| `-`   | Horizontal wall                                   |
| `\|`  | Vertical wall                                     |

## 3. Player Character

The player character is represented by the `fighter` structure.

### 3.1. Initial Statistics

| Attribute          | Initial Value | Maximum Value |
|--------------------|---------------|---------------|
| HP (current)       | 12            | 800           |
| HP (max)           | 12            | 800           |
| Strength (current) | 16            | 99            |
| Strength (max)     | 16            | 99            |
| Gold               | 0             | 900000        |
| Experience Level   | 1             | 21            |
| Experience Points  | 0             | 10000000      |
| Moves remaining    | 1250          | —             |

### 3.2. Starting Equipment

The player MUST begin each new game with the following items in the pack (all pre-identified):

1. One ration of food.
2. Ring mail +1 (enchant), worn.
3. Mace, damage "2d3", hit-enchant +1, d-enchant +1, wielded.
4. Bow, damage "1d2", hit-enchant +1, d-enchant 0.
5. 25–35 arrows (random), damage "1d2", unenchanted.

## 4. Game Loop and Turn Structure

The game operates on a turn-by-turn basis where one player action constitutes one turn.
The sequence within a turn is:

1. Display the oldest queued message on row 0.  If additional messages remain, append an
   inverted-colour `--More--` prompt and block all game input until the player presses Space
   to advance the queue.
2. Check for trap-door descent (ends current level immediately).
3. Refresh cursor to player position.
4. Read one input command from the player.
5. Execute the command.
6. After most commands: call `reg_move()`, which:
   1. Decrements the hunger counter.
   2. Runs all monster moves (`mv_mons()`) unless player is hasted and the haste counter is odd.
   3. Applies regeneration from rings.
   4. Applies auto-search if wearing ring of searching.
   5. Checks faint-from-hunger condition.

Actions that do NOT trigger `reg_move()` include: inventory display, help, message recall,
ring-query commands, and the quit/save dialogs.

## 5. Dungeon Generation

### 5.1. Room Layout

Each dungeon level is divided into a 3×3 grid of nine slot regions:

```text
Slots:  0 | 1 | 2
        3 | 4 | 5
        6 | 7 | 8
```

Column boundaries (COL1, COL2) and row boundaries (ROW1, ROW2) divide the 80-column, 22-row play
area into the nine slots.

For each new level, one complete row or column of rooms MUST be guaranteed to exist. Three required
slot indices are chosen at random from one of:

- `{0,1,2}`, `{3,4,5}`, `{6,7,8}` — full rows
- `{0,3,6}`, `{1,4,7}`, `{2,5,8}` — full columns

All other slots exist with 60% probability (40% chance of being skipped before the
required-room check).

### 5.2. Room Types

A slot can hold one of the following room kinds:

| Kind        | Description                                                                               |
|-------------|-------------------------------------------------------------------------------------------|
| `R_ROOM`    | Normal rectangular room (most common).                                                    |
| `R_MAZE`    | Maze room (irregular tunnel network).                                                     |
| `R_DEADEND` | Dead-end (a single tunnel cell used to connect passages).                                 |
| `R_CROSS`   | Virtual crossing point between two rooms separated by an empty slot; no room is drawn.   |
| `R_NOTHING` | Slot is completely absent this level.                                                     |

A special BIG_ROOM is generated with 1% probability when the current level equals the party
counter. The big room spans nearly the entire play area and replaces all normal room generation.

### 5.3. Passages and Connections

After rooms are placed, the generator MUST ensure all rooms are reachable from every other room.
Connection attempts are made in randomized order for:

- Adjacent horizontal neighbors (slot `i` and `i+1` in the same row).
- Adjacent vertical neighbors (slot `i` and `i+3` in the same column).
- Skip connections across an empty slot (`i` to `i+2` or `i` to `i+6`), which mark the
  intermediate empty slot as `R_CROSS`.

Generation terminates early once full connectivity is confirmed. Remaining isolated or poorly
connected slots are then processed by `fill_out_level()`, which connects them as dead-ends or
extends existing passages.

Passages are drawn as horizontal or vertical tunnel segments (`#`), potentially with one bend.
There is a 4% chance of drawing a second overlapping passage segment for the same connection.

### 5.4. Mazes

Some slots without rooms may be converted to maze rooms (`R_MAZE`) through the `add_mazes()`
procedure. Exact conditions are implementation-specific but mazes appear as irregular tunnel
networks that are connected to adjacent rooms.

### 5.5. Level Content Placement

After dungeon geometry is determined, the following items are placed:

**a) Gold:** In each room and maze (46% chance per room; always placed in mazes).
Amount = `rand(2 * level, 16 * level)`; mazes award 50% extra.

**b) Objects:** A random count of items is placed per level.
Algorithm (`put_objects` / `gr_object` in `object.c`):

1. `base_count = coin_toss() ? rand(3, 5) : rand(2, 4)`
2. Each slot has an additional 33% chance to generate one extra item.
3. For each item: pick a random floor or tunnel tile not already occupied; generate the item
   type by rolling `rand(1, 91)`:

| Roll  | Category | Sub-selection                       |
|-------|----------|-------------------------------------|
| 1–30  | Scroll   | `rand(0, 85)`: see §8.4 index order |
| 31–60 | Potion   | `rand(1, 118)`: see §8.3 index order|
| 61–64 | Wand     | `rand(0, 9)`: see §8.5 index order  |
| 65–74 | Weapon   | `rand(0, 7)`: see §8.1 index order  |
| 75–83 | Armor    | `rand(0, 6)`: see §8.2 index order  |
| 84–88 | Food     | mostly food rations, ~25% slime-mold|
| 89–91 | Ring     | `rand(0, 10)`: see §8.6 index order |

Food is forced (overrides roll) if `foods_count < cur_level/2`.

4. Items are only placed on the first visit to a level.

**c) Stairs:** One downward staircase (`%`) per level, placed on a random floor tile.

**d) Traps:** 0–MAX_TRAPS (10) hidden traps per level. All begin hidden (HIDDEN flag set).
Trap count by dungeon depth (`add_traps` in `trap.c`):

| Depth | Count Range          |
|-------|----------------------|
| 1–2   | 0 (no traps)         |
| 3–7   | `rand(0, 2)`         |
| 8–11  | `rand(1, 2)`         |
| 12–16 | `rand(2, 3)`         |
| 17–21 | `rand(2, 4)`         |
| 22–26 | `rand(3, 5)`         |
| ≥ 27  | `rand(5, MAX_TRAPS)` |

Each trap is placed on a random FLOOR tile. Trap type is chosen with equal probability from the
six types (indices 0–5).

**e) Monsters:** 4–6 monsters placed at random across walkable terrain (`put_mons` in `monster.c`).
Algorithm:

1. `count = rand(4, 6)`
2. For each monster slot:
   1. Collect all walkable floor, tunnel, and stairs cells not already occupied by another monster.
   2. Exclude cells within Chebyshev distance 3 of the player spawn position.
   3. Pick a random eligible cell and MonsterKind whose `level_range` includes the current dungeon
      depth.
   4. Monster starts asleep (ASLEEP flag).
3. In a party room: PARTY_WAKE_PERCENT (75%) of monsters start awake instead.

**f) Amulet of Yendor:** If not already carried and `level >= 26` (AMULET_LEVEL), one Amulet
is placed.

**g) Party room:** Once every PARTY_TIME (10) levels a designated room receives a party: extra
monsters and an item cache.

Objects are only placed on levels the player visits for the first time (`cur_level == max_level`
check).

## 6. Movement and Navigation

### 6.1. Single Move

Movement keys (`hjkl yubn` and their uppercase variants) move the player one cell per turn in the
corresponding direction:

```text
y  k  u
h  @  l
b  j  n
```

Movement MUST fail (MOVE_FAILED) if:

- The target cell is a wall or outside the play area.
- The player is held by a monster or caught in a bear trap AND the target cell does not contain a
  monster.
- The target cell contains an impassable tile.

On a successful move:

- If the target cell contains a MONSTER: melee attack is performed instead of moving.
- If the target cell contains a DOOR boundary: room lighting and wake-room logic triggers.
- If the target cell contains an OBJECT and pickup mode is active: automatic pickup is attempted
  (STOPPED_ON_SOMETHING result).
- If the target cell contains a visible TRAP (or invisible trap that the player steps on):
  `trap_player()` is called (STOPPED_ON_SOMETHING result).
- The ring of teleportation has an 8% (R_TELE_PERCENT) chance per move to trigger random
  teleportation.
- Confusion (active): move direction is randomized to a random cardinal/diagonal direction.

### 6.2. Run Mode

Uppercase direction keys (`H J K L B Y U N`) run the player continuously in that direction until:

- Movement fails.
- Something interesting is adjacent (monster, object, door, junction).
- The player is interrupted.

Ctrl+direction keys behave similarly but also stop when the player moves away from a passable
tunnel side-branch.

### 6.3. Visibility

Rooms are lit when the player enters them and darkened upon exit (unless the player is blind).
Tunnels reveal only the immediate neighboring cells of the player position.

Invisible monsters (INVISIBLE flag) display as their trail character (floor tile beneath them)
unless the player has `detect_monster`, `see_invisible`, or the ring of see invisible active.

## 7. Combat System

### 7.1. Melee Combat

A melee attack (player or monster) proceeds as follows.

**Hit chance (monster attacking player):**

```text
hit_chance = monster.m_hit_chance
hit_chance -= 2 * rogue.exp + 2 * ring_exp - r_rings
```

**Hit chance (player attacking monster):**

```text
hit_chance = min(60 + 2*weapon_hit_enchant + 2*exp_level, 100)
```

`weapon_hit_enchant` is the `+hit` enchantment of the wielded weapon (0 if unarmed).
`exp_level` is the player's current experience level.

If `rng(0..99) < hit_chance` the attack hits, otherwise it misses.

**Melee messages:**

| Event                      | Message shown                              |
|----------------------------|---------------------------------------------|
| Player hits monster        | `You hit <monster> for <n>.`                |
| Player hits and kills      | `You hit <monster> for <n> and kill it.`    |
| Player misses monster      | `You miss <monster>.`                       |
| Monster hits player        | `<Monster> hits you for <n>.`               |
| Monster misses player      | `The <monster> misses.`                     |

Wizard mode bypasses the player hit-chance check (every player attack lands).

**Damage (monster to player):**

```text
damage = roll(monster.m_damage)          ; dice notation e.g. "2d6"
minus   = get_armor_class(armor) * 3 / 100 * damage
net     = damage - minus
```

**Strength bonus to player damage:**

| Strength | Bonus        |
|----------|--------------|
| ≤ 6      | strength − 5 |
| 7–14     | +1           |
| 15–17    | +3           |
| 18       | +4           |
| 19–20    | +5           |
| 21       | +6           |
| 22–30    | +7           |
| > 30     | +8           |

A killed monster:

- Is removed from `level_monsters`.
- May drop an item (`cough_up` in `monster.c`): Roll `rand_percent(drop_percent)`; on success
  generate one random item via `gr_object()` and place it at the monster's last position.
  `drop_percent` per monster type (from `mon_tab`):

| Monster        | % | Monster          | %   | Monster       | %  | Monster        | %   |
|----------------|---|------------------|-----|---------------|----|----------------|-----|
| A (Aquator)    | 0 | B (Bat)          | 10  | C (Centaur)   | 15 | D (Dragon)     | 100 |
| E (Emu)        | 10| F (Flytrap)      | 0   | G (Griffin)   | 20 | H (Hobgoblin)  | 10  |
| I (Ice)        | 0 | J (Jabberwock)   | 70  | K (Kestrel)   | 10 | L (Leprechaun) | 0   |
| M (Medusa)     | 40| N (Nymph)        | 100 | O (Orc)       | 15 | P (Phantom)    | 20  |
| Q (Quagga)     | 15| R (Rattlesnake)  | 10  | S (Snake)     | 0  | T (Troll)      | 35  |
| U (Unicorn)    | 60| V (Vampire)      | 20  | W (Wraith)    | 0  | X (Xeroc)      | 30  |
| Y (Yeti)       | 30| Z (Zombie)       | 0   |               |    |                |     |

- Awards experience points (see [Section 14](#14-experience-and-leveling)).
- Releases HOLDS effect if applicable.

### 7.2. Ranged Combat (Throwing)

The `t` command throws a weapon in a chosen direction. The projectile travels up to 24 cells,
stopping at walls or when it hits a monster.

Bonus conditions:

- Arrow fired while wielding a Bow: `damage = (arrow_dmg + bow_dmg)*2/3`,
  hit_chance increased by 1/3.
- Dagger, Shuriken, or Dart thrown while wielded: `damage = dmg*3/2`,
  hit_chance increased by 1/3.

A thrown wand has a 75% chance to apply its effect instead of dealing weapon damage.

### 7.3. Special Monster Attacks

Certain monsters apply additional effects on a successful hit:

| Attack       | Effect                                                                                         |
|--------------|------------------------------------------------------------------------------------------------|
| RUSTS        | Reduces armor `d_enchant` by 1 (unless protected, ring of maintain armor worn, or leather).   |
| HOLDS        | Sets `being_held=1` (unless player levitates); player cannot move away.                        |
| FREEZES      | Immobilizes player; may cause hypothermia death.                                               |
| STINGS       | Reduces player max strength.                                                                   |
| DRAINS_LIFE  | Reduces both current and max HP.                                                               |
| DROPS_LEVEL  | Reduces player experience level by 1.                                                          |
| STEALS_GOLD  | Steals `rand(level*10, level*30)` gold; monster disappears.                                    |
| STEALS_ITEM  | Steals one non-equipped item from pack; monster disappears.                                    |
| FLAMES       | Dragon fire: damage reduced by armor class.                                                    |
| CONFUSES     | May confuse player.                                                                            |

Confused monsters have a 66% chance to skip their special attack.

### 7.4. Death and Victory Conditions

Player death triggers when `rogue.hp_current` reaches 0. Death causes:

- Gold reduced to 90% of current amount (except on quit).
- Skull display (if `show_skull=1` and not quit).
- Score recording.

Victory occurs when the player stands on `<` (up-stairs) while carrying the Amulet of Yendor.
Gold is fully tallied and sold.

## 8. Items

Items are identified by appearance (potion color, scroll title, wand material) which are
randomized per game session. Players may use items to reveal their true identity, or name them
with `c` (call).

### 8.1. Weapons

| Index | Name             | Damage | Notes                           |
|-------|------------------|--------|---------------------------------|
| 0     | Short bow        | "1d2"  | Improves arrow attacks          |
| 1     | Darts            | "1d1"  | Stackable, thrown bonus         |
| 2     | Arrows           | "1d2"  | Stackable, bow-bonus compatible |
| 3     | Daggers          | "1d6"  | Thrown bonus while wielded      |
| 4     | Shurikens        | "1d5"  | Thrown bonus while wielded      |
| 5     | Mace             | "2d3"  | Player starting weapon          |
| 6     | Long sword       | "3d4"  |                                 |
| 7     | Two-handed sword | "4d5"  |                                 |

Weapons have `hit_enchant` (modifier to to-hit) and `d_enchant` (modifier to damage dice result).
Enchants can be negative (cursed).

### 8.2. Armor

| Index | Name          | Class | Notes                 |
|-------|---------------|-------|-----------------------|
| 0     | Leather armor | 2     | Immune to rust        |
| 1     | Ring mail     | 3     | Player starting armor |
| 2     | Scale mail    | 4     |                       |
| 3     | Chain mail    | 5     |                       |
| 4     | Banded mail   | 6     |                       |
| 5     | Splint mail   | 6     |                       |
| 6     | Plate mail    | 7     |                       |

Class is the base armor class before `d_enchant` modifier. Higher armor class means better
protection. A protected armor cannot be rusted. Cursed armor cannot be removed.

### 8.3. Potions

| Index | Effect              | Base Value |
|-------|---------------------|------------|
| 0     | Increase strength   | 100        |
| 1     | Restore strength    | 250        |
| 2     | Healing             | 100        |
| 3     | Extra healing       | 200        |
| 4     | Poison              | 10         |
| 5     | Raise level         | 300        |
| 6     | Blindness           | 10         |
| 7     | Hallucination       | 25         |
| 8     | Detect monster      | 100        |
| 9     | Detect things       | 100        |
| 10    | Confusion           | 10         |
| 11    | Levitation          | 80         |
| 12    | Haste self          | 150        |
| 13    | See invisible       | 145        |

Healing restores HP equal to experience level; may permanently raise HP max if current HP is
close to max. Extra healing doubles this.

### 8.4. Scrolls

| Index | Effect            | Base Value |
|-------|-------------------|------------|
| 0     | Protect armor     | 505        |
| 1     | Hold monster      | 200        |
| 2     | Enchant weapon    | 235        |
| 3     | Enchant armor     | 235        |
| 4     | Identify          | 175        |
| 5     | Teleportation     | 190        |
| 6     | Sleep             | 25         |
| 7     | Scare monster     | 610        |
| 8     | Remove curse      | 210        |
| 9     | Create monster    | 100        |
| 10    | Aggravate monster | 25         |
| 11    | Magic mapping     | 180        |

A dropped Scare Monster scroll is destroyed when picked up again after being placed on the floor
(it has been "used").

### 8.5. Wands / Staves

Wands and staves share the same mechanics; appearance (wood vs. metal) is randomized per game.
Each wand has a finite charge count.

| Index | Effect           | Base Value | Notes                              |
|-------|------------------|------------|------------------------------------|
| 0     | Teleport away    | 25         |                                    |
| 1     | Slow monster     | 50         |                                    |
| 2     | Confuse monster  | 45         |                                    |
| 3     | Invisibility     | 8          | Makes monster invisible            |
| 4     | Polymorph        | 55         | Transforms monster randomly        |
| 5     | Haste monster    | 2          |                                    |
| 6     | Sleep            | 25         |                                    |
| 7     | Magic missile    | 20         | Force-hit attack                   |
| 8     | Cancellation     | 20         | Removes monster special abilities  |
| 9     | Do nothing       | 0          |                                    |

### 8.6. Rings

A maximum of two rings may be worn simultaneously (one per hand). Cursed rings cannot be removed.

| Index | Effect                 | Base Value | Notes                               |
|-------|------------------------|------------|-------------------------------------|
| 0     | Stealth                | 250        | Reduces monster wake radius         |
| 1     | Teleportation (cursed) | 100        | 8% chance per move to teleport      |
| 2     | Regeneration           | 255        | Recovers 1 HP per turn              |
| 3     | Slow digestion         | 295        | Halves hunger rate                  |
| 4     | Add strength           | 200        | ±1 to ±2 strength modifier          |
| 5     | Sustain strength       | 250        | Prevents strength loss              |
| 6     | Dexterity              | 250        | ±1 to ±2 to-hit modifier            |
| 7     | Adornment              | 25         | Cosmetic; may be cursed             |
| 8     | See invisible          | 300        | Reveals invisible monsters          |
| 9     | Maintain armor         | 290        | Prevents rust damage                |
| 10    | Searching              | 270        | Auto-searches each turn             |

Wearing two rings causes the hunger counter to decrease twice as fast.

### 8.7. Food

Two food subtypes exist:

| Index | Name                                                                     |
|-------|--------------------------------------------------------------------------|
| 0     | Ration of food (generic food item)                                       |
| 1     | Fruit (named by ROGUEOPTS `fruit=` option; default: slime-mold)          |

Eating food replenishes the hunger counter and may print a flavor message.

### 8.8. Gold

Gold piles (`*`) are automatically collected when the player steps onto their tile (no `,` key
required); they are never placed in the pack.

Spawning (`put_gold` / `plant_gold` in `object.c`):

- Each room in the level is processed once.
- Regular rooms: 46% chance (GOLD_PERCENT) of receiving a pile.
- Maze rooms: always receive a pile (100% spawn chance).
- Quantity: `rand(2 × level, 16 × level)`.
- Maze-room bonus: `quantity += quantity / 2` (50% extra).
- Placement: up to 50 random attempts inside the room interior; the tile must be FLOOR or TUNNEL
  and must not already hold a pile or stairs.

Hard cap: MAX_GOLD = 900 000; gold is clamped on pickup.

**Leprechaun interactions:**

- STEALS_GOLD: When a Leprechaun hits the player, `rand(level×10, level×30)` gold is stolen
  (10% immunity chance per hit). The Leprechaun disappears immediately after the theft
  (`disappear()` in original). Message: "Your purse feels lighter."
- Cough-up on kill: Killing a Leprechaun always drops a gold pile of `rand(level×15, level×30)`
  at the kill position (`cough_up()` in `spec_hit.c`).

**Death penalty** (`killed_by()` in `score.c`):

- On death: `gold = (gold × 9) / 10` before the score is recorded.
- Voluntary quit does not incur the penalty.

**Orc (SEEKS_GOLD):** Original behaviour of navigating toward floor gold in the same room is not
yet implemented (see CONSISTENCY.md).

### 8.9. Amulet of Yendor

Represented by glyph `,`. Exactly one appears when the player first reaches level 26
(AMULET_LEVEL) or deeper, if not already carried. Carrying the amulet while ascending stairs past
level 1 triggers the victory sequence. After retrieving the amulet, monsters on deeper levels
become permanently hasted.

## 9. Inventory and Pack

The pack is a linked list of `object` structs. Maximum capacity is MAX_PACK_COUNT = 24 items
(gold does not count toward this limit).

Stackable item types: weapons of the same kind/quiver group (arrows, darts, daggers, shurikens),
food rations, scrolls of the same kind, potions of the same kind.

Items are assigned inventory letters `a`–`z` in first-available order. Letter `L` is used for
items placed on the floor via drop.

Item use-state flags:

| Flag           | Meaning                        |
|----------------|--------------------------------|
| BEING_WIELDED  | Weapon currently in hand.      |
| BEING_WORN     | Armor currently equipped.      |
| ON_LEFT_HAND   | Ring on left hand.             |
| ON_RIGHT_HAND  | Ring on right hand.            |

Cursed items in use cannot be unequipped.

## 10. Monsters

### 10.1. Monster Roster

Twenty-six monster types exist, one per uppercase letter A–Z.

| Char | Name           | HP  | Levels  | Hit% | Damage(s) | XP   |
|------|----------------|-----|---------|------|-----------|------|
| A    | Aquator        | 25  | 9–18    | 100  | 0d0       | 20   |
| B    | Bat            | 10  | 1–8     | 60   | 1d3       | 2    |
| C    | Centaur        | 32  | 7–16    | 85   | 3d3/2d5   | 15   |
| D    | Dragon         | 145 | 21–126  | 100  | 4d6/4d9   | 5000 |
| E    | Emu            | 11  | 1–7     | 65   | 1d3       | 2    |
| F    | Venus fly-trap | 73  | 12–126  | 80   | 5d5       | 91   |
| G    | Griffin        | 115 | 20–126  | 85   | 5d5/5d5   | 2000 |
| H    | Hobgoblin      | 15  | 1–10    | 67   | 1d3/1d2   | 3    |
| I    | Ice monster    | 15  | 2–11    | 68   | 0d0       | 5    |
| J    | Jabberwock     | 132 | 21–126  | 100  | 3d10/4d5  | 3000 |
| K    | Kestrel        | 10  | 1–6     | 60   | 1d4       | 2    |
| L    | Leprechaun     | 25  | 6–16    | 75   | 0d0       | 21   |
| M    | Medusa         | 97  | 18–126  | 85   | 4d4/3d7   | 250  |
| N    | Nymph          | 25  | 10–19   | 75   | 0d0       | 39   |
| O    | Orc            | 25  | 4–13    | 70   | 1d6       | 5    |
| P    | Phantom        | 76  | 15–24   | 80   | 5d4       | 120  |
| Q    | Quagga         | 30  | 8–17    | 78   | 3d5       | 20   |
| R    | Rattlesnake    | 19  | 3–12    | 70   | 2d5       | 10   |
| S    | Snake          | 8   | 1–9     | 50   | 1d3       | 2    |
| T    | Troll          | 75  | 13–22   | 75   | 4d6/1d4   | 125  |
| U    | Black unicorn  | 90  | 17–26   | 85   | 4d10      | 200  |
| V    | Vampire        | 55  | 19–126  | 85   | 1d14/1d4  | 350  |
| W    | Wraith         | 45  | 14–23   | 75   | 2d8       | 55   |
| X    | Xeroc          | 42  | 16–25   | 75   | 4d6       | 110  |
| Y    | Yeti           | 35  | 11–20   | 80   | 3d6       | 50   |
| Z    | Zombie         | 21  | 5–14    | 69   | 1d7       | 8    |

Monsters only appear on levels within their `[first_level, last_level]` range. Above
AMULET_LEVEL + 2, all monsters gain the HASTED flag.

### 10.2. Monster Flags

| Flag            | Description                                                                                          |
|-----------------|------------------------------------------------------------------------------------------------------|
| ASLEEP          | Monster starts in sleep state; WAKENS flag controls proximity-based wake-up.                         |
| WAKENS          | Monster wakes if player is adjacent (chance modified by stealth).                                    |
| WANDERS         | Monster moves randomly when awake.                                                                   |
| FLITS           | Monster moves randomly regardless of target.                                                         |
| FLIES           | Monster gets an extra movement step.                                                                 |
| HASTED          | Monster acts twice per turn.                                                                         |
| SLOWED          | Monster acts every other turn.                                                                       |
| CONFUSED        | Monster movement is randomized; 66% skip on special hits.                                            |
| INVISIBLE       | Monster is hidden unless player has see_invisible.                                                   |
| IMITATES        | Monster disguises itself as a random object glyph.                                                   |
| STATIONARY      | Monster does not move (Venus fly-trap); deals escalating damage each turn player stays adjacent.     |
| FREEZING_ROGUE  | Monster is actively freezing the player.                                                             |
| ALREADY_MOVED   | Monster skips its next move (set after hasted move).                                                 |
| NAPPING         | Monster in temporary sleep from Sleep wand.                                                          |
| HOLDS           | Monster prevents player movement.                                                                    |

### 10.3. Monster AI

On each turn (`mv_mons()`), each monster executes in list order:

1. If HASTED: perform one extra move.
2. If SLOWED: skip every other turn (`slowed_toggle`).
3. If CONFUSED: try `move_confused()` (random walk).
4. If FLIES and not adjacent to player: perform bonus move.
5. Standard move toward player.

Pathfinding: monsters prefer to move on the axis closest to the player, trying the exact target
cell first, then adjacent cells in random order. If stuck for more than 4 turns, the monster picks
a random wander target.

Wake-up chance = `WAKE_PERCENT / (STEALTH_FACTOR + stealthy)`, where WAKE_PERCENT is a baseline
percentage and `stealthy` is the ring-of-stealth bonus.

### 10.4. Special Monster Behaviors

- **Aquator (A):** Always hits; rusts equipped armor.
- **Dragon (D):** Breathes flame at range (FLAMES flag) dealing armor-reduced fire damage.
- **Venus fly-trap (F):** STATIONARY; HOLDS player; damage escalates by 1 each consecutive turn
  player remains adjacent.
- **Leprechaun (L):** STEALS_GOLD; teleports away after theft.
- **Medusa (M):** CONFUSES player on hit.
- **Nymph (N):** STEALS_ITEM; teleports away after theft.
- **Phantom (P):** Invisible; flits randomly.
- **Vampire (V):** DRAINS_LIFE on hit; cannot be killed to 0 HP by drain alone (drain merely
  reduces max and current HP).
- **Wraith (W):** DROPS_LEVEL; reduces player experience level by 1.
- **Xeroc (X):** IMITATES a random object; reveals itself on attack.

## 11. Traps

Up to MAX_TRAPS (10) traps may exist per level. All traps begin hidden (HIDDEN flag). A trap is
revealed when:

- The player steps on it (trap always triggers on first step).
- The player uses `^` to identify an adjacent trap (marks it known).
- The player uses `s` to search adjacent cells (probabilistic reveal).

The `^` command identifies the first hidden trap among the 8 cells adjacent to the player and
marks it in the `known_traps` list.

The `s` search command (`search()` in `move.c`) examines all 8 adjacent cells in a single pass.
For each cell, if a hidden trap (or secret passage) occupies it, a reveal roll is made:

```text
reveal_chance = HIDE_PERCENT + player_exp_level + ring_search_exp
```

where `ring_search_exp = 2` per equipped Ring of Searching. A successful `rand_percent(reveal_chance)`
roll reveals the feature:

- Hidden trap: marked as known, message "You found a \<name\>."
- Secret passage: HIDDEN flag cleared; tunnel becomes visible.

One world turn is consumed by `s` regardless of outcome.

**Trap types (index 0–5):**

| Index | Name              | Effect                                                                      |
|-------|-------------------|-----------------------------------------------------------------------------|
| 0     | Trap door         | Immediately descends player one level.                                      |
| 1     | Bear trap         | Immobilizes player for `rand(4,7)` turns.                                   |
| 2     | Teleport trap     | Random teleportation (same level).                                          |
| 3     | Dart trap         | 1d6 damage; 40% chance to reduce strength by 1 (unless sustain_strength ring worn). |
| 4     | Sleeping gas trap | Player falls asleep for several monster turns.                              |
| 5     | Rust trap         | Reduces equipped armor `d_enchant` by 1.                                    |

Save chance for all traps: `rand_percent(rogue.exp + ring_exp)` succeeds → trap fails with
message "the trap failed".

## 12. Status Effects

| Effect                  | Description                                                                                  |
|-------------------------|----------------------------------------------------------------------------------------------|
| `halluc` (counter)      | Hallucination: monsters and items shown as random glyphs. Decremented each turn. Cured by poison potion or restore strength. |
| `blind` (counter)       | Blindness: dungeon display suppressed. Player cannot read scrolls. Auto-stops running.       |
| `confused` (counter)    | Confusion: movement direction randomized. Cured when counter reaches 0.                     |
| `levitate` (counter)    | Levitation: player cannot trigger bear traps, rust traps, or pick up items. `being_held` cleared. Does not affect trap-door and other traps. |
| `haste_self` (counter)  | Haste: player acts on every tick regardless of `mv_mons()` haste check. Odd values ensured. |
| `see_invisible` (bool)  | Reveals invisible monsters.                                                                   |
| `detect_monster` (bool) | All monsters shown regardless of visibility.                                                  |
| `being_held` (bool)     | Set by Venus fly-trap; player cannot move.                                                    |
| `bear_trap` (counter)   | Set by bear trap; counts down; player cannot move until counter reaches 0.                   |

## 13. Hunger System

`rogue.moves_left` starts at 1250 and decrements each `reg_move()`. Wearing two rings doubles
consumption (decrements by 2 per move). The ring of slow digestion halves consumption.

Hunger thresholds (checked in `reg_move()`):

| moves_left | Status String   | Effect                                    |
|------------|-----------------|-------------------------------------------|
| > 300      | ""              | Normal                                    |
| 151–300    | "Hungry"        | Printed on status bar                     |
| 1–150      | "Weak"          | Printed on status bar; may faint          |
| ≤ 0        | "Faint" / dead  | Death by starvation if sustained          |

Eating food restores `moves_left` to a normal level and clears the hunger status string.

## 14. Experience and Leveling

Experience points are awarded on monster kill. Thresholds:

| Level | Points Required              |
|-------|------------------------------|
| 1     | 10                           |
| 2     | 20                           |
| 3     | 40                           |
| 4     | 80                           |
| 5     | 160                          |
| 6     | 320                          |
| 7     | 640                          |
| 8     | 1,300                        |
| 9     | 2,600                        |
| 10    | 5,200                        |
| 11    | 10,000                       |
| 12    | 20,000                       |
| 13    | 40,000                       |
| 14    | 80,000                       |
| 15    | 160,000                      |
| 16    | 320,000                      |
| 17    | 1,000,000                    |
| 18    | 3,333,333                    |
| 19    | 6,666,666                    |
| 20    | 10,000,000 (MAX_EXP)         |
| 21    | 99,900,000 (hard cap for display) |

Maximum experience level is 21 (MAX_EXP_LEVEL).

On level-up: HP max increases by a random amount; HP current restored to new maximum.

## 15. Scoring

The score file holds up to 10 entries, ranked by gold amount. Each entry records: rank, gold,
player login name, cause of death, and dungeon level reached.

Score = gold carried at game end. On death the gold is reduced to 90% before recording
(`killed_by()` in `score.c`). On voluntary quit the full gold total is recorded.

Scores are stored in an obfuscated binary format to discourage tampering. The file is opened with
setuid-games privileges (`turn_into_games()` / `turn_into_user()`).

A player's existing score is replaced only if the new gold total is higher.

Victory (WIN) awards bonus gold equivalent to the sold value of all remaining inventory.

## 16. Save and Restore

The `S` command prompts for a filename and saves the full game state, then exits. Saving writes:

- All status flags, counters, and player stats.
- Dungeon grid state.
- All level monsters and level objects.
- The player's pack contents.
- Item identification tables (potions, scrolls, wands, rings).
- Trap array and room geometry.
- A file-modification timestamp used as an anti-cheat measure.

Restoration fails (`clean_up()`) if:

- The file has been modified after saving.
- The save file has been hard-linked (link count > 1).
- The login name does not match the original player.
- The file ID has changed.

The error save file `rogue.esave` is used for crash recovery.

## 17. Command Reference

All commands are case-sensitive unless noted.

**Movement:**

| Key                    | Action                             |
|------------------------|------------------------------------|
| `h` / `j` / `k` / `l` | Move left / down / up / right      |
| `y` / `u` / `b` / `n` | Move diagonal (↖ ↗ ↙ ↘)           |
| `H` / `J` / `K` / `L` | Run left / down / up / right       |
| `Y` / `U` / `B` / `N` | Run diagonal (until stop)          |
| Ctrl+direction         | Same as run (`hjkluynb` equivalents)|
| `m <dir>`              | Move onto (no pickup)              |

**Combat:**

| Key       | Action                          |
|-----------|---------------------------------|
| `f <dir>` | Fight in direction (stop if risky) |
| `F <dir>` | Fight to the death              |
| `t <dir>` | Throw weapon in direction       |
| `z <dir>` | Zap wand in direction           |

**Items:**

| Key | Action                                     |
|-----|--------------------------------------------|
| `e` | Eat food                                   |
| `q` | Quaff potion                               |
| `r` | Read scroll                                |
| `w` | Wield weapon                               |
| `W` | Wear armor                                 |
| `T` | Take off armor                             |
| `P` | Put on ring                                |
| `R` | Remove ring                                |
| `d` | Drop item                                  |
| `,` | Kick into pack (pick up item at position)  |
| `c` | Call item (assign personal name)           |

**Information:**

| Key     | Action                                     |
|---------|--------------------------------------------|
| `i`     | Inventory (all items)                      |
| `I`     | Single item inventory (by letter)          |
| `)`     | Show wielded weapon                        |
| `]`     | Show worn armor                            |
| `=`     | Show worn rings                            |
| `^`     | Identify adjacent trap                     |
| `s`     | Search adjacent cells (repeat with count)  |
| `.`     | Rest one turn (or count turns)             |
| Ctrl+A  | Show run statistics overlay                |

**System:**

| Key     | Action                                            |
|---------|---------------------------------------------------|
| `S`     | Save game (and exit)                              |
| `Q`     | Quit game (counted as death)                      |
| `v`     | Print version string                              |
| `?`     | Display help file                                 |
| Ctrl+P  | Recall last message                               |
| `>`     | Descend stairs                                    |
| `<`     | Ascend stairs (requires Amulet for surface)       |
| `0`–`9` | Numeric prefix for repeat count                  |

**Wizard mode (if enabled):**

| Key          | Action                       |
|--------------|------------------------------|
| Ctrl+W       | Toggle wizard mode           |
| Tab (Ctrl+I) | Show all level objects       |
| Ctrl+S       | Draw magic map               |
| Ctrl+T       | Show all traps               |
| Ctrl+O       | Show all objects             |
| Ctrl+C       | Add random item (wizard only)|
| Ctrl+M       | Show all monsters            |

## 18. Options (ROGUEOPTS)

Options are read from the ROGUEOPTS environment variable as a comma-separated list of
`key=value` pairs.

| Option          | Default      | Description                                       |
|-----------------|--------------|---------------------------------------------------|
| `fruit=<name>`  | "slime-mold" | Name of the fruit item (FRUIT subtype).           |
| `file=<path>`   | ""           | Default save file path.                           |
| `name=<nick>`   | ""           | Player nickname for score display.                |
| `nojump`        | —            | Disable screen-jump animation on run.             |
| `noaskquit`     | —            | Skip "really quit?" confirmation.                 |
| `noskull`/`notomb` | —         | Suppress skull art on death.                      |

Command line flag: `-s` — Display scores only (no game).  
Command line arg: `<filename>` — Restore saved game from file.

## 19. Constants Reference

| Constant       | Value    | Description                                  |
|----------------|----------|----------------------------------------------|
| DROWS          | 24       | Dungeon display rows                         |
| DCOLS          | 80       | Dungeon display columns                      |
| MAXROOMS       | 9        | Number of room slots per level               |
| MAX_PACK_COUNT | 24       | Maximum items in player pack                 |
| AMULET_LEVEL   | 26       | Dungeon level where Amulet appears           |
| LAST_DUNGEON   | 99       | Maximum dungeon depth                        |
| MAX_EXP_LEVEL  | 21       | Maximum player experience level              |
| MAX_EXP        | 10000000 | Maximum experience points                    |
| MAX_GOLD       | 900000   | Maximum gold displayable                     |
| MAX_ARMOR      | 99       | Maximum armor class                          |
| MAX_HP         | 800      | Maximum player hit points                    |
| MAX_STRENGTH   | 99       | Maximum player strength                      |
| MAX_TRAPS      | 10       | Maximum traps per level                      |
| PARTY_TIME     | 10       | Level interval between party rooms           |
| HIDE_PERCENT   | 12       | Base % chance per search to reveal hidden tile |
| STEALTH_FACTOR | 3        | Divisor for wake-chance with stealth ring    |
| R_TELE_PERCENT | 8        | % chance per move for teleport ring trigger  |
| INIT_HP        | 12       | Player starting hit points                   |
| GOLD_PERCENT   | 46       | % chance of gold in a normal room            |
| BIG_ROOM       | 10       | Room-number sentinel for big room            |
| MONSTERS       | 26       | Number of different monster types            |
| WEAPONS        | 8        | Number of weapon types                       |
| ARMORS         | 7        | Number of armor types                        |
| SCROLLS        | 12       | Number of scroll types                       |
| POTIONS        | 14       | Number of potion types                       |
| WANDS          | 10       | Number of wand types                         |
| RINGS          | 11       | Number of ring types                         |
| TRAPS          | 6        | Number of trap types                         |
| NO_TRAP        | -1       | Sentinel: no trap at location                |
| PASSAGE        | -3       | `cur_room` value when in a tunnel            |

---

> **Authors' Note:** This specification was derived by static analysis of the rogue-libc5-ncurses
> C source code (Rogue 5.3-clone, patchlevel 1), originally ported to Linux by Alan Cox. Where
> behavior is controlled by randomness, the specified ranges match the source exactly.
> Implementation details not observable through gameplay (e.g., score file obfuscation algorithm,
> exact RNG sequence) are intentionally omitted.

---

## 20. Wizard Mode

Wizard mode is a privileged debug/testing mode that grants the player invulnerability, unrestricted
navigation, and access to diagnostic commands. When wizard mode is active the game run is
automatically disqualified from the high-score table (`score_only = true`).

### 20.1. Activation and Deactivation

Wizard mode is toggled at runtime by pressing Ctrl+W.

**If wizard mode is currently INACTIVE:**

1. The game prompts "wizard's password:".
2. The player types the password (input is not echoed).
3. The input is transformed with the obfuscation cipher defined in `score.c`: `xxx()` initialises
   the PRNG to `f=37, s=7`; `xxxx()` XORs each byte of the input with successive
   `(unsigned char) xxx(0)` values.
4. The transformed result is compared with the seven-byte literal `"\247\104\126\272\115\243\027"`.
   The plaintext password that satisfies this check is "bathtub".
5. On success: `wizard` is set to 1, `score_only` is set to 1, and the message
   "Welcome, mighty wizard!" is shown.
6. On failure: the message "sorry" is shown and the game continues with wizard mode inactive.

**If wizard mode is currently ACTIVE:**

1. `wizard` is set to 0 immediately (no password required to deactivate).
2. The message "not wizard anymore" is shown.

Source: `wizardize()` in `zap.c`.

### 20.2. Score Disqualification

Setting `wizard = 1` also sets `score_only = 1`. When `score_only` is set, the normal score-file
update path is skipped; the run is NOT recorded to the shared top-ten score file. This flag
persists for the rest of the run and CANNOT be cleared.

Source: `wizardize()` in `zap.c`; score recording guards in `score.c`.

### 20.3. Wizard-Only Commands

The following key bindings are available ONLY while `wizard == 1`. Pressing these keys without
wizard mode active displays the "unknown command" error message.

| Key    | Function                                                                              |
|--------|---------------------------------------------------------------------------------------|
| Tab    | Show all items currently on the dungeon floor (`inventory(&level_objects, ALL_OBJECTS)`) |
| Ctrl+S | Draw magic map – reveal the entire level layout (walls, tunnels, doors, stairs, traps); equivalent to the Scroll of Magic Mapping effect |
| Ctrl+T | Show all traps – mark every trap cell with `^`                                        |
| Ctrl+O | Show all objects – render the map character of every floor item at its position       |
| Ctrl+C | Create a new item – prompt for object type (using the item glyph: `!`, `?`, `:`, `)`, `]`, `/`, `=`, `,`) and then for the specific kind number; add the resulting item to the player's pack |
| Ctrl+M | Show all monsters – render every monster's glyph at its position and reveal imitators |

Source: `play_level()` in `play.c`; `new_object_for_wizard()` in `object.c`;
`show_monsters()` in `monster.c`; `show_objects()` in `object.c`; `show_traps()` in `trap.c`;
`draw_magic_map()` in `room.c`.

### 20.4. Unrestricted Navigation

While `wizard == 1`:

**a) Descend (`>`):** `drop_check()` always returns 1 regardless of whether the player is
standing on a STAIRS tile. The player MAY descend from any floor cell.
Source: `drop_check()` in `level.c`.

**b) Ascend (`<`):** `check_up()` skips both the stairs-presence check and the Amulet-of-Yendor
requirement. The player MAY ascend from any floor cell at any depth, including level 1 which
triggers the victory condition.
Source: `check_up()` in `level.c`.

### 20.5. Combat Modifiers

While `wizard == 1` the following modifiers apply to every combat calculation for the duration
of the run:

**Player attacking** (`rogue_hit`):

- Hit chance is DOUBLED (`hit_chance × 2`, then compared with `rand_percent`).
- Damage dealt is TRIPLED (`damage × 3`).

Source: `rogue_hit()` in `hit.c`.

**Monster attacking** (`mon_hit`):

- Hit chance is HALVED (`hit_chance / 2`).
- Damage dealt is reduced to ONE-THIRD (`damage / 3`).

Source: `mon_hit()` in `hit.c`.

### 20.6. Leveling HP Gain

When the player gains an experience level while `wizard == 1`, the `hp_raise()` function returns
the fixed value 10 instead of the normal `get_rand(3, 10)`. The player always gains exactly 10
maximum HP per level-up in wizard mode.

Source: `hp_raise()` in `level.c`.

### 20.7. Inventory Identification

While `wizard == 1`, `get_desc()` treats every item as fully identified regardless of its actual
`id_status`:

- Scrolls and potions show their real name.
- Wands show their real name AND the current charge count in brackets
  (e.g., "wand of slow monster[3]").
- Rings show their real name AND the modifier value for Dexterity / Add Strength rings
  (e.g., "+2 ring of add strength").
- Armor and weapon enchantments are always shown.

Source: `get_desc()` in `inventory.c` (wizard jump-to-ID branch).

### 20.8. Ring Diagnostics

When the player views their worn rings while `wizard == 1` (`inv_rings()` in `ring.c`), an
additional diagnostic message is appended after the ring descriptions showing the current values
of all ring-effect counters:

```text
"ste %d, r_r %d, e_r %d, r_t %d, s_s %d, a_s %d, reg %d, r_e %d, s_i %d, m_a %d, aus %d"
```

(stealthy, r_rings, e_rings, r_teleport, sustain_strength, add_strength, regeneration,
ring_exp, r_see_invisible, maintain_armor, auto_search)
