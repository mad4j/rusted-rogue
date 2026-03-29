# BL-006 - Rust data model (split item/monster + GameState)

## Objective
Replace legacy overloaded `struct obj` usage with explicit Rust types and a single ownership root (`GameState`).

## Design

### Core types
- `Position { row: i16, col: i16 }`
- `TileFlags` bitflags for dungeon cells.
- `Room`, `Door`, `Trap` as dedicated structs.

### Actor model
- `Player` and `Monster` are separate types.
- `MonsterKind` enum replaces character-based monster identity.
- `MonsterState` stores status flags and combat-related transient state.

### Item model
- `Item` is separate from monsters.
- `ItemKind` enum with variants:
  - Armor
  - Weapon
  - Scroll
  - Potion
  - Wand
  - Ring
  - Gold
  - Food
  - Amulet

### Inventory model
- `Inventory` owns `Vec<Item>` and equip slots:
  - `equipped_weapon: Option<ItemId>`
  - `equipped_armor: Option<ItemId>`
  - `left_ring: Option<ItemId>`
  - `right_ring: Option<ItemId>`

### Entity identity
- Stable IDs:
  - `MonsterId(u32)`
  - `ItemId(u32)`
- Avoid linked-list pointer semantics from C.

## GameState ownership root

```rust
pub struct GameState {
    pub dungeon: DungeonGrid,
    pub rooms: Vec<Room>,
    pub traps: Vec<Trap>,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub floor_items: Vec<Item>,
    pub level: i16,
    pub rng: GameRng,
    pub flags: RuntimeFlags,
}
```

## Boundary and mutability rules
- Domain systems mutate only `&mut GameState`.
- No global mutable state.
- Platform side effects go through adapter traits.

## Migration mapping from legacy C
- `fighter rogue` -> `Player`
- `object level_monsters` list -> `Vec<Monster>`
- `object level_objects` list -> `Vec<Item>`
- `object pack` linked list -> `Inventory`

## DoD coverage
- Split item/monster achieved conceptually.
- Ownership model centered on `GameState` defined.
- Global mutable state elimination strategy documented.
