# Agent Structure Guardrail

## Status: ACTIVE (non-negotiable)

## Rule

All work must use subagents. The main conversation (Opus) coordinates and synthesizes. Subagents run in parallel when tasks are independent.

| Agent | Model | When to Use |
|-------|-------|-------------|
| **Main** | Opus | Coordination, synthesis, user communication |
| **Research** | Sonnet | When investigating unknowns, searching docs, exploring APIs |
| **Coding** | Sonnet | Always when writing or modifying code |
| **Testing** | Sonnet | Always when coding — validate syntax, check builds, verify changes |
| **Critic** | Opus | Always when coding — audit changes for bugs, regressions, edge cases |
| **Debug** | Sonnet | When diagnosing crashes, unexpected behavior, or pipeline failures |

## Mandatory Enforcement

- **Always** use coding + testing + critic agents together when making ANY changes, no matter how trivial
- **Never** skip the critic — it catches regressions before they ship
- Launch agents in **parallel** when their tasks are independent
- The debug agent should trace through actual code paths, not guess

## Violation

If any code change is made without all three agents (coding, testing, critic), the change is invalid regardless of correctness.
