use crate::core_types::{Position, TileFlags, DCOLS, DROWS};

pub(super) const DIR_UP: usize = 0;
pub(super) const DIR_RIGHT: usize = 1;
pub(super) const DIR_DOWN: usize = 2;
pub(super) const DIR_LEFT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoorLink {
    pub door_row: i16,
    pub door_col: i16,
    pub oth_room: i16,
    pub oth_row: i16,
    pub oth_col: i16,
}

impl DoorLink {
    pub const NONE: Self = Self {
        door_row: -1,
        door_col: -1,
        oth_room: -1,
        oth_row: -1,
        oth_col: -1,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    pub top_row: i16,
    pub bottom_row: i16,
    pub left_col: i16,
    pub right_col: i16,
    pub slot_index: i16,
    pub doors: [DoorLink; 4],
}

impl Room {
    pub fn new(top_row: i16, bottom_row: i16, left_col: i16, right_col: i16) -> Self {
        Self {
            top_row,
            bottom_row,
            left_col,
            right_col,
            slot_index: -1,
            doors: [DoorLink::NONE; 4],
        }
    }

    pub fn with_slot(top_row: i16, bottom_row: i16, left_col: i16, right_col: i16, slot_index: usize) -> Self {
        let mut room = Self::new(top_row, bottom_row, left_col, right_col);
        room.slot_index = slot_index as i16;
        room
    }

    pub fn with_metadata(
        top_row: i16,
        bottom_row: i16,
        left_col: i16,
        right_col: i16,
        slot_index: i16,
        doors: [DoorLink; 4],
    ) -> Self {
        Self {
            top_row,
            bottom_row,
            left_col,
            right_col,
            slot_index,
            doors,
        }
    }

    pub fn contains(&self, row: i16, col: i16) -> bool {
        row >= self.top_row
            && row <= self.bottom_row
            && col >= self.left_col
            && col <= self.right_col
    }
}

#[derive(Debug, Clone)]
pub struct DungeonGrid {
    pub cells: [[TileFlags; DCOLS]; DROWS],
}

impl DungeonGrid {
    pub fn new() -> Self {
        Self {
            cells: [[TileFlags::NOTHING; DCOLS]; DROWS],
        }
    }

    pub fn in_bounds(row: i16, col: i16) -> bool {
        row >= 0 && row < DROWS as i16 && col >= 0 && col < DCOLS as i16
    }

    pub fn get(&self, row: i16, col: i16) -> Option<TileFlags> {
        if Self::in_bounds(row, col) {
            Some(self.cells[row as usize][col as usize])
        } else {
            None
        }
    }

    pub fn set(&mut self, row: i16, col: i16, flags: TileFlags) -> bool {
        if Self::in_bounds(row, col) {
            self.cells[row as usize][col as usize] = flags;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = TileFlags::NOTHING;
            }
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (DROWS, DCOLS)
    }

    pub fn is_walkable(&self, row: i16, col: i16) -> bool {
        self.get(row, col).is_some_and(|flags| {
            flags.intersects(
                TileFlags::FLOOR
                    | TileFlags::TUNNEL
                    | TileFlags::DOOR
                    | TileFlags::STAIRS
                    | TileFlags::TRAP,
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedLevel {
    pub grid: DungeonGrid,
    pub rooms: Vec<Room>,
    pub stairs_position: Option<Position>,
}

impl GeneratedLevel {
    pub fn spawn_position(&self) -> Position {
        let room = self.rooms[0];
        Position::new(
            (room.top_row + room.bottom_row) / 2,
            (room.left_col + room.right_col) / 2,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SlotKind {
    Nothing,
    Room,
    Maze,
    Cross,
    DeadEnd,
}
