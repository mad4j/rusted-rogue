# TODO List

[] End game statistics (Ctrl+S in original) | `show_statistics()` not implemented |
[] Blocking messages (e.g. "The orc hits you") | Original: Message blocks input until acknowledged. Rust port: Messages are non-blocking overlays; player can continue acting while messages are visible. |
[] Wizard Ctrl+C (random item) | `new_object_for_wizard()` equivalent not implemented |
[] Auto-pickup gold | Original: Player automatically picks up gold when stepping on it. Rust port: Auto-pickup not implemented; player must explicitly pick up gold with the `g` command. |
[] Ring diagnostics (Ctrl+R in original) | `ring.c` `ring_stats()` diagnostic not implemented |
[] Score-only flag in saved game | `score_only` is not persisted to disk (resets on load) |
[] Wizard flag persistence | `wizard` flag is not persisted; reload always starts non-wizard |