# Automated Backlog Runner

This folder contains the assets to execute the Rogue C -> Rust plan step by step.

## Files
- backlog.yaml: machine-readable backlog with dependencies and status.
- runner.prompt.md: prompt for cloud/backlog execution mode.
- execution-log.md: append-only task execution journal.

## How to run
1. Start a delegate/cloud agent session.
2. Provide the content of runner.prompt.md as execution prompt.
3. Ensure the agent updates backlog.yaml and execution-log.md after every task.
4. Repeat until all tasks are DONE or a task is BLOCKED.

## Status model
- TODO: ready but not started.
- DOING: currently in progress.
- DONE: finished with mandatory gates passed.
- BLOCKED: cannot proceed after max fix attempts.

## Mandatory gates per task
- format
- lint
- tests
- parity_if_applicable
