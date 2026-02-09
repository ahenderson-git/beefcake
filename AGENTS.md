# Beefcake Agent Rules (Global)

## Mission
Help improve Beefcake safely and predictably:
- Make targeted, non-breaking changes by default
- Preserve behaviour and UX unless explicitly asked to change it
- Ask clarifying questions when requirements or risk are unclear
- Prefer incremental improvements over large rewrites

---

## Prime Directive (Non-Breaking Contract)
1. Default to **non-breaking** changes.
2. Default to **small, targeted** edits.
3. If unsure, **STOP and ask for clarification**.
4. Do not trade correctness/stability for “cleaner” code.
5. Preserve existing behaviour unless the user explicitly requests a change.

---

## Clarification Gate (Mandatory)
The assistant must ask clarifying questions BEFORE changing code when:
- Requirements are ambiguous or incomplete
- More than one reasonable implementation exists with different tradeoffs
- A change may impact UI/UX, styling, performance, or data correctness
- A refactor is implied but not explicitly requested
- The assistant proposes touching >10 files, moving files, renaming APIs, or changing schema

Clarifying questions must be short and actionable (max 3 questions).
If blocked, propose safe defaults and ask approval.

---

## Scope Control (All Work)
- Prefer the **smallest change** that achieves the outcome.
- Avoid “drive-by” edits (formatting, renaming, reorganising) in unrelated files.
- If the task is large, split into **staged steps** that each compile/test.
- Limit each step to **≤10 files changed** unless explicitly approved.

Before edits, announce:
- Intent (1–2 lines)
- Files to be touched (exact list)
- Risks (what might break)
- Checks to run after

---

## Safe Change Strategy (How to Work)
Follow this order:
1. Understand current behaviour (search references, read call sites)
2. Identify the minimal change
3. Implement behind flags/config where possible
4. Add/adjust tests close to the change
5. Run required checks and report results
6. Provide manual smoke steps

Avoid:
- sweeping refactors
- renaming for style preferences
- “improving” architecture unless asked

---

## Compatibility Rules
Do NOT change these without explicit approval:
- Public APIs (function signatures, exported names)
- CLI flags/commands
- Config formats and defaults
- File formats and schemas (CSV/JSON export, metadata receipts, etc.)
- Persistent storage structures (DuckDB/SQLite/Postgres schema)
- User-visible UI workflows

If a breaking change is required:
- explain why
- propose a migration path
- stage it (deprecate → warn → remove)

---

## Testing & Verification (Required)
After each step, the assistant must:
- Run the best available automated checks (or specify exact commands)
- If checks do not exist, STOP and propose adding them first

Minimum expectation per step:
- Build/compile succeeds
- Typecheck/lint succeeds (if applicable)
- Unit tests run (if any exist)

For UI-related changes:
- Run UI smoke tests (Playwright/Cypress) if present
- Otherwise provide a clear manual smoke checklist

---

## Logging & Observability Rules
When adding logging:
- Do not log secrets or raw sensitive data
- Prefer structured logs with stable keys
- Log at appropriate levels (debug/info/warn/error)
- Avoid noisy logs in hot paths unless gated behind debug flags

---

## Performance & Data Correctness Rules
- Never change numerical/statistical behaviour without calling it out
- If changing performance-critical code, include:
   - baseline reasoning (what is slow and why)
   - expected improvement
   - any risk to correctness
- Prefer correctness over micro-optimisations

---

## Output Format (Always)
Every response that proposes or performs changes must include:
1. Summary of intent
2. Files changed (grouped by reason)
3. Risks + mitigations
4. Checks run + results (or commands to run)
5. Manual smoke steps (if relevant)
6. Follow-ups / next safe step

---

## Forbidden Actions (Unless Explicitly Requested)
- Large-scale rewrites
- Moving folders / renaming modules for aesthetics
- Removing “unused” code without proving it is unused
- Replacing technologies/frameworks
- Changing UI appearance/behaviour without permission

---

## When In Doubt Protocol
If uncertain:
- STOP
- Ask up to 3 clarification questions
- Provide 1–2 safe options with tradeoffs
- Recommend the lowest-risk default
