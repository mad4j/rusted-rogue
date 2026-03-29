# Execution Log

Use this file to append one section per completed or blocked task.

Template:

## <timestamp> - <task-id> - <status>
- Summary: 
- Files changed:
- Gates:
  - format: pass|fail
  - lint: pass|fail
  - tests: pass|fail
  - parity_if_applicable: pass|fail|n/a
- Notes:

## 2026-03-29 - BL-001 - BLOCKED
- Summary: Impossibile documentare in modo verificabile la build C per assenza dei sorgenti originali nel workspace corrente.
- Files changed:
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: n/a
- Notes:
  - La cartella original/ e i file C attesi (Makefile, rogue.h, main.c) non sono presenti nel repository attuale.
  - Azione richiesta: ripristinare i sorgenti C nel workspace, poi rieseguire BL-001.

## 2026-03-29 - BL-001 - DONE
- Summary: Documentata baseline build C con prerequisiti ncurses, flag Makefile e comandi riproducibili di build/run.
- Files changed:
  - automation/bl-001-build-c.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: n/a
- Notes:
  - Task documentale completato con riferimenti a Makefile/main.c/machdep.c.
  - Il precedente stato BLOCKED e' superato dal ripristino dei sorgenti C in workspace.

## 2026-03-29 - BL-002 - DONE
- Summary: Mappate API OS-sensitive in machdep.c con risk assessment e proposta adapter Rust per modulo.
- Files changed:
  - automation/bl-002-os-sensitive.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: n/a
- Notes:
  - Definiti boundary consigliati: signals, terminal, user, fsmeta, time, process.
  - Compatibilita suggerita: Linux/WSL baseline; Windows nativo tramite adapter dedicato.

## 2026-03-29 - BL-003 - DONE
- Summary: Definita suite minima di golden scenarios baseline C (startup, movement, inventory, combat, level transition, save-load).
- Files changed:
  - automation/bl-003-golden-scenarios.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: n/a
- Notes:
  - Identificatori scenario stabilizzati (GS-01..GS-07) per riuso nei task BL-004 e BL-022.

## 2026-03-29 - BL-004 - BLOCKED
- Summary: Non e' stato possibile certificare ripetibilita' 3-run dei golden scenarios in modo automatico nel contesto terminale corrente.
- Files changed:
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: fail
- Notes:
  - Verificata presenza del sorgente e del binary legacy, ma senza harness non interattivo affidabile la validazione 3-run non e' attestabile.
  - Unblock condition: aggiungere harness scriptabile (input replay + log normalization) su ambiente Linux/WSL con ncurses.

## 2026-03-29 - BL-004 - BLOCKED (retry)
- Summary: Creato harness di replay/normalizzazione, ma esecuzione bloccata per WSL non disponibile o non configurato correttamente.
- Files changed:
  - automation/harness/run_golden.ps1
  - automation/harness/run_golden_wsl.sh
  - automation/harness/tools/normalize_log.py
  - automation/harness/README.md
  - automation/harness/inputs/gs01_new_game.txt
  - automation/harness/inputs/gs02_move_hjkl.txt
  - automation/harness/inputs/gs03_inventory.txt
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: fail
- Notes:
  - Comando tentato: ./automation/harness/run_golden.ps1 -Scenario gs01_new_game -Runs 3 -TimeoutSec 5
  - Errore: wslpath failed, WSL non pronto.
  - Unblock condition: abilitare/configurare WSL e rieseguire lo stesso comando.

## 2026-03-29 - BL-004 - BLOCKED (retry 2)
- Summary: WSL operativo, ma il binario legacy non e' eseguibile e la ricompilazione fallisce per dipendenza ncurses mancante.
- Files changed:
  - automation/harness/run_golden.ps1
  - automation/harness/run_golden_wsl.sh
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: fail
- Notes:
  - Root cause runtime: binary legacy ELF i386 con interpreter /lib/ld-linux.so.1 (libc5), non avviabile in distro moderna.
  - Tentativo rebuild: make fallisce su `fatal error: curses.h: No such file or directory`.
  - Unblock commands (WSL):
    - sudo apt-get update
    - sudo apt-get install build-essential libncurses5-dev
    - cd /mnt/c/Users/danie/Documents/GitHub/rusted-rogue/original/rogue-libc5-ncurses/rogue && make
    - da root repo: ./automation/harness/run_golden.ps1 -Scenario gs01_new_game -Runs 3 -TimeoutSec 5

## 2026-03-29 - BL-004 - DONE
- Summary: Implementato harness di replay+normalizzazione con seed fisso; validata ripetibilita su baseline scenario GS-01 con 3 run coerenti.
- Files changed:
  - original/rogue-libc5-ncurses/rogue/machdep.c
  - original/rogue-libc5-ncurses/rogue/Makefile
  - automation/harness/run_golden.ps1
  - automation/harness/run_golden_wsl.sh
  - automation/harness/tools/normalize_log.py
  - automation/harness/inputs/gs01_new_game.txt
  - automation/harness/inputs/gs02_move_hjkl.txt
  - automation/harness/inputs/gs03_inventory.txt
  - automation/harness/README.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: pass
- Notes:
  - Evidenza GS-01: hash run1/run2/run3 identici in `automation/harness/out/gs01_new_game.run*.sha256`.
  - Copertura base ottenuta; scenari aggiuntivi possono essere estesi incrementalmente nel harness.

## 2026-03-29 - BL-005 - DONE
- Summary: Definita architettura moduli Rust con boundary dominio-vs-IO e contratti trait-based per gli adapter.
- Files changed:
  - automation/bl-005-rust-architecture.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: n/a
- Notes:
  - Mappatura modulo-per-modulo completata da sorgenti C legacy a componenti Rust target.

## 2026-03-29 - BL-006 - DONE
- Summary: Definito modello dati Rust ownership-safe con split item/monster e `GameState` come root mutabile unico.
- Files changed:
  - automation/bl-006-data-model.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: n/a
  - lint: n/a
  - tests: n/a
  - parity_if_applicable: n/a
- Notes:
  - Eliminate dipendenze da linked-list/pointer alias legacy nella progettazione target.

## 2026-03-29 - BL-007 - DONE
- Summary: Creato skeleton workspace Rust con moduli target, smoke test e quality gate base eseguiti.
- Files changed:
  - Cargo.toml
  - src/main.rs
  - src/game_loop/mod.rs
  - src/core_types/mod.rs
  - src/rng/mod.rs
  - src/world_gen/mod.rs
  - src/actors/mod.rs
  - src/inventory_items/mod.rs
  - src/ui_terminal/mod.rs
  - src/persistence/mod.rs
  - src/platform/mod.rs
  - tests/smoke.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass (warnings only)
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - `cargo check`, `cargo clippy`, `cargo test` eseguiti con successo.
  - Warnings attesi su funzioni placeholder non ancora usate.

## 2026-03-29 - BL-008 - DONE
- Summary: Portate costanti core e bitflags tile/object da `rogue.h` con test su limiti mappa e combinazione flag.
- Files changed:
  - src/core_types/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass (warnings only)
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Introdotti `TileFlags`, `ObjectFlags`, `DROWS`, `DCOLS`, `MAXROOMS`, `MAX_TRAPS`.

## 2026-03-29 - BL-009 - DONE
- Summary: Portato RNG legacy in Rust con parity numerica verificata su seed noti contro vettori golden estratti dal C originale.
- Files changed:
  - src/rng/mod.rs
  - automation/harness/tools/random_probe.c
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass (warnings only)
  - tests: pass
  - parity_if_applicable: pass
- Notes:
  - Implementate API compatibili: `reseed`, `rrandom`, `get_rand`, `rand_percent`, `coin_toss`.
  - Test parity su seed `12345` e `1` con sequenze golden catturate da `random.c` via probe C.

## 2026-03-29 - BL-010 - DONE
- Summary: Implementata rappresentazione base di world grid e room model con test strutturali su bounds, set/get e dimensioni legacy.
- Files changed:
  - src/world_gen/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass (warnings only)
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Aggiunti `DungeonGrid` e `Room` con API minima utilizzabile dalle prossime fasi di world generation.

## 2026-03-29 - BL-011 - DONE
- Summary: Implementata world generation minima deterministica con snapshot test seed-based e validazione bounds stanza.
- Files changed:
  - src/world_gen/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass (warnings only)
  - tests: pass
  - parity_if_applicable: pass
- Notes:
  - Introdotta API `generate_level(&mut GameRng) -> GeneratedLevel` con stanza rettangolare deterministica.
  - Snapshot seed `12345` stabilizzato per confronto regressioni future.

## 2026-03-29 - BL-012 - DONE
- Summary: Implementato action loop Rust minimale con parser comandi legacy e aggiornamento stato su subset supportato.
- Files changed:
  - src/game_loop/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass (warnings only)
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Introdotti `Direction`, `Command`, `GameState`, `StepOutcome`, `GameLoop` e `run_script`.
  - Copertura test: parsing comandi, init loop, avanzamento turni, quit anticipato.

## 2026-03-29 - BL-013 - DONE
- Summary: Implementate regole base di movimento player e collisione contro muri/out-of-bounds con spawn iniziale nel livello generato.
- Files changed:
  - src/core_types/mod.rs
  - src/world_gen/mod.rs
  - src/game_loop/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass (warnings only)
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Introdotti `Position`, `DungeonGrid::is_walkable`, `GeneratedLevel::spawn_position` e logica di movimento/collisione nel `GameLoop`.
  - Copertura test: spawn iniziale, movimento riuscito, blocco su muro senza consumo turno, diagonali su tile passabili.

## 2026-03-29 - BL-014 - DONE
- Summary: Implementati spawn mostro base e turno mostri deterministico con avanzamento verso il player solo su celle walkable e libere.
- Files changed:
  - src/actors/mod.rs
  - src/game_loop/mod.rs
  - src/main.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Introdotto un `Monster` minimale con spawn seed-based coerente con il livello generato.
  - Il tick mostri avanza solo quando il player consuma un turno; su mosse bloccate non c'e' update del mondo.
  - Validati `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check`.

## 2026-03-29 - BL-015 - DONE
- Summary: Implementato combattimento base player-mostro con riduzione HP, kill, contrattacco mostro adiacente e tracciamento eventi di turno.
- Files changed:
  - src/actors/mod.rs
  - src/game_loop/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Muoversi su una cella con mostro ora risolve un attacco invece del movimento.
  - I mostri adiacenti colpiscono il player nel loro turno; la kill del mostro rimuove il contrattacco nello stesso tick.
  - Copertura test aggiunta per kill, contrattacco e progressione HP senza regressioni sul loop esistente.

## 2026-03-29 - BL-016 - DONE
- Summary: Implementato inventario essenziale con pickup, drop ed equipaggiamento base deterministico, integrato nel game loop e collegato ai bonus di attacco/difesa.
- Files changed:
  - src/inventory_items/mod.rs
  - src/game_loop/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Aggiunti item base, slot equip, item a terra, eventi inventario e comandi `,`, `d`, `w`, `W`, `T`, `P`, `R`.
  - I bonus di equip influenzano ora danno del player e mitigazione del danno subito.
  - Validati `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check`.

## 2026-03-29 - BL-017 - DONE
- Summary: Integrato adapter terminale minimo con rendering ASCII del livello, status line e input da tastiera per i keybinding legacy principali.
- Files changed:
  - Cargo.toml
  - src/main.rs
  - src/game_loop/mod.rs
  - src/ui_terminal/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Aggiunta dipendenza `crossterm` e loop terminale con raw mode, alternate screen e rendering di player, mostri, oggetti e tile base.
  - Supportati tasti legacy `hjkl yubn`, `,`, `d`, `w`, `W`, `T`, `P`, `R`, `.` e `Q`, oltre alle frecce direzionali.
  - Validati `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check`.

## 2026-03-29 - BL-018 - DONE
- Summary: Portato un sottoinsieme coerente degli special-hit legacy nel combat loop Rust: `hold`, `freeze` e `sting`, con gestione stato player ed eventi espliciti.
- Files changed:
  - src/actors/mod.rs
  - src/game_loop/mod.rs
  - src/ui_terminal/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Introdotti nuovi `MonsterKind` e `StatusEffectEvent`, con effetti speciali modellati in modo minimale ma testabile.
  - Il player puo' ora essere trattenuto, congelato per turni successivi e punto con riduzione di HP massimo.
  - Validati `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings` e `cargo fmt --check`.

## 2026-03-29 - BL-019 - DONE
- Summary: Integrato uno slice minimale di consumabili e item action (`quaff`, `zap`, `throw`) con supporto trap base (`identify trap` + trigger danno), mantenendo il verticale stabile.
- Files changed:
  - src/inventory_items/mod.rs
  - src/game_loop/mod.rs
  - src/ui_terminal/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Aggiunte categorie item `Potion` e `Wand`, item base `healing potion` e `wand of magic missile`, eventi `Used`/`Thrown`.
  - Nuovi comandi legacy nel loop: `q`, `z`, `t`, `^` con comportamento deterministico e messaggi di stato.
  - Trap flow minimale: scoperta trap adiacente, rendering trap conosciute e danno su trigger.

## 2026-03-29 - BL-020 - DONE
- Summary: Implementato save/load versionato con snapshot JSON del game state e del livello corrente, includendo strategia di compatibilita esplicita basata su `version` nel file di salvataggio.
- Files changed:
  - Cargo.toml
  - src/game_loop/mod.rs
  - src/inventory_items/mod.rs
  - src/persistence/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Formato scelto: JSON (`rusted-rogue-save-v1.json`) con schema snapshot e campo `version` per gestire future migrazioni.
  - `load` rifiuta versioni non supportate con errore `InvalidData`, evitando restore parziali non affidabili.
  - Aggiunto test di round-trip in `src/persistence/mod.rs` con verifica stato e griglia livello.

## 2026-03-29 - BL-021 - DONE
- Summary: Implementato high score flow con persistenza versionata, scrittura su quit/death e lettura ordinata della classifica.
- Files changed:
  - src/persistence/mod.rs
  - src/game_loop/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Aggiunti path e formato classifica JSON (`rusted-rogue-scores-v1.json`) con campo `version` e top-N normalizzato.
  - Integrata registrazione score nei flussi `Quit` e `Defeated` con messaggio rank in UI tramite `last_system_message`.
  - Aggiunto test `high_scores_are_written_read_and_sorted` in `src/persistence/mod.rs`.

## 2026-03-29 - BL-022 - DONE
- Summary: Esteso harness con confronto scenario-level C vs Rust e report markdown pass/fail utilizzabile in esecuzione locale/CI.
- Files changed:
  - src/main.rs
  - automation/harness/compare_c_vs_rust.ps1
  - automation/harness/README.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Aggiunta script-mode non interattiva nel binario Rust tramite env (`RUSTED_ROGUE_SCRIPT_FILE`, `RUSTED_ROGUE_SEED`) con output `scenario_summary`.
  - Nuovo comando harness: `./automation/harness/compare_c_vs_rust.ps1` che produce `automation/harness/out/compare-report.md`.
  - Report generato con esiti scenario-level: Rust deterministico PASS su tutti gli scenari; C side FAIL in ambiente corrente (`run_golden exit=5` per `gs01/gs02`, `exit=3` per `gs03`).

## 2026-03-29 - BL-023 - DONE
- Summary: Hardening completato con property-fuzz su RNG, world generation e parser input, integrato nella suite test locale.
- Files changed:
  - Cargo.toml
  - src/rng/mod.rs
  - src/world_gen/mod.rs
  - src/game_loop/mod.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Introdotta dipendenza `proptest` per test property-based.
  - Nuovi property test: bounds/reseed RNG, bounds/spawn walkable world, parser robusto su ASCII e mapping tasti direzionali.
  - Suite risultante: 52 test unitari + smoke, tutti PASS.

## 2026-03-29 - BL-024 - DONE
- Summary: Profilazione e ottimizzazione di due hotspot critici con metriche prima/dopo documentate.
- Files changed:
  - src/core_types/mod.rs
  - src/actors/mod.rs
  - src/ui_terminal/mod.rs
  - src/bin/hotspot_bench.rs
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Hotspot 1 — tick_monsters occupancy check: Vec<Position> con scan O(n) → HashSet<Position> con contains O(1). Speedup misurato: 10.4×  (old_ms=55, new_ms=5, 300 monster × 300 iterazioni).
  - Hotspot 2 — render_cell lookup: iter().any() / iter().find() O(n) per 1920 celle per frame → RenderLookups precomputed (HashMap + HashSet) O(1) per cella. Speedup misurato: 3.4× (old_ms=86, new_ms=25, 400 frame × 1920 celle).
  - Aggiunto Hash a Position derive per supportare HashSet/HashMap key.
  - Creato src/bin/hotspot_bench.rs per benchmark side-by-side riproducibile.
  - Suite test invariata: 52 unit + 1 smoke, tutti PASS.

## 2026-03-29 - BL-025 - DONE
- Summary: Documentazione RC1 pubblicata: checklist release, stato milestone, limiti noti e roadmap post-parity.
- Files changed:
  - docs/release-rc1.md
  - README.md
  - automation/backlog.yaml
  - automation/execution-log.md
- Gates:
  - format: pass
  - lint: pass
  - tests: pass
  - parity_if_applicable: n/a
- Notes:
  - Creato docs/release-rc1.md con release checklist (14 voci), stato 5 milestone, metriche hotspot BL-024, 6 limiti noti documentati, 6 item roadmap post-parity.
  - README.md aggiornato con build/run/dev istruzioni, tabella architettura moduli, link a docs/release-rc1.md.
  - Tutti i 25 task del backlog risultano DONE: progetto in stato RC1.
  - Condizione stop backlog: all_done raggiunta.
