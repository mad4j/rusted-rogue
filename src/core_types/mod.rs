use bitflags::bitflags;

pub const DROWS: usize = 24;
pub const DCOLS: usize = 80;
pub const MAXROOMS: usize = 9;
pub const MAX_TRAPS: usize = 10;
pub const MIN_ROW: i16 = 1;
pub const ROW1: i16 = 7;
pub const ROW2: i16 = 15;
pub const COL1: i16 = 26;
pub const COL2: i16 = 52;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: i16,
    pub col: i16,
}

impl Position {
    pub const fn new(row: i16, col: i16) -> Self {
        Self { row, col }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TileFlags: u16 {
        const NOTHING  = 0;
        const OBJECT   = 0b000000000001;
        const MONSTER  = 0b000000000010;
        const STAIRS   = 0b000000000100;
        const HORWALL  = 0b000000001000;
        const VERTWALL = 0b000000010000;
        const DOOR     = 0b000000100000;
        const FLOOR    = 0b000001000000;
        const TUNNEL   = 0b000010000000;
        const TRAP     = 0b000100000000;
        const HIDDEN   = 0b001000000000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ObjectFlags: u16 {
        const ARMOR       = 0b000000001;
        const WEAPON      = 0b000000010;
        const SCROLL      = 0b000000100;
        const POTION      = 0b000001000;
        const GOLD        = 0b000010000;
        const FOOD        = 0b000100000;
        const WAND        = 0b001000000;
        const RING        = 0b010000000;
        const AMULET      = 0b100000000;
    }
}

#[cfg(test)]
mod tests {
    use super::{COL1, COL2, MAXROOMS, MAX_TRAPS, MIN_ROW, ObjectFlags, ROW1, ROW2, TileFlags, DCOLS, DROWS};

    #[test]
    fn map_limits_match_legacy() {
        assert_eq!(DROWS, 24);
        assert_eq!(DCOLS, 80);
        assert_eq!(MAXROOMS, 9);
        assert_eq!(MAX_TRAPS, 10);
        assert_eq!(MIN_ROW, 1);
        assert_eq!(ROW1, 7);
        assert_eq!(ROW2, 15);
        assert_eq!(COL1, 26);
        assert_eq!(COL2, 52);
    }

    #[test]
    fn tile_flags_can_be_combined() {
        let cell = TileFlags::DOOR | TileFlags::TUNNEL;
        assert!(cell.contains(TileFlags::DOOR));
        assert!(cell.contains(TileFlags::TUNNEL));
        assert!(!cell.contains(TileFlags::MONSTER));
    }

    #[test]
    fn object_flags_cover_all_base_categories() {
        let all = ObjectFlags::ARMOR
            | ObjectFlags::WEAPON
            | ObjectFlags::SCROLL
            | ObjectFlags::POTION
            | ObjectFlags::GOLD
            | ObjectFlags::FOOD
            | ObjectFlags::WAND
            | ObjectFlags::RING
            | ObjectFlags::AMULET;

        assert!(all.contains(ObjectFlags::RING));
        assert!(all.contains(ObjectFlags::AMULET));
        assert!(!all.is_empty());
    }
}
