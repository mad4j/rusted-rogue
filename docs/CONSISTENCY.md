# Consistency: Original C vs. Rust Port

This document lists notable differences between the original C version
(`original/rogue-libc5-ncurses/`) and the Rust port (`src/`).  Its purpose
is to help contributors understand intentional deviations and known gaps.

---

## Implemented and Consistent

| Feature | Status | Notes |
|---|---|---|
| Map generation (rooms+passages) | ✓ Consistent | Dungeon layout algorithm matches original |
| Monster movement / AI | ✓ Consistent | |
| Combat (hit/damage dice) | ✓ Consistent | |
| Inventory management | ✓ Consistent | |
| Auto-pickup on step | ✓ Consistent | Items and gold are auto-collected when the player steps onto their tile, matching `one_move_rogue()` / `pick_up()` in `pack.c` / `move.c`; `,` still works as a manual fallback |
| Food/hunger system | ✓ Consistent | |
| Traps | ✓ Consistent | |
| Potions / scrolls / wands | ✓ Consistent | |
| Save / restore | ✓ Consistent | |
| High-score file | ✓ Consistent | JSON format instead of binary |
| Wizard mode | ✓ Consistent | See section below |
| Orc SEEKS_GOLD | ✓ Consistent | Orc navigates toward floor gold in its room before pursuing the player; `seeks_gold` flag cleared on reaching gold or being attacked |

---

## Intentional Differences

### UI framework
- **Original**: ncurses terminal  
- **Rust port**: [iced](https://github.com/iced-rs/iced) GPU-accelerated canvas  
- Arrow keys are supported in addition to vi-keys (`hjkl`).

### Save-file format
- **Original**: Binary struct dump  
- **Rust port**: JSON (`saves/rusted-rogue-scores-v1.json`)

### Wizard mode activation
- **Original**: The password is obfuscated with the `xxx()/xxxx()` cipher
  (hardcoded cipher-text `"\247\104\126\272\115\243\027"`; plaintext
  **"bathtub"**).  
- **Rust port**: Same cipher and same plaintext password are implemented in
  `wizard_check_password()` in `src/game_loop/mod.rs`.

### Wizard mode – Ctrl+C (add random item)
- **Original**: Ctrl+C calls `new_object_for_wizard()` to drop a random item
  at the player's feet.  
- **Rust port**: **Not yet implemented** (item generation helper not yet
  exposed; tracked as a backlog item).

### Wizard mode – Ctrl+M (invoke monster)
- **Original**: Ctrl+M calls `show_monsters()` which reveals all monster
  positions on the current level.  
- **Rust port**: Implemented as `WizardShowMonsters`; highlights all monsters
  using the existing reveal overlay mechanism.

### Help pages
- **Original**: Single `?`-triggered text dump listing all commands.  
- **Rust port**: Five-page help overlay navigated with arrow keys; wizard
  bindings appear on a dedicated sixth page that is only shown when wizard
  mode is active. `Ctrl+P` recalls the last displayed message (no game turn
  consumed).

---

## Known Gaps / Not Yet Implemented

| Feature | Notes |
|---|---|
| Wizard Ctrl+C (random item) | `new_object_for_wizard()` equivalent not implemented |
| Ring diagnostics (Ctrl+R in original) | `ring.c` `ring_stats()` diagnostic not implemented |
| Score-only flag in saved game | `score_only` is not persisted to disk (resets on load) |
| Wizard flag persistence | `wizard` flag is not persisted; reload always starts non-wizard |
