# Analisi Generatore di Mappe: C vs Rust

## Eseguibile Creato ✅

Nuovo binario: `map_viewer` (in `src/bin/map_viewer.rs`)

**Utilizzo:**
```bash
cargo run --bin map_viewer [seed] [num_levels]
```

**Esempi:**
```bash
cargo run --bin map_viewer              # default: seed=42, levels=1
cargo run --bin map_viewer 12345        # seed=12345, levels=1
cargo run --bin map_viewer 42 5         # seed=42, levels=5
```

---

## Comparativa Dettagliata: C vs Rust

### 1. ALGORITMO DI GENERAZIONE

#### C (level.c) - COMPLETO
```c
make_level() {
  // 1. Genera 9 stanze in griglia 3x3 predefinita
  // 2. Aggiunge labirinti casuali (add_mazes)
  // 3. Mescola stanze casualmente (mix_random_rooms)
  // 4. Connette stanze adiacenti in griglia
  //    - Connessione orizzontale (i a i+1)
  //    - Connessione verticale (i a i+3)
  //    - Connessioni diagonali (i a i+2, i a i+6)
  // 5. Riempie aree vuote (fill_out_level)
  // 6. Aggiunge dead ends ricorsivi
}
```

**Stanze predefinite (3x3 grid):**
- Stanza 0,1,2: riga superiore
- Stanza 3,4,5: riga centrale
- Stanza 6,7,8: riga inferiore

#### Rust (world_gen/mod.rs) - SEMPLIFICATO ❌
```rust
pub fn generate_level(rng: &mut GameRng) -> GeneratedLevel {
  // 1. Genera UNA SOLA stanza rettangolare
  // 2. NO labirinti
  // 3. NO corridoi
  // 4. NO dead ends
  // 5. Commento: "Minimal deterministic generation for BL-011"
}
```

### 2. DIMENSIONI STANZE

| Aspetto | C Original | Rust Actual | MATCH? |
|---------|-----------|------------|--------|
| Numero stanze | 9 | 1 | ❌ NO |
| Disposizione | Griglia 3x3 | Casuale | ❌ NO |
| Dimensioni | Predefinite per slot | Randomiche | ❌ PARZIALE |
| Bordi | Muri fissi | Muri fissi | ✅ SÌ |
| Numero corridoi | 4-8 per livello | 0 | ❌ NO |

### 3. STRUTTURE DATI

#### C
```c
struct room {
    char is_room;           // R_ROOM, R_MAZE, R_CROSS, R_DEADEND, R_NOTHING
    short top_row, bottom_row;
    short left_col, right_col;
    struct door {
        short oth_room;     // Stanza connessa
        short oth_row, oth_col;
        short door_row, door_col;
    } doors[4];             // UP, DOWN, LEFT, RIGHT
};
```

#### Rust
```rust
pub struct Room {
    pub top_row: i16,
    pub bottom_row: i16,
    pub left_col: i16,
    pub right_col: i16,
}
// NO: connessioni inter-stanze
// NO: tipo stanza (room/maze/etc)
// NO: porte con coordinate
```

### 4. TILE FLAGS - EQUIVALENTI ✅

```
C               │ Rust              │ Binary
─────────────────┼───────────────────┼─────────────────
NOTHING         │ TileFlags::NOTHING │ 0b0000000000000
OBJECT          │ TileFlags::OBJECT  │ 0b0000000000001
MONSTER         │ TileFlags::MONSTER │ 0b0000000000010
STAIRS          │ TileFlags::STAIRS  │ 0b0000000000100
HORWALL         │ TileFlags::HORWALL │ 0b0000000001000
VERTWALL        │ TileFlags::VERTWALL│ 0b0000000010000
DOOR            │ TileFlags::DOOR    │ 0b0000000100000
FLOOR           │ TileFlags::FLOOR   │ 0b0000001000000
TUNNEL          │ TileFlags::TUNNEL  │ 0b0000010000000
TRAP            │ TileFlags::TRAP    │ 0b0000100000000
HIDDEN          │ TileFlags::HIDDEN  │ 0b0001000000000
```

### 5. ALGORITMI PRINCIPALI - MANCANTI IN RUST

#### A. `connect_rooms()` - MANCANTE
**Funzione:** Connette due stanze adiacenti con corridoi
**Logica:**
- Controlla se stanze sono nella stessa riga o colonna
- Piazza porte sulle pareti
- Disegna passaggio a L con punto di mezzo casuale

**Equivalente Rust:** ❌ NON ESISTE

#### B. `draw_simple_passage()` - MANCANTE
**Funzione:** Disegna corridoi con algoritmo Manhattan (L-shaped)
**Logica:**
```
[Room 1] ===== [Branch] ===== [Room 2]
                   |
                   | (vertical or horizontal)
```

**Equivalente Rust:** ❌ NON ESISTE

#### C. `add_mazes()` - MANCANTE
**Funzione:** Aggiunge labirinti casuali nelle stanze vuote
**Logica:** 
- Probabilità aumenta con profondità dungeon
- Labirinti generati solo per livelli > 1
- Aumentano difficoltà

**Equivalente Rust:** ❌ NON ESISTE

#### D. `fill_out_level()` - MANCANTE
**Funzione:** Riempie aree vuote con dead ends
**Logica:**
- Evita aree disconnesse
- Crea ramificazioni per gameplay
- Usa corridoi nascosti

**Equivalente Rust:** ❌ NON ESISTE

#### E. `make_maze()` - MANCANTE
**Funzione:** Genera labirinti ricorsivi
**Logica:** Backtracking depth-first per creare pattern complessti

**Equivalente Rust:** ❌ NON ESISTE

### 6. DISTRIBUZIONE RANDOM

#### C
```c
char random_rooms[MAXROOMS+1] = { 3,7,5,2,0,6,1,4,8 };

switch(must_exist1) {
    case 0: must_exist[0,1,2];  // Riga 0 garantita
    case 1: must_exist[3,4,5];  // Riga 1 garantita
    case 2: must_exist[6,7,8];  // Riga 2 garantita
    // ... anche per colonne
}
// Garantisce stanze garantite per ogni livello
```

#### Rust
```rust
// Nessun vincolo: stanza casuale ogni volta
let room_height = rng.get_rand(4, 8) as i16;
let room_width = rng.get_rand(8, 20) as i16;
let top = rng.get_rand(1, max_top as i32) as i16;
let left = rng.get_rand(1, max_left as i32) as i16;
```

**Differenza:** Rust non garantisce stanze critiche

---

## EQUIVALENZA FUNZIONALE: ❌ NO

### Livello di implementazione: ~10% del codice C originale

### Cosa è stato implementato:
- ✅ Struttura griglia (24x80)
- ✅ TileFlags (HORWALL, VERTWALL, FLOOR, ecc.)
- ✅ Struct Room basilare
- ✅ Dimensioni costanti (DROWS=24, DCOLS=80, MAXROOMS=9)
- ✅ Spawn position nel centro prima stanza

### Cosa MANCA (90%):
- ❌ Griglia 3x3 di stanze predefinite
- ❌ Sistema di connessioni inter-stanze
- ❌ Corridoi (tunnels) e passaggi
- ❌ Porte con coordinate e connessioni
- ❌ Labirinti
- ❌ Dead ends
- ❌ Sistema di visibilità (hidden passages)
- ❌ Tipo stanza (ROOM, MAZE, CROSS, DEADEND)
- ❌ Algoritmi di connessione

---

## NOTE DALLA CODEBASE RUST

**File:** `src/world_gen/mod.rs` (linee 116-119)
```rust
pub fn generate_level(rng: &mut GameRng) -> GeneratedLevel {
    // Minimal deterministic generation for BL-011:
    // one rectangular room placed via legacy-compatible RNG.
```

**Ticket associato:** BL-011 (evidentemente specifico per scenario minimalista)

**Conclusione:** Questa è un'implementazione parziale intenzionale per un backlog item specifico, NON l'intera generazione di dungeon del Rogue originale.

---

## RACCOMANDAZIONI

### 1. **Per raggiungere equivalenza totale:**
```
1. Riscrivere generate_level() con algoritmo grid 3x3
2. Implementare connect_rooms() e draw_simple_passage()
3. Implementare add_mazes() e make_maze()
4. Implementare fill_out_level() e dead ends
5. Estendere Room struct con connessioni e tipo
6. Aggiungere sistema di connessione room-to-room
```

### 2. **Sforzo stimato:** 8-12 ore di implementazione

### 3. **Test di equivalenza:** 
```bash
# Usando map_viewer per confrontare visivamente
cargo run --bin map_viewer 12345 10  # 10 livelli stesso seed
```

---

## APPENDICE: Output Map Viewer

Il visualizzatore genera mappe come questa per ogni livello:

```
╭────────────────────────────────────────────────────────────────────────────────╮
│................................................................................│
│......────────────────..........................................................│
│......│··············│..........................................................│
│......│··············│..........................................................│
│......─────────────────..........................................................│
│................................................................................│
╰────────────────────────────────────────────────────────────────────────────────╯
```

**Legenda implementata:**
- `.` = Empty
- `─` = Horizontal wall
- `│` = Vertical wall
- `·` = Floor
- `#` = Tunnel (pronto per futura implementazione)
- `+` = Door (pronto per futura implementazione)
