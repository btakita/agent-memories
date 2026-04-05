# Agent Memories

A convention for externalizing learned lessons from AI agent sessions into committed, version-controlled memory files.

## The Problem

AI coding tools accumulate knowledge during sessions — corrections, confirmations, discoveries, user preferences. This knowledge currently lives in:

1. **Machine-local storage** (e.g., `~/.claude/projects/*/memory/`) — not portable, not version-controlled, not shareable
2. **Instruction files** (CLAUDE.md) — mixed with prescribed rules, no structured metadata, hard to manage at scale

Neither approach is ideal. Machine-local memories are lost when switching machines or sharing projects. Instruction files become bloated when every lesson is inlined.

## The Convention

Place memories in a `memories/` directory under your agent config directory:

```
your-project/
├── .agent/memories/               # or .agents/memories/
│   ├── feedback_use_agent_doc_init.md
│   ├── project_auth_rewrite.md
│   ├── user_role.md
│   └── reference_linear_project.md
├── .agent/memories/MEMORY.md      # optional index
├── CLAUDE.md                      # (or AGENTS.md, .cursorrules, etc.)
└── ...
```

Each memory is a standalone markdown file with YAML frontmatter:

```markdown
---
name: use-agent-doc-init
description: Use agent-doc init for new session documents, not manual file creation
type: feedback
scope: agent-doc
---

Use `agent-doc init` for new session documents, not manual file creation with Write tool.

**Why:** Manual creation bypasses the agent-doc pipeline (init -> snapshot -> git baseline -> commit).

**How to apply:** When creating a new task/session document, run `agent-doc init <path>`.
```

## Memory Types

| Type | Purpose | Origin |
|------|---------|--------|
| `user` | Who the user is, their role, preferences, expertise | Observed from interaction |
| `feedback` | Corrections and confirmations of approach | User says "don't do X" or "yes, exactly" |
| `project` | Ongoing work, goals, deadlines, decisions | Learned during sessions |
| `reference` | Pointers to external systems and resources | User mentions external tools/systems |

## Memories vs Rules

AI coding tools use "rules" (CLAUDE.md, `.cursorrules`, `.windsurfrules`) for **prescribed policy** — conventions and guidelines written by the developer upfront.

Memories are different: they contain **learned lessons** — knowledge captured from experience during agent sessions. The distinction matters because it tells the agent the provenance:

- **Rules** = "use snake_case for Python functions" (prescribed, developer-authored)
- **Memories** = "last time we tried mocking the database, it masked a broken migration" (learned, experience-derived)

But they converge: a mature memory ("always use agent-doc init") is functionally a rule. The distinction is origin (prescribed vs. learned), not final form. Memories can be the source that generates rule entries, or coexist as the detailed version alongside terse rule summaries.

## Design Principles

1. **One lesson per file.** Each memory captures exactly one piece of knowledge.
2. **Structured metadata.** YAML frontmatter enables filtering, searching, and validation.
3. **Why and How.** Every feedback/project memory includes the reason and application guidance.
4. **Committed and portable.** Memories are version-controlled project files, not ephemeral state.
5. **Index for scanning.** An optional `MEMORY.md` provides a one-line-per-memory quick reference.

## Cross-Harness Compatibility

This convention works with any AI coding assistant that can read project files:

| Tool | How It Loads Memories |
|------|----------------------|
| Claude Code | Auto memory reads `MEMORY.md` + individual files on demand |
| Cursor | Agent Requested mode or file reference from `.cursorrules` |
| GitHub Copilot | Scoped instructions with `applyTo` globs |
| Windsurf | Agent reads referenced files |
| Any tool | Memories are plain markdown — any tool that reads files can use them |

## Spec

See [SPEC.md](SPEC.md) for the full format specification, including validation rules, field definitions, and body structure conventions.

## Related Work

- [Agent Runbooks](https://github.com/btakita/agent-runbooks) — Convention for externalizing procedures into on-demand runbook files
- [AGENTS.md](https://agents.md/) — Universal instruction file spec (Linux Foundation)
- [Claude Code Auto Memory](https://docs.anthropic.com/en/docs/claude-code) — Machine-local memory system that inspired this convention

## Install

### Quick install (prebuilt binary)

```bash
curl -fsSL https://raw.githubusercontent.com/btakita/agent-memories/main/install.sh | sh
```

### Cargo

```bash
cargo install agent-memories
```

### From source

```bash
git clone https://github.com/btakita/agent-memories.git
cd agent-memories
cargo install --path .
```

## License

[CC0 1.0](LICENSE) — Public domain. Use this convention however you like.
