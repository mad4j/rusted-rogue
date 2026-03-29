Execution mode: backlog runner

Goal:
Execute the Rust porting backlog one task at a time with strict dependency checks and mandatory gates.

Inputs:
- Backlog file: automation/backlog.yaml
- Source C code: original/rogue-libc5-ncurses/rogue/

Rules:
1. Pick the first TODO task whose deps are all DONE.
2. Implement only that task. No scope creep.
3. Run gates after implementation:
   - format
   - lint
   - tests
   - parity_if_applicable
4. If all gates pass:
   - set task status to DONE in automation/backlog.yaml
   - write a short execution note in automation/execution-log.md
   - commit with message: <task-id>: <short-title>
5. If any gate fails:
   - attempt up to 3 focused fixes for the same task
   - if still failing, set task to BLOCKED with root cause and failing gate
6. Continue automatically with the next eligible task.
7. Stop only when:
   - all tasks are DONE, or
   - a task is BLOCKED and cannot be resolved autonomously.

Required output per cycle:
- Selected task id and title
- Files changed
- Gate results
- Backlog status delta
- Next selected task

Safety constraints:
- Do not modify unrelated files.
- Do not skip dependency order.
- Do not mark DONE without gate evidence.
- Preserve behavior parity with C baseline where applicable.
