// Game constants ported from rogue.h

pub const DROWS: usize = 24;
pub const DCOLS: usize = 80;
pub const MAX_TITLE_LENGTH: usize = 30;
pub const MAXSYLLABLES: usize = 40;
pub const MAX_METAL: usize = 14;
pub const WAND_MATERIALS: usize = 30;
pub const GEMS: usize = 14;
pub const GOLD_PERCENT: i32 = 46;

// Cell type flags
pub const NOTHING: u16 = 0;
pub const OBJECT: u16 = 0o001;
pub const MONSTER: u16 = 0o002;
pub const STAIRS: u16 = 0o004;
pub const HORWALL: u16 = 0o010;
pub const VERTWALL: u16 = 0o020;
pub const DOOR: u16 = 0o040;
pub const FLOOR: u16 = 0o100;
pub const TUNNEL: u16 = 0o200;
pub const TRAP: u16 = 0o400;
pub const HIDDEN: u16 = 0o1000;

// Item type flags
pub const ARMOR: u16 = 0o001;
pub const WEAPON: u16 = 0o002;
pub const SCROLL: u16 = 0o004;
pub const POTION: u16 = 0o010;
pub const GOLD: u16 = 0o020;
pub const FOOD: u16 = 0o040;
pub const WAND: u16 = 0o100;
pub const RING: u16 = 0o200;
pub const AMULET: u16 = 0o400;
pub const ALL_OBJECTS: u16 = 0o777;

// Armor kinds
pub const LEATHER: u16 = 0;
pub const RINGMAIL: u16 = 1;
pub const SCALE: u16 = 2;
pub const CHAIN: u16 = 3;
pub const BANDED: u16 = 4;
pub const SPLINT: u16 = 5;
pub const PLATE: u16 = 6;
pub const ARMORS: usize = 7;

// Weapon kinds
pub const BOW: u16 = 0;
pub const DART: u16 = 1;
pub const ARROW: u16 = 2;
pub const DAGGER: u16 = 3;
pub const SHURIKEN: u16 = 4;
pub const MACE: u16 = 5;
pub const LONG_SWORD: u16 = 6;
pub const TWO_HANDED_SWORD: u16 = 7;
pub const WEAPONS: usize = 8;

pub const MAX_PACK_COUNT: usize = 24;

// Scroll kinds
pub const PROTECT_ARMOR: u16 = 0;
pub const HOLD_MONSTER: u16 = 1;
pub const ENCH_WEAPON: u16 = 2;
pub const ENCH_ARMOR: u16 = 3;
pub const IDENTIFY: u16 = 4;
pub const TELEPORT: u16 = 5;
pub const SLEEP: u16 = 6;
pub const SCARE_MONSTER: u16 = 7;
pub const REMOVE_CURSE: u16 = 8;
pub const CREATE_MONSTER: u16 = 9;
pub const AGGRAVATE_MONSTER: u16 = 10;
pub const MAGIC_MAPPING: u16 = 11;
pub const SCROLLS: usize = 12;

// Potion kinds
pub const INCREASE_STRENGTH: u16 = 0;
pub const RESTORE_STRENGTH: u16 = 1;
pub const HEALING: u16 = 2;
pub const EXTRA_HEALING: u16 = 3;
pub const POISON: u16 = 4;
pub const RAISE_LEVEL: u16 = 5;
pub const BLINDNESS: u16 = 6;
pub const HALLUCINATION: u16 = 7;
pub const DETECT_MONSTER: u16 = 8;
pub const DETECT_OBJECTS: u16 = 9;
pub const CONFUSION: u16 = 10;
pub const LEVITATION: u16 = 11;
pub const HASTE_SELF: u16 = 12;
pub const SEE_INVISIBLE: u16 = 13;
pub const POTIONS: usize = 14;

// Wand kinds
pub const TELE_AWAY: u16 = 0;
pub const SLOW_MONSTER: u16 = 1;
pub const CONFUSE_MONSTER: u16 = 2;
pub const INVISIBILITY: u16 = 3;
pub const POLYMORPH: u16 = 4;
pub const HASTE_MONSTER: u16 = 5;
pub const PUT_TO_SLEEP: u16 = 6;
pub const MAGIC_MISSILE: u16 = 7;
pub const CANCELLATION: u16 = 8;
pub const DO_NOTHING: u16 = 9;
pub const WANDS: usize = 10;

// Ring kinds
pub const STEALTH: u16 = 0;
pub const R_TELEPORT: u16 = 1;
pub const REGENERATION: u16 = 2;
pub const SLOW_DIGEST: u16 = 3;
pub const ADD_STRENGTH: u16 = 4;
pub const SUSTAIN_STRENGTH: u16 = 5;
pub const DEXTERITY: u16 = 6;
pub const ADORNMENT: u16 = 7;
pub const R_SEE_INVISIBLE: u16 = 8;
pub const MAINTAIN_ARMOR: u16 = 9;
pub const SEARCHING: u16 = 10;
pub const RINGS: usize = 11;

// Food kinds
pub const RATION: u16 = 0;
pub const FRUIT: u16 = 1;

// In-use flags
pub const NOT_USED: u16 = 0;
pub const BEING_WIELDED: u16 = 0o01;
pub const BEING_WORN: u16 = 0o02;
pub const ON_LEFT_HAND: u16 = 0o04;
pub const ON_RIGHT_HAND: u16 = 0o010;
pub const ON_EITHER_HAND: u16 = 0o014;
pub const BEING_USED: u16 = 0o017;

// Trap types
pub const NO_TRAP: i16 = -1;
pub const TRAP_DOOR: i16 = 0;
pub const BEAR_TRAP: i16 = 1;
pub const TELE_TRAP: i16 = 2;
pub const DART_TRAP: i16 = 3;
pub const SLEEPING_GAS_TRAP: i16 = 4;
pub const RUST_TRAP: i16 = 5;
pub const TRAPS: usize = 6;

// Identification status
pub const UNIDENTIFIED: u16 = 0;
pub const IDENTIFIED: u16 = 1;
pub const CALLED: u16 = 2;

// Room flags
pub const R_NOTHING: u16 = 0o01;
pub const R_ROOM: u16 = 0o02;
pub const R_MAZE: u16 = 0o04;
pub const R_DEADEND: u16 = 0o010;
pub const R_CROSS: u16 = 0o020;

pub const MAXROOMS: usize = 9;
pub const BIG_ROOM: usize = 10;
pub const NO_ROOM: i16 = -1;
pub const PASSAGE: i16 = -3;

// Level/character limits
pub const AMULET_LEVEL: i16 = 26;
pub const MAX_EXP_LEVEL: usize = 21;
pub const MAX_EXP: i64 = 10_000_000;
pub const MAX_GOLD: i64 = 900_000;
pub const MAX_ARMOR: i16 = 99;
pub const MAX_HP: i16 = 800;
pub const MAX_STRENGTH: i16 = 99;
pub const LAST_DUNGEON: i16 = 99;
pub const INIT_HP: i16 = 12;

// Stat print flags
pub const STAT_LEVEL: u16 = 0o001;
pub const STAT_GOLD: u16 = 0o002;
pub const STAT_HP: u16 = 0o004;
pub const STAT_STRENGTH: u16 = 0o010;
pub const STAT_ARMOR: u16 = 0o020;
pub const STAT_EXP: u16 = 0o040;
pub const STAT_HUNGER: u16 = 0o100;
pub const STAT_LABEL: u16 = 0o200;
pub const STAT_ALL: u16 = 0o377;

// Party/game constants
pub const PARTY_TIME: i16 = 10;
pub const MAX_TRAPS: usize = 10;
pub const HIDE_PERCENT: i32 = 12;

// Monster flags
pub const HASTED: u64 = 0o01;
pub const SLOWED: u64 = 0o02;
pub const INVISIBLE: u64 = 0o04;
pub const ASLEEP: u64 = 0o010;
pub const WAKENS: u64 = 0o020;
pub const WANDERS: u64 = 0o040;
pub const FLIES: u64 = 0o100;
pub const FLITS: u64 = 0o200;
pub const CAN_FLIT: u64 = 0o400;
pub const CONFUSED_MON: u64 = 0o1000;
pub const RUSTS: u64 = 0o2000;
pub const HOLDS: u64 = 0o4000;
pub const FREEZES: u64 = 0o10000;
pub const STEALS_GOLD: u64 = 0o20000;
pub const STEALS_ITEM: u64 = 0o40000;
pub const STINGS: u64 = 0o100000;
pub const DRAINS_LIFE: u64 = 0o200000;
pub const DROPS_LEVEL: u64 = 0o400000;
pub const SEEKS_GOLD: u64 = 0o1000000;
pub const FREEZING_ROGUE: u64 = 0o2000000;
pub const RUST_VANISHED: u64 = 0o4000000;
pub const CONFUSES: u64 = 0o10000000;
pub const IMITATES: u64 = 0o20000000;
pub const FLAMES: u64 = 0o40000000;
pub const STATIONARY: u64 = 0o100000000;
pub const NAPPING: u64 = 0o200000000;
pub const ALREADY_MOVED: u64 = 0o400000000;

pub const SPECIAL_HIT: u64 =
    RUSTS | HOLDS | FREEZES | STEALS_GOLD | STEALS_ITEM | STINGS | DRAINS_LIFE | DROPS_LEVEL;

pub const MONSTERS: usize = 26;

// Wake percentages
pub const WAKE_PERCENT: i32 = 45;
pub const FLIT_PERCENT: i32 = 33;
pub const PARTY_WAKE_PERCENT: i32 = 75;

// Death causes
pub const HYPOTHERMIA: i32 = 1;
pub const STARVATION: i32 = 2;
pub const POISON_DART: i32 = 3;
pub const QUIT: i32 = 4;
pub const WIN: i32 = 5;

// Directions
pub const UP: u8 = 0;
pub const UPRIGHT: u8 = 1;
pub const RIGHT: u8 = 2;
pub const RIGHTDOWN: u8 = 3;
pub const DOWN: u8 = 4;
pub const DOWNLEFT: u8 = 5;
pub const LEFT: u8 = 6;
pub const LEFTUP: u8 = 7;
pub const DIRS: usize = 8;

// Grid boundaries
pub const ROW1: usize = 7;
pub const ROW2: usize = 15;
pub const COL1: usize = 26;
pub const COL2: usize = 52;

// Move results
pub const MOVED: i32 = 0;
pub const MOVE_FAILED: i32 = -1;
pub const STOPPED_ON_SOMETHING: i32 = -2;

pub const CANCEL: char = '\x1b';

// Hunger levels
pub const HUNGRY: i16 = 300;
pub const WEAK: i16 = 150;
pub const FAINT: i16 = 20;
pub const STARVE: i16 = 0;

pub const MIN_ROW: i16 = 1;
pub const STEALTH_FACTOR: i32 = 3;
pub const R_TELE_PERCENT: i32 = 8;

// Exp level points thresholds
pub const LEVEL_POINTS: [i64; MAX_EXP_LEVEL] = [
    10, 20, 40, 80, 160, 320, 640, 1300, 2600, 5200, 10000, 20000, 40000, 80000, 160000, 320000,
    1000000, 3333333, 6666666, MAX_EXP, 99900000,
];
