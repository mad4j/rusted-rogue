# BL-001 - Build C baseline and ncurses prerequisites

## Scope
Document reproducible build and run instructions for the original Rogue C project, including Makefile flags and environment assumptions.

## Source of truth
- original/rogue-libc5-ncurses/rogue/Makefile
- original/rogue-libc5-ncurses/rogue/main.c
- original/rogue-libc5-ncurses/rogue/machdep.c

## Build target
- Binary name: rogue
- Build system: make
- Objects: 23 object files listed in ROGUE_OBJS

## Compiler and flags
From Makefile:
- CC: cc -g
- CFLAGS: -c -DUNIX -DUNIX_SYS5
- Link step: cc <objects> -o rogue -lncurses

Notes:
- -DUNIX and -DUNIX_SYS5 enable UNIX/System V conditional paths in machdep.c.
- Optional define in Makefile comments: -DCURSES (for self-contained curses emulation path).

## Runtime assumptions (from code)
- POSIX APIs expected (_POSIX_SOURCE, unistd.h, termios.h, pwd.h).
- UNIX process/user behavior used (geteuid, getuid, setuid in main.c).
- Terminal behavior and signal handling are UNIX oriented (machdep.c).

## Reproducible build commands
Run from project root:

```bash
cd original/rogue-libc5-ncurses/rogue
make
```

Expected result:
- Executable generated at original/rogue-libc5-ncurses/rogue/rogue

## Run command

```bash
cd original/rogue-libc5-ncurses/rogue
./rogue
```

## Prerequisites by environment

### Linux (recommended)
- build-essential (or equivalent: cc, make)
- ncurses development package

Typical setup examples:
- Debian/Ubuntu: sudo apt-get install build-essential libncurses5-dev
- Fedora/RHEL: sudo dnf install gcc make ncurses-devel

### macOS
- Xcode Command Line Tools (cc, make)
- ncurses available via system/Homebrew if needed

### Windows
- Native PowerShell build is not the primary target.
- Recommended: WSL with Linux toolchain and ncurses dev package.

## Validation checklist
- make completes without linker errors for ncurses.
- rogue binary exists in original/rogue-libc5-ncurses/rogue/.
- launching ./rogue opens terminal UI and accepts input.

## Known caveats
- Legacy setuid/user handling in main.c may behave differently on modern systems.
- Some machdep.c branches are platform conditional; keep -DUNIX -DUNIX_SYS5 unless intentionally migrating behavior.
