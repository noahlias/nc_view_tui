# AGENTS

## Hard Rule: No Change-Note Comments In Code

- Agents MUST NOT add comments that describe the change they just made (e.g., "removed", "legacy", "cleanup", "hotfix", "flag removed", "temporary workaround").
- Only add comments for genuinely non-obvious, persistent logic or external invariants. Keep such comments short (max 2 lines).

Forbidden examples:
- // shouldShowDoneButton removed; UI reacts to selection
- // legacy code kept for now
- // temporary cleanup / hotfix

Allowed examples (non-obvious logic):
- // Bound must be >= 30px to render handles reliably
- // Server returns seconds (not ms); convert before diffing

Rationale placement:
- Put change reasoning in your plan/final message or PR description — not in code.

## Critical Workflow Requirement

- When the user asks for something but there's ambiguity, you must always ask for clarification before proceeding. Provide users some options.
- When giving user responses, give short and concise answers. Avoid unnecessary verbosity.
- Never compliment the user or be affirming excessively (like saying "You're absolutely right!" etc). Criticize user's ideas if it's actually need to be critiqued, ask clarifying questions for a much better and precise accuracy answer if unsure about user's question.
- Avoid getting stuck. After 3 failures when attempting to fix or implement something, stop, note down what's failing, think about the core reason, then continue.
- When migrating or refactoring code, do not leave legacy code. Remove all deprecated or unused code.

## Code Change Guidelines

- No useless comments or code. Only comment truly complex logic.
- No need to write a comment like "removed foo" after removing code.
- Keep diffs minimal and scoped; do not add files/utilities unless required.
- Prefer existing mechanisms.
- Remove dead code, unused imports, debug prints, and extra empty lines.
- Do not leave temporary scaffolding; revert anything not needed.

## C++ Rules

- Do not using lambda functions.
- Use `nullptr` instead of `NULL` or `0` for pointer types.

## Development

- Workspace root: this repo.
- Crates:
  - `crates/cnc-geom`: math + projection
  - `crates/cnc-gcode`: G-code parser + toolpath model
  - `crates/cnc-tui`: ratatui UI (binary)

### Build

- `cargo build -p cnc-view-tui`

### Run

- `cargo run -p cnc-view-tui -- <path-to-gcode>`

### Tests

- `cargo test -p cnc-gcode`

### Config

- Config file lookup:
  - `./cnc_view_tui.toml`
  - `~/.config/cnc_view_tui/config.toml`
