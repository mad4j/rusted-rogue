# Gap Analysis Map Generator: Rust vs Rogue C (Aggiornata)

Data analisi: 2026-03-30

## Sintesi Esecutiva

La versione Rust attuale ha gia implementato la parte strutturale principale della generazione (griglia 3x3, room slot, corridoi, maze depth-aware, cross-slot e fill dei vuoti), quindi il documento precedente non e piu valido.

Il gap residuo non e su "esistenza delle feature", ma su fedelta comportamentale rispetto a `level.c`.

Stima di parita attuale (solo map generation): ~70-80%.

## Stato Attuale in Rust

Implementato in `src/world_gen/mod.rs`:

- Generazione stanza per slot 3x3 con vincolo di 3 slot obbligatori (`required_room_group`).
- Connessione stanze con porte + corridoio a L (`connect_rooms`, `draw_simple_passage`).
- Maze sui livelli profondi con formula legacy della probabilita (`add_mazes`, `maze_percent_for_level`).
- Fill delle aree "Nothing/Cross" con tunnel di dead-end (`fill_out_level`).
- Test di coerenza/determinismo/proprieta (reachability, dead-end, depth->tunnel).

## Gap Reali vs C Originale

Riferimento C: `original/rogue-libc5-ncurses/rogue/level.c`

1. Ordine di connessione stanze non allineato
- C usa `mix_random_rooms()` e interrompe quando `is_all_connected()` diventa vero.
- Rust connette in ordine fisso `0..8`, senza break anticipato per connettivita globale.
- Impatto: distribuzione topologica dei corridoi diversa da quella legacy.

2. Assenza metadati porta->stanza
- C mantiene `rooms[].doors[]` con `oth_room/oth_row/oth_col`.
- Rust disegna porte sul grid ma non conserva grafo porte/stanze.
- Impatto: comportamenti futuri dipendenti da porte (AI, pathing, room wakeup) non allineabili 1:1.

3. Big Room non implementata
- C attiva `BIG_ROOM` quando `cur_level == party_counter` con probabilita 1%.
- Rust non ha il ramo big-room nella generazione.
- Impatto: manca una variante rara ma iconica del layout.

4. Maze algorithm differente
- C usa `make_maze()` ricorsiva con vincoli locali su celle adiacenti.
- Rust usa DFS iterativo a step 2.
- Impatto: densita e pattern dei maze non identici a seed parita.

5. Fill/dead-end semplificato
- C usa `fill_it`, `recursive_deadend`, `mask_room`, coin toss, e pass secondario `r_de`.
- Rust usa una versione semplificata a singolo collegamento e fallback.
- Impatto: minor varianza nella forma dei rami ciechi.

6. Hidden passages/doors non applicati come in C
- C applica `HIDDEN` su porte e tunnel (`put_door`, `hide_boxed_passage`) con condizioni di livello.
- Rust espone `TileFlags::HIDDEN` ma la logica equivalente non e completa nella pipeline map-gen.
- Impatto: differenza su visibilita/esplorazione.

7. Compatibilita room-connection con slot maze
- In C, `connect_rooms` opera su `R_ROOM | R_MAZE`.
- In Rust, `connect_rooms` opera su `Option<Room>` (quindi room-room), mentre i collegamenti verso slot non-room sono gestiti in `fill_out_level`.
- Impatto: meccanica simile ma non equivalente nella sequenza di connessione.

## Passi Implementativi per Coprire il Gap

### Fase 1: Allineamento topologia connessioni (alta priorita)

1. Introdurre `random_rooms` e shuffle legacy-compatible prima delle connessioni principali.
2. Eseguire il loop connessioni su ordine mescolato (come C).
3. Implementare `is_all_connected()` lato Rust (BFS su room centers o su grafo slot).
4. Interrompere il loop connessioni quando il livello e connesso.

Criterio di accettazione:
- Seed window (es. 0..1024) con media componenti connesse e numero corridoi piu vicine al C baseline.

### Fase 2: Big Room e varianti strutturali

1. Introdurre un ramo opzionale `big_room` in `generate_level_with_depth` con probabilita/condizione configurabile.
2. Parametrizzare la condizione in modo da poterla guidare dai futuri state variables (equivalente di `party_counter`).
3. Aggiungere test dedicato che forzi la branch big-room (feature flag o RNG stub).

Criterio di accettazione:
- Possibilita di produrre layout big-room in test deterministici.

### Fase 3: Metadati porte/stanze

1. Estendere modello room con `doors[4]` (o struttura equivalente Rust-safe).
2. Salvare `door_row/door_col` e mapping verso stanza opposta in `connect_rooms`.
3. Aggiornare serializzazione/persistenza se necessario.

Criterio di accettazione:
- Invariant: per ogni connessione A->B esiste il reverse B->A coerente.

### Fase 4: Fill/dead-end parity

1. Portare `mask_room` per scegliere start point da tunnel gia presente quando opportuno.
2. Portare la logica `recursive_deadend` con `did_this`, `rooms_found`, `coin_toss` e pass secondario.
3. Mantenere il comportamento attuale dietro feature flag finche non si valida la parity.

Criterio di accettazione:
- Distribuzione lunghezza dead-end e branching factor simile al C su seed window condivisa.

### Fase 5: Maze parity e hidden logic

1. Aggiungere un percorso "strict_legacy_maze" che replica `make_maze` (ricorsiva + stesse guardie locali).
2. Portare `hide_boxed_passage` e hidden door policy con soglia livello.
3. Validare che il conteggio tile `HIDDEN` rientri in intervalli attesi.

Criterio di accettazione:
- Statistiche maze/hidden comparabili con baseline C (non per-cell equality, ma distribuzioni).

### Fase 6: Golden parity harness

1. Estendere harness in `automation/harness` per confrontare metriche aggregate C vs Rust su N seed.
2. Tracciare almeno: numero room, numero maze slot, numero tunnel, numero door, componenti connesse, dead-end count, hidden count.
3. Definire soglie accettabili e fail CI se fuori soglia.

Criterio di accettazione:
- Report automatico di regressione parity ad ogni modifica di map generation.

## Ordine Raccomandato di Esecuzione

1. Fase 1 (topologia connessioni)
2. Fase 3 (metadati porte)
3. Fase 4 (fill/dead-end)
4. Fase 5 (maze strict + hidden)
5. Fase 2 (big-room, dipende da game state)
6. Fase 6 (harness definitivo)

Motivo: prima si allinea il backbone del grafo, poi i dettagli di varianza layout.

## Comandi Utili

```bash
cargo test world_gen -- --nocapture
cargo run --bin map_viewer 12345 5
```

Per confronto con C usare gli script in `automation/harness/`.
