# BL-002 - OS-sensitive surface mapping from machdep.c

## Scope
Map OS-sensitive APIs used by the legacy C code and propose Rust adapter boundaries for migration.

## Source
- original/rogue-libc5-ncurses/rogue/machdep.c

## API inventory by category

### Signals
- Used APIs: signal(), kill()
- Used signals: SIGINT, SIGQUIT, SIGHUP, SIGTSTP
- Relevant functions: md_heed_signals(), md_ignore_signals(), md_control_keybord(), md_tstp()
- Risk: HIGH
- Why: async signal handling with global mutable state is unsafe in direct Rust translation.

### User and identity
- Used APIs: getuid(), getpwuid()
- Relevant function: md_gln()
- Risk: MEDIUM
- Why: Unix-specific behavior and portability differences on Windows.

### Terminal and TTY control
- Used APIs: tcgetattr(), tcsetattr(), ioctl() legacy branches
- Relevant functions: md_control_keybord(), md_cbreak_no_echo_nonl(), md_slurp()
- Risk: HIGH (portability), MEDIUM (Linux-only migration)
- Why: termios/ioctl paths are POSIX/BSD-centric and not portable to native Windows.

### File and path metadata
- Used APIs: stat(), unlink()
- Relevant functions: md_get_file_id(), md_link_count(), md_gfmt(), md_df()
- Risk: LOW to MEDIUM
- Why: mostly mappable to Rust std::fs, but inode/link semantics vary by platform.

### Time
- Used APIs: gettimeofday(), localtime()
- Relevant functions: md_gct(), md_gfmt()
- Risk: LOW
- Why: straightforward replacement with Rust time APIs.

### Process and environment
- Used APIs: getpid(), sleep(), getenv(), exit()
- Relevant functions: md_gseed(), md_sleep(), md_getenv(), md_exit()
- Risk: LOW to MEDIUM
- Why: portable wrappers exist, but behavior for seed and env should be normalized.

## Rust adapter proposal

### Module boundary proposal
- src/platform/signals.rs
- src/platform/terminal.rs
- src/platform/user.rs
- src/platform/fsmeta.rs
- src/platform/time.rs
- src/platform/process.rs

### Adapter responsibilities
- signals.rs
  - Register handlers and expose safe flags/events for game loop consumption.
  - Use feature-gated unix implementation; fallback no-op or ctrl-c handling on non-unix.
- terminal.rs
  - Abstract cbreak/raw, echo, and keyboard control.
  - Unix implementation via termios crate path; non-unix backend adapter later.
- user.rs
  - Provide get_login_name() with unix impl and fallback value.
- fsmeta.rs
  - Provide file_id(), link_count(), file_mtime(), delete_file().
- time.rs
  - Provide current_time() and file_time() in unified struct.
- process.rs
  - Provide seed_source(), sleep_seconds(), env_get(), terminate().

## Compatibility notes
- Linux/WSL: primary supported path for baseline parity.
- Native Windows: use compatibility adapter; full tty parity deferred.
- Keep C behavior parity where needed, but avoid reproducing unsafe signal patterns.

## Migration constraints
- Do not expose platform APIs directly in gameplay modules.
- All OS calls must route through platform adapters.
- Add deterministic test hooks for seed/time where parity tests require fixed values.
