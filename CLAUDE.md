# CLAUDE.md

## Agent Structure

Use subagents for all work while making changes no matter how trivial.  The main conversation (Opus) coordinates and synthesizes. Subagents run in parallel when tasks are independent.

| Agent | Model | When to Use |
|-------|-------|-------------|
| **Main** | Sonnet (user switches to Opus manually) | Coordination, synthesis, user communication |
| **Research** | Sonnet | When investigating unknowns, searching docs, exploring APIs |
| **Coding** | Sonnet | Always when writing or modifying code |
| **Testing** | Sonnet | Always when coding — validate syntax, check builds, verify changes |
| **Critic** | Sonnet | Always when coding — audit changes for bugs, regressions, edge cases |
| **Debug** | Sonnet | When diagnosing crashes, unexpected behavior, or pipeline failures |

### Rules
- **Always** use coding + testing + critic agents together when making ANY changes, no matter how trivial
- **Never** skip the critic — it catches regressions before they ship
- Launch agents in **parallel** when their tasks are independent
- The debug agent should trace through actual code paths, not guess

---

**Guardrail‑First Instructions for Working in This Repository**

---

## ❗ PRIMARY DIRECTIVE (READ FIRST)

This repository is protected by **explicit guardrails**.
**Guardrails override assumptions, aesthetics, convenience, and inferred intent.**

If code behavior and guardrail documents differ:
> **The guardrails are authoritative until explicitly revoked.**

If you are unsure whether a change is allowed:
> **STOP and ask. Do not guess.**

---

## 📂 GUARDRAILS LOCATION (AUTHORITATIVE)
Guardrail documents live under the repository folder:

- `docs/guardrails/` — process + invariants (always-on rules)
- `docs/architecture/` — camera/rendering/parallax/layer contracts
- `docs/design/` — level/gameplay design constraints
- `docs/agents/` — agent workflow + enforcement contracts

If a referenced guardrail is not found under `docs/`, **STOP** and ask for the correct path.

---
### Agent Workflow
Each phase uses 4 agents: Spec → Implementation → Test → Critic.

## 🛑 STOP & TEST ENFORCEMENT (NON‑NEGOTIABLE)

**No multi‑step execution is permitted without validation.**

### Mandatory Workflow
1. **STOP** after each phase
2. **TEST / VALIDATE**
3. **WAIT for human confirmation**
4. Proceed only after approval

Forbidden behaviors:
- Proceeding “to save time”
- Chaining fixes across systems
- Making “minor” follow‑ups without confirmation

If STOP & TEST is skipped:
> **All output is invalid regardless of correctness.**

---

## ✅ ALLOWED COMMANDS (REFERENCE ONLY)

- **Build:** `cargo build`
- **Run:** `cargo run`
- **Test:** `cargo test`
- **Single test:** `cargo test <test_name>`
- **Check (fast):** `cargo check`
- **Format:** `cargo fmt`
- **Lint:** `cargo clippy -- -D warnings`

Commands are informational only.
Running them is **not** an instruction to modify code.

---

## 🎮 PROJECT OVERVIEW (DESCRIPTIVE, NOT A CONTRACT)

**Jasper’s World (test2)**
A 2D raccoon platformer rendered with **Camera3d** in **Bevy 0.18** (Rust 2024).

This section describes the **current state**, not eternal truth.

---

## 🧱 CURRENT ARCHITECTURE (SUBJECT TO CHANGE)

> ⚠️ The following are **current architectural choices**, not invariants.

### Rendering Model
- World rendered using **Camera3d + OrthographicProjection**
- Camera looks down the **-Z axis** onto the **XY plane**
- Z axis is used for **visual depth only**
- Sprites rendered as **Mesh3d quads** with `StandardMaterial`
- Real 3D lighting is used (`DirectionalLight3d`, `PointLight3d`)

🔒 **Guardrail override:**
Camera usage, ordering, clearing, and anchoring are governed by formal camera guardrails.
Do **not** assume this architecture cannot evolve.

---

## 📷 CAMERA SAFETY RULES (CRITICAL)

- All cameras **must have explicit role markers**
  - `GameplayCamera`
  - `TitleCamera`
  - `UICamera`
  - etc.
- **Camera3d‑only queries are forbidden**
- Do NOT assume there is “only one camera” unless guarded by role

Violations here are considered **hard bugs**, not stylistic issues.

---

## 🕹️ PHYSICS & MOVEMENT STACK (LOCKED)

- Physics uses **avian2d**
- Character control uses the current controller stack (do not replace)

🚫 **Do NOT:**
- Swap physics engines
- Replace controllers
- “Simplify” physics to fix visuals

If a behavior looks wrong:
> Assume integration or configuration error first — not engine choice.

---

## 🎨 ASSET USAGE & SAFETY

### Asset Sources
- `buildassets/` — **STAGING ONLY**
  - Large CC0 packs
  - Not safe for runtime use
- `assets/` — **RUNTIME‑AUTHORITATIVE**

🚫 **Forbidden:**
- Referencing assets directly from `buildassets`
- Introducing new assets without the pruning/safety pass
- Assuming all Kenney assets are acceptable for gameplay scenes

Asset role honesty and gameplay envelope rules apply.

---

## 🧠 ASSUMPTIONS VS INVARIANTS (IMPORTANT)

Some statements exist as **current tuning assumptions**, not guarantees.

Example:
- Platform spacing (e.g., “~5 tiles apart”) reflects **current movement tuning**
- If movement physics change, geometry must be revalidated

🚫 Treating assumptions as eternal invariants is a defect.

Only formally documented guardrails are permanent.

---

## 🤖 AGENT ROLE EXPECTATIONS

Agents may assist with:
- Analysis
- Diagnosis
- Narrow, scoped changes

Agents must **not**:
- Expand scope
- Improve aesthetics unrequested
- Combine multiple fixes
- Change systems outside the declared intent

If a guardrail blocks a requested change:
> You must explain the conflict and stop.

---

## 🧾 DOCUMENTATION REQUIREMENT

Any non‑obvious number, offset, or workaround **must be documented in code comments**.

Rules:
- Explain **WHY** the value exists
- Explain **WHAT breaks if it changes**
- Move comments with code when refactoring

Undocumented “magic values” are considered future bugs.

---

## 📚 REQUIRED READING (AUTHORITATIVE)

Before implementing changes, agents must respect:
- Quit / shutdown lifecycle guardrail
- Camera role & pipeline guardrails
- Camera ↔ world anchor rules
- Background lifecycle & bounds rules
- Implementation (one‑axis) rules
- Regression snapshot discipline

If unsure which guardrails apply:
> Ask before proceeding.

---

## ✅ FINAL RULE

If a change:
- Fixes the requested problem **but**
- Violates a guardrail
- Introduces new unrequested behavior
- Touches multiple systems

Then:
> **The change is invalid. Roll it back.**

---

**Guardrails exist to protect correctness and time.
Creativity happens first — constraints lock it safely.**
