# Hit Point Management

Analysis of hit-point logic from the original C sources and its status in the Rust port.
Reference files: `original/rogue-libc5-ncurses/rogue/hit.c`, `spec_hit.c`, `use.c`, `move.c`, `level.c`, `ring.c`, `trap.c`, `monster.c`, `rogue.h`, `object.c`.

---

## §1 — Initial HP

**Original:** `rogue.hp_current = rogue.hp_max = INIT_HP = 12` (`object.c: player_init`)

**Rust:** `player_hit_points: 12`, `player_max_hit_points: 12` in `GameLoop::new()` ✅

---

## §2 — Monster Damage Dice (NdD format)

**Original:** Each monster in `mon_tab` has a damage string `"NdD"` or `"NdD/NdD"` (multiple attacks summed). Function `get_damage(str, mult)` rolls each component and accumulates the result.

| Monster | Damage string |
|---|---|
| Aquator | `0d0` |
| Bat | `1d3` |
| Centaur | `3d3/2d5` |
| Dragon | `4d6/4d9` |
| Emu | `1d3` |
| VenusFlytrap | STATIONARY: 0, 1, 2, 3… (increments each attack) |
| Griffin | `5d5/5d5` |
| Hobgoblin | `1d3/1d2` |
| IceMonster | `0d0` |
| Jabberwock | `3d10/4d5` |
| Kestrel | `1d4` |
| Leprechaun | `0d0` |
| Medusa | `4d4/3d7` |
| Nymph | `0d0` |
| Orc | `1d6` |
| Phantom | `5d4` |
| Quagga | `3d5` |
| Rattlesnake | `2d5` |
| Snake | `1d3` |
| Troll | `4d6/1d4` |
| Black Unicorn | `4d10` |
| Vampire | `1d14/1d4` |
| Wraith | `2d8` |
| Xeroc | `4d6` |
| Yeti | `3d6` |
| Zombie | `1d7` |

**Rust:** `Monster` now stores `damage_string: &'static str`; `roll_damage_string()` parses `NdD/NdD` and rolls each component with the game RNG. ✅

---

## §3 — Monster Hit Chance (`m_hit_chance`)

**Original:** `mon_hit()` calls `rand_percent(hit_chance)` before dealing damage. Each monster has a specific `m_hit_chance` value in `mon_tab`:

| Monster | Hit chance |
|---|---|
| Aquator, Dragon, Jabberwock | 100% |
| Bat, Kestrel | 60% |
| Centaur, Medusa, Griffin, Vampire, Black Unicorn | 85% |
| Emu | 65% |
| VenusFlytrap, Phantom | 80% |
| Hobgoblin | 67% |
| IceMonster | 68% |
| Leprechaun, Nymph, Troll, Wraith, Xeroc | 75% |
| Orc, Rattlesnake | 70% |
| Quagga, Yeti | 78%/80% |
| Snake | 50% |
| Zombie | 69% |

**Rust:** `Monster` now stores `hit_chance: i16`. `tick_monsters` checks `rng.get_rand(0,99) < hit_chance` before emitting a damage event; on a miss, no event is emitted. ✅

---

## §4 — Armor Class Mitigation

**Original:** `mon_hit()` subtracts armor class from damage. AC in Rogue is inverted (lower = better); the mitigation formula is: `damage -= rogue.ac` where AC can be negative for heavy armor.

**Rust:** `player_armor_bonus()` sums `armor_bonus` from equipped items; mitigated damage = `max(damage - armor_bonus, 1)`. Matches the spirit of the original. ✅ (simplified)

---

## §5 — Player Hit Chance (`get_hit_chance`)

**Original:** `rogue_hit()` calls `get_hit_chance(weapon)`: base 40% + 3 × weapon enchantments. Player always hits on forced attacks (e.g., wand, throw).

**Rust:** Player attacks always land (no hit-chance roll). ⚠️ (partial — force-hit only)

---

## §6 — Player Damage (`damage_for_strength` + `get_weapon_damage`)

**Original:**
- Strength bonus table `damage_for_strength()`: str 3 → -3 … str 16 → +2 … str 18 → +5
- Weapon rolls its own NdD dice, scaled by enchantments
- Player exp-level bonus: +1 every two levels gained

**Rust:** `player_attack_damage()` applies `damage_for_strength()` lookup table and `(exp_level-1)/2` bonus on top of weapon `attack_bonus`. Weapon lacks per-weapon dice (uses flat `attack_bonus`). ✅ (simplified dice)

---

## §7 — Passive Healing (`heal`)

**Original:** `move.c: heal()` called every turn. Heals +1 HP every N turns, alternating +2 HP every other interval. Interval N is a function of experience level:

| Exp level | Heal interval (turns) |
|---|---|
| 1 | 20 |
| 2 | 18 |
| 3 | 17 |
| 4 | 14 |
| 5 | 13 |
| 6 | 10 |
| 7 | 9 |
| 8 | 8 |
| 9 | 7 |
| 10 | 4 |
| 11 | 3 |
| 12+ | 2 |

**Rust:** Implemented in `advance_world_turn()`: heals +1 or +2 HP when `turns % interval == 0` and not at full HP. Ring of Regeneration adds +1 bonus per tick. ✅

---

## §8 — Potion of Healing (`potion_heal`)

**Original:** `use.c: potion_heal(n)` where `n = rogue.exp` (current level):
```
new_hp = current + n
if new_hp > max:
    if current == max: max_hp++, hp = max_hp   (increase max)
    else:              hp = max_hp              (cap at max)
else:
    hp = new_hp
```
Extra Healing potion uses `n = 2 × rogue.exp`.

**Rust:** Implemented matching this formula. Healing potion heals `player_exp_level` HP; Extra Healing heals `2 × player_exp_level` HP. Both can increment `player_max_hit_points` by 1 when already at max. ✅

---

## §9 — Level-Up HP Gain (`hp_raise`) + Per-Monster EXP

**Original:**
- `hp_raise() = get_rand(3, 10)` — random HP gain on level-up
- `rogue.exp_points += mon_tab[monster].kill_exp` — per-monster experience reward

| Monster | kill_exp |
|---|---|
| Bat, Kestrel, Snake, Emu | 2 |
| Hobgoblin | 3 |
| Zombie | 8 |
| Rattlesnake | 10 |
| Centaur | 15 |
| Aquator, Quagga | 20 |
| Leprechaun | 21 |
| Orc | 5 |
| IceMonster | 5 |
| Nymph | 39 |
| Yeti | 50 |
| Wraith | 55 |
| VenusFlytrap | 91 |
| Xeroc | 110 |
| Phantom | 120 |
| Troll | 125 |
| Black Unicorn | 200 |
| Medusa | 250 |
| Vampire | 350 |
| Griffin | 2000 |
| Jabberwock | 3000 |
| Dragon | 5000 |

**Rust:** `Monster` stores `kill_exp: i32`; `attack_monster` returns it in `CombatEvent::PlayerHitMonster { kill_exp }`; `try_move_player` uses `rng.get_rand(3, 10)` for HP gain. ✅

---

## §10 — Dart Trap

**Original:** `trap.c`: always deals `get_damage("1d6", 1)` HP. Strength drain only occurs with 40% probability (and is blocked by Ring of Sustain Strength).

**Rust:** Dart damage is now `rng.get_rand(1, 6)`. Strength drain has 40% chance and checks for Ring of Sustain Strength. Old fixed `-2 damage` removed. ✅

---

## §11 — Special Effect Probabilities

### §11a — Vampire: `drain_life()`

**Original:** 60% skip chance; also skips when `hp_max ≤ 30`. When triggered, drains `rand(1, min(current, max))` from both `hp_current` and `hp_max`.

**Rust:** Handler in `advance_world_turn` now checks 60% skip (`rng.rand_percent(60)`) and the `hp_max > 30` guard. ✅

### §11b — Wraith: `drop_level()`

**Original:** 80% skip chance; also skips when `rogue.exp ≤ 5` (player level ≤ 5). HP loss = `hp_raise() = rand(3, 10)`. Drops 2 experience levels.

**Rust:** Handler now checks 80% skip and the level > 5 guard. HP loss uses `rng.get_rand(3, 10)`. ✅

### §11c — Rattlesnake: `sting()`

**Original:** 50% skip; also skips when `str_current ≤ 3`.

**Rust:** Handler checks 50% skip and `str > 3` guard. ✅

### §11d — IceMonster: `freeze()`

**Original:** 12% immunity chance. If not immune, freezes player for `rand(4, FREEZE_TIME=8)` turns. If freeze_percent > 50, player dies of hypothermia.

**Rust:** Handler checks 12% immunity via `rng.rand_percent(12)`. Full hypothermia formula not implemented yet. ⚠️ (partial)

### §11e — Aquator: `rust()`

**Original:** Ring of Maintain Armor prevents rusting entirely. Leather Armor is immune to rust.

**Rust:** Handler now checks for Ring of Maintain Armor and skips rusting for Leather Armor. ✅

---

## §12 — Ring of Regeneration

**Original:** `ring.c / move.c`: When wearing Ring of Regeneration, `heal()` grants +1 extra HP per tick (always, not just on heal-interval turns).

**Rust:** Passive healing loop in `advance_world_turn` checks for equipped Ring of Regeneration and adds +1 to the heal amount. ✅

---

## §13 — Deep Level Scaling (level > 26)

**Original:** Below `AMULET_LEVEL = 26`, monster hit_chance increases slightly. Monsters at very high levels may have enhanced damage or HP, depending on implementation variant.

**Rust:** Not implemented. ❌

---

## §14 — Maximum HP Cap

**Original:** `rogue.h: MAX_HP = 800`. Player `hp_max` can never exceed 800.

**Rust:** `core_types::MAX_HP = 800`. Applied when HP max increases (level-up hp_raise, potion overflow). ✅

---

## Differences from Original

See `CONSISTENCY.md` for all tracked differences. Key remaining deviations:

| # | Description | Status |
|---|---|---|
| §5 | Player always hits (no hit-chance roll) | Not implemented |
| §11d | Hypothermia kill from IceMonster | Not implemented |
| §13 | Level > 26 monster scaling | Not implemented |
| §6 | Weapon-specific damage dice | Not implemented (uses flat bonus) |
