# agent-memories spec

Committed memory format for AI agent sessions. Memories capture lessons learned during work — corrections, confirmations, discoveries — in a portable, version-controlled format.

## File Location

```
.agent/memories/*.md
```

One file per memory. Filenames should be descriptive slugs (e.g., `feedback_use_agent_doc_init.md`).

## Format

Each memory is a markdown file with YAML frontmatter:

```markdown
---
name: <short identifier>
description: <one-line summary, used for relevance matching>
type: <user | feedback | project | reference>
scope: <optional — limits applicability to a specific area>
---

<memory content>
```

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Short identifier, unique within the project |
| `description` | string | One-line summary used for relevance matching in future sessions |
| `type` | enum | Memory category (see types below) |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `scope` | string | Limits applicability (e.g., `agent-doc`, `testing`, `email`) |

## Memory Types

| Type | Purpose | Example |
|------|---------|---------|
| `user` | Who the user is, their role, preferences, expertise | "Senior Rust developer, new to React frontend" |
| `feedback` | Corrections and confirmations of approach | "Never mock the database in integration tests" |
| `project` | Ongoing work, goals, deadlines, decisions | "Auth rewrite driven by compliance, not tech debt" |
| `reference` | Pointers to external systems and resources | "Pipeline bugs tracked in Linear project INGEST" |

## Body Structure

### feedback type

```markdown
<rule>

**Why:** <the reason — often a past incident or strong preference>

**How to apply:** <when/where this guidance kicks in>
```

### project type

```markdown
<fact or decision>

**Why:** <the motivation — constraint, deadline, stakeholder ask>

**How to apply:** <how this shapes suggestions>
```

### user and reference types

Free-form content. Keep it concise and actionable.

## Index File

An optional `MEMORY.md` index at the project root provides a quick-scan summary:

```markdown
# Project Memory

- [Use agent-doc init](feedback_use_agent_doc_init.md) — always use agent-doc init, not Write tool
- [Auth rewrite](project_auth_rewrite.md) — compliance-driven, not tech debt
```

Each entry is one line, under 150 characters.

## Validation Rules

When instruction-files audits agent-memories:

1. **Frontmatter required**: `name`, `description`, `type` must be present
2. **Valid type**: Must be one of `user`, `feedback`, `project`, `reference`
3. **Non-empty body**: Memory must have content after frontmatter
4. **No machine-local paths**: Same context invariant as other instruction files
5. **Unique names**: No duplicate `name` values within a project

## Relationship to Rules Files

| System | Format | Origin |
|--------|--------|--------|
| Cursor Rules (`.cursorrules`) | Markdown | Prescribed by developer |
| Windsurf Rules (`.windsurfrules`) | Markdown | Prescribed by developer |
| CLAUDE.md | Markdown per-directory | Prescribed by developer |
| **agent-memories** | Markdown with frontmatter | **Learned from experience** |

Memories and rules converge: a mature memory ("always use X") is functionally a rule. The distinction is origin (prescribed vs. learned), not final form. agent-memories can be the source that generates CLAUDE.md entries, or coexist as the detailed version alongside terse CLAUDE.md summaries.
