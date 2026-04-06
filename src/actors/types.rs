use crate::core_types::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterKind {
    Aquator,
    Bat,
    Centaur,
    Dragon,
    Emu,
    VenusFlytrap,
    Griffin,
    Hobgoblin,
    IceMonster,
    Jabberwock,
    Kestrel,
    Leprechaun,
    Medusa,
    Nymph,
    Orc,
    Phantom,
    Quagga,
    Rattlesnake,
    Snake,
    Troll,
    BlackUnicorn,
    Vampire,
    Wraith,
    Xeroc,
    Yeti,
    Zombie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialHit {
    Freeze,
    Hold,
    Sting,
    Rusts,
    StealsGold,
    StealsItem,
    DrainsLife,
    DropsLevel,
    /// Medusa: confuses the player from sight range (CONFUSES flag)
    Confuse,
    /// Dragon: breathes fire at range along a straight or diagonal line (FLAMES flag)
    Flames,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectEvent {
    Frozen { turns: u8 },
    Held,
    /// Rattlesnake sting: drains player strength by `amount`.
    Stung { amount: i16 },
    ArmorRusted,
    GoldStolen,
    ItemStolen,
    LifeDrained { max_hit_points_lost: i16 },
    LevelDropped,
    /// Medusa gaze: confuses the player for `turns` moves.
    Confused { turns: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEvent {
    PlayerHitMonster {
        monster_kind: MonsterKind,
        position: Position,
        damage: i16,
        killed: bool,
        kill_exp: i32,
    },
    PlayerMissedMonster {
        monster_kind: MonsterKind,
        position: Position,
    },
    MonsterHitPlayer {
        monster_kind: MonsterKind,
        position: Position,
        damage: i16,
    },
    MonsterMissedPlayer {
        monster_kind: MonsterKind,
        position: Position,
    },
    MonsterAppliedEffect {
        monster_kind: MonsterKind,
        position: Position,
        effect: StatusEffectEvent,
    },
}
