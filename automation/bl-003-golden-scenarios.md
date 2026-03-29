# BL-003 - Golden scenarios baseline C

## Scope
Define a minimal, reproducible scenario set to validate behavior parity between legacy C and Rust port.

## Inputs used
- original/rogue-libc5-ncurses/rogue/main.c
- original/rogue-libc5-ncurses/rogue/play.c
- original/rogue-libc5-ncurses/rogue/move.c
- original/rogue-libc5-ncurses/rogue/hit.c
- original/rogue-libc5-ncurses/rogue/pack.c
- original/rogue-libc5-ncurses/rogue/save.c

## Scenario set (minimum)

### GS-01 - New game startup
- Goal: verify game starts and enters play loop.
- Expected:
  - player initialized
  - level generated
  - stats rendered

### GS-02 - 8-direction movement
- Goal: validate movement commands hjklyubn.
- Expected:
  - player position changes according to direction rules
  - illegal moves are blocked without crash

### GS-03 - Pickup and inventory display
- Goal: validate pickup flow and inventory list.
- Expected:
  - item can be picked when present
  - inventory command shows item and state remains consistent

### GS-04 - Drop and equip basic
- Goal: validate drop/wield/wear baseline interactions.
- Expected:
  - dropped item leaves inventory and appears on map
  - equip action updates active slots/state

### GS-05 - Basic combat
- Goal: validate attack resolution path.
- Expected:
  - combat command drives hit/miss and hp updates
  - death/cleanup path does not corrupt state

### GS-06 - Level transition
- Goal: validate stairs/trap door transition lifecycle.
- Expected:
  - transition triggers level rebuild flow
  - objects/monsters cleanup and re-init occur

### GS-07 - Save and load
- Goal: validate persistence baseline flow.
- Expected:
  - save writes file successfully
  - load restores playable state and resumes loop

## Acceptance criteria for BL-003
- All seven scenarios are defined with goal and expected behavior.
- Scenarios map directly to gameplay loops and command dispatcher.
- Scenario identifiers are stable and can be reused in BL-004 and BL-022.

## Next step
BL-004 will make these scenarios deterministic and repeatable with fixed seed and 3-run consistency checks.
