# CLAUDE.md

## Agent Structure

See `docs/agents/` for full workflow contract. Always use subagents for any change — coding + testing + critic in parallel.

---

## ❗ PRIMARY DIRECTIVE

Guardrails override assumptions, aesthetics, convenience, and inferred intent.

If code behavior and guardrail documents differ: **guardrails are authoritative until explicitly revoked.**

If unsure whether a change is allowed: **STOP and ask. Do not guess.**

---

## 📂 GUARDRAILS LOCATION

- `docs/guardrails/` — process + invariants
- `docs/architecture/` — camera/rendering/parallax/layer contracts
- `docs/design/` — level/gameplay design constraints
- `docs/agents/` — agent workflow + enforcement contracts

If a referenced guardrail is not found under `docs/`, **STOP** and ask.

---

## 🛑 STOP & TEST (NON-NEGOTIABLE)

1. **STOP** after each phase
2. **TEST / VALIDATE**
3. **WAIT for human confirmation**
4. Proceed only after approval

Forbidden: proceeding to save time, chaining fixes, making "minor" follow-ups without confirmation.

> If STOP & TEST is skipped, all output is invalid regardless of correctness.

---

## 📷 CAMERA SAFETY RULES

- All cameras must have explicit role markers (`GameplayCamera`, `TitleCamera`, `UICamera`, etc.)
- Camera3d-only queries are forbidden
- Do NOT assume there is "only one camera" unless guarded by role

Violations are hard bugs, not style issues.

---

## 🕹️ PHYSICS & MOVEMENT (LOCKED)

- Physics: **avian2d** — do not swap
- Do not replace the character controller stack
- If behavior looks wrong: assume integration/configuration error, not engine choice

---

## 🎨 ASSET SAFETY

- `buildassets/` — staging only, not safe for runtime
- `assets/` — runtime authoritative
- Never reference `buildassets` directly in code
- No new assets without a pruning/safety pass

---

## 🤖 AGENT SCOPE

Agents must not expand scope, improve aesthetics unrequested, combine multiple fixes, or touch systems outside declared intent. If a guardrail blocks a change, explain the conflict and stop.

---

## Cargo Commands

`cargo build` / `run` / `test` / `check` / `fmt` / `clippy -- -D warnings`

Running these is not permission to modify code.

---

## ✅ FINAL RULE

If a change fixes the requested problem but violates a guardrail, introduces unrequested behavior, or touches multiple systems: **the change is invalid. Roll it back.**

---

## 📚 REQUIRED READING

Before implementing: quit/shutdown lifecycle guardrail, camera role & pipeline guardrails, camera ↔ world anchor rules, background lifecycle & bounds rules, implementation (one-axis) rules, regression snapshot discipline.
