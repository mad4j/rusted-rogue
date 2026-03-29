# rusted-rogue — Release Candidate 1 (RC1)

**Date:** 2026-03-29  
**Branch:** `main`  
**Milestone coverage:** M1 – M5 (BL-001 → BL-025)

---

## Release checklist

| # | Item | Status |
|---|------|--------|
| 1 | Tutti i 25 task del backlog completati (DONE) | ✅ |
| 2 | `cargo fmt -- --check` pulito (zero diff) | ✅ |
| 3 | `cargo clippy --all-targets --all-features -- -D warnings` pulito | ✅ |
| 4 | Suite test: 52 unit + 1 smoke = 53 PASS, 0 FAIL | ✅ |
| 5 | Property-based fuzz su RNG, world gen, input parser (proptest) | ✅ |
| 6 | RNG parity numerica seed-based rispetto al C legacy | ✅ |
| 7 | World generation deterministica e snapshot-testata | ✅ |
| 8 | Inventario: pick / drop / equip / use / throw / wand / ring | ✅ |
| 9 | Combattimento: hit resolution, effetti speciali (sting, freeze, hold) | ✅ |
| 10 | Save / load round-trip stabile (formato JSON) | ✅ |
| 11 | High score: write / read / sort | ✅ |
| 12 | Harness comparativo C vs Rust su golden scenarios | ✅ |
| 13 | Profilazione hotspot con metriche prima/dopo | ✅ |
| 14 | Documentazione RC e limiti noti (questo file) | ✅ |

---

## Milestone status

| Milestone | Task | Descrizione | Status |
|-----------|------|-------------|--------|
| M1 | BL-001 → BL-004 | Analisi baseline C, golden scenarios, harness replay | ✅ DONE |
| M2 | BL-005 → BL-011 | Architettura Rust, modello dati, skeleton, RNG, world gen | ✅ DONE |
| M3 | BL-012 → BL-017 | Action loop, movimento, mostri, combat base, inventario, rendering | ✅ DONE |
| M4 | BL-018 → BL-021 | Special-hit, consumabili/wand/ring/trap, save-load, high score | ✅ DONE |
| M5 | BL-022 → BL-025 | Harness comparativo, fuzz, ottimizzazione hotspot, RC docs | ✅ DONE |

---

## Hotspot — metriche prima/dopo (BL-024)

| Hotspot | Struttura old | Struttura new | Speedup |
|---------|--------------|--------------|---------|
| `tick_monsters` — occupancy check | `Vec<Position>` scan O(n) | `HashSet<Position>` contains O(1) | **10.4×** |
| `render_cell` — lookup per cella | `iter().any()` × 1920 celle/frame | `RenderLookups` precomputed `HashMap`/`HashSet` | **3.4×** |

Benchmark riproducibile: `cargo run --release --bin hotspot_bench`

---

## Limiti noti

### L1 — Platform module parzialmente stub
`src/platform/mod.rs` espone solo `init_platform()` placeholder. Le API OS-sensitive mappate in BL-002
(signal handling, user identity, process/env metadata, file-system paths) non sono portate su Windows nativo.
Il gioco funziona perché `game_loop` non le invoca durante il normale flusso — ma eventuali estensioni
che richiedono SIGWINCH, uid/gid o paths riconfigurables dovranno completare questo modulo.

### L2 — Formato di salvataggio non compatibile con il C legacy
Il salvataggio usa JSON (decisione presa in BL-020).
I file `.rogue` prodotti dal binario C originale non sono caricabili dalla versione Rust e viceversa.

### L3 — Parity C↔Rust unilaterale
La verifica parity è deterministica solo lato Rust (tutti gli scenari PASS).
Il binario C legacy (ELF i386 con libc5) non è eseguibile in distribuzioni Linux moderne senza
emulazione x86 e dipendenze ncurses legacy; il confronto bilateral completo richiede un ambiente
di build appositamente configurato (vedi `automation/harness/README.md`).

### L4 — Rendering richiede terminale ANSI-compatible
Il backend usa `crossterm` (ANSI escape codes), non ncurses.
Su Windows è richiesta la versione 10 1809+ con Virtual Terminal Processing abilitato.
In ambienti SSH o emulatori non standard i colori/la formattazione potrebbero non essere corretti.

### L5 — Script-mode bypassa il render path
I test automatizzati usano la script-mode che salta le chiamate crossterm.
Il render interattivo è validato solo manualmente: non è coperto dalla suite `cargo test`.

### L6 — Transizione multi-livello non esaustivamente validata
La logica di staircase/livello successivo è implementata, ma non è stata oggetto di golden-scenario
parity testing contro il C originale. Il comportamento è corretto per i casi comuni ma non certificato
su tutti gli edge case (amulet of Yendor, livelli profondi, re-seed a ogni livello).

---

## Roadmap post-parity

Le seguenti aree non fanno parte del piano parity RC1 ma sono candidate naturali per iterazioni successive.

### P1 — Completamento platform adapter
Implementare `signals`, `user identity` e `fs paths` in `src/platform/` per supportabilità
su Linux/macOS senza residue limitazioni dei limiti L1.

### P2 — Backend ncurses (Linux)
Aggiungere un feature flag `ncurses` per chi vuole il comportamento di rendering originale
su terminal Unix. L'attuale backend crossterm rimarrebbe come default Windows/cross-platform.

### P3 — Golden scenario parity bilaterale
Configurare un CI che compili il C legacy con ncurses in un container Docker (Debian bookworm + multilib)
e confronti automaticamente gli output C vs Rust su tutti i GS-01..GS-07.

### P4 — Validazione progressione multi-livello
Estendere la suite di test e il harness per coprire il flusso completo livello 1→26
con seed fisso, verificando spawn, amulet, escape finale e calcolo punteggio.

### P5 — Target WASM / browser
Esportare la logica di dominio (tutto tranne `ui_terminal` e `platform`) come crate WASM
con un frontend HTML5 canvas per permettere il gioco in browser.

### P6 — Compatibilità salvataggio
Implementare un parser del formato binario C `.rogue` per abilitare la migrazione
dei save file esistenti nel nuovo formato JSON.
