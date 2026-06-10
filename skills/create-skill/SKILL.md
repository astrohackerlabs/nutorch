---
name: create-skill
description: "Create a new agent skill. Use when creating a new skill for the project."
---

# Create Skill

Create a new agent skill for Nutorch. Every new skill must follow the Agent
Skills specification and match the conventions established by existing skills.

## When This Skill Applies

Any time a new skill needs to be created.

## Process

1. **Read the specification.** Fetch `https://agentskills.io/specification` to
   review the current Agent Skills format.

2. **Read every existing skill.** Read all `SKILL.md` files in
   `skills/*/SKILL.md` to understand the project's conventions, tone, and level
   of detail.

3. **Create the skill directory and file.** The skill is a directory in
   `skills/<name>/` (symlinked as `.claude/skills/`) containing a `SKILL.md`
   file. The directory name must match the `name` field in the frontmatter.

4. **Follow the spec.** The `SKILL.md` must have:
   - YAML frontmatter with `name` (lowercase, hyphens, max 64 chars) and
     `description` (what it does and when to use it)
   - Markdown body with instructions

5. **Match existing conventions.** Based on the current skills:
   - Keep it to a single `SKILL.md` (no `scripts/`, `references/`, `assets/`
     unless truly needed)
   - Include a "When This Skill Applies" section
   - Include a "Process" or "Steps" section with numbered steps
   - Reference specific paths in the Nutorch repo where relevant
   - Keep under 500 lines

## Naming Rules

From the spec:

- Lowercase letters, numbers, and hyphens only
- Must not start or end with a hyphen
- No consecutive hyphens
- Max 64 characters
- Directory name must match the `name` field
