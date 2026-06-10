---
name: claude-review
description:
  "Ask Claude to review Nutorch work. Use when the user requests Claude review
  or during large/risky experiments before acting on conclusions."
---

# Claude Review

Use Claude as an independent reviewer for Nutorch issue designs, code diffs,
experiment results, and architectural decisions.

The review is advisory. The calling agent remains responsible for deciding
whether the reviewer's feedback is correct, acting on valid findings, and
explaining any rejected findings.

Do not rely on Claude session memory alone. Session IDs are useful for grouping
related reviews and may help continuity, but every important review prompt must
include the essential issue, diff, log, and question context explicitly.

This skill shells out to a separate `claude -p` process. Its in-session
counterpart is the `adversarial-review` skill, which spawns a fresh-context
subagent instead.

## When This Skill Applies

Use this skill when:

- the user asks for "claude review", "ask Claude", or similar;
- an experiment is large, risky, or touches the tensor registry, autograd,
  device placement, data conversion, or the plugin protocol boundary;
- an experiment has already failed once and the next design needs review;
- a code change affects the dual-input contract, PyTorch API fidelity, or
  persistent state;
- before closing an issue after a complex series of experiments.

## Review Timeout

Every Claude review has a hard timeout of 15 minutes.

Use the helper script in this skill instead of calling `claude` directly. The
script enforces the timeout with Python's subprocess API and terminates Claude's
process group on timeout because macOS does not provide a portable `timeout`
command by default.

## Files and Logs

All review state and output lives under:

```text
logs/claude-review/
```

The helper script maintains:

| Path                                         | Purpose                         |
| -------------------------------------------- | ------------------------------- |
| `logs/claude-review/current-session-id`      | reusable Claude session UUID    |
| `logs/claude-review/<timestamp>-prompt.md`   | exact prompt sent to Claude     |
| `logs/claude-review/<timestamp>-stdout.json` | raw Claude JSON output          |
| `logs/claude-review/<timestamp>-stderr.log`  | Claude stderr / timeout details |

These logs are intentionally under `logs/`, which is gitignored. They may still
contain sensitive diffs or local paths. Keep context narrow and do not write
secrets into review prompts.

## Process

1. **Prepare the review context.** Include only the context Claude needs. Prefer
   concrete artifacts:
   - the user's current request;
   - the issue file and experiment file;
   - relevant `git diff` or `git diff --staged`;
   - test commands and observed outputs;
   - screenshots/log paths when those are part of the evidence.

2. **Ask specific questions.** The prompt should tell Claude what kind of review
   to perform. Examples:
   - "Audit this experiment design for contradictions, missing verification, and
     architectural risks."
   - "Review this diff for correctness, regressions, and missing tests."
   - "Given these logs, identify what failed and what experiment should come
     next."

3. **Run the helper script.**

   ```bash
   python3 skills/claude-review/scripts/claude_review.py \
     --context issues/0001-nutorchd-architecture/README.md \
     "Audit Experiment 2. Focus on whether the scope is right and whether the verification would prove the result."
   ```

   For a staged diff review:

   ```bash
   git diff --staged | python3 skills/claude-review/scripts/claude_review.py \
     "Review this staged diff for bugs, regressions, and missing tests."
   ```

   For a fresh thread:

   ```bash
   python3 skills/claude-review/scripts/claude_review.py --new-session \
     "Start a new review thread. Reply with the review session status."
   ```

4. **Read the result.** The helper prints:
   - the session ID;
   - the prompt path;
   - the raw JSON output path;
   - the extracted Claude text if it can parse it.

5. **Act on valid findings.** If Claude finds a real issue and the user asked
   for implementation, fix it. If Claude is wrong or speculative, do not make
   the change; briefly document why.

6. **Report high-signal results.** Summarize Claude's actionable findings to the
   user. Do not paste long raw output unless the user asks for it.

## Session Handling

The helper script uses `claude -p --session-id <uuid>` for a new review thread
and `claude -p --resume <uuid>` for follow-up turns.

**Default policy: maintain one continuous review session.** The helper resumes
the stored session id on every run. Keep doing that — review after review goes
into the same thread.

- **Do NOT pass `--new-session` on your own judgment.** Not for a "different
  topic," not for a new experiment, not for a new issue. The user decides when
  to start a new thread. Only pass `--new-session` when the user explicitly asks
  for a fresh/new review session.
- If `logs/claude-review/current-session-id` exists, the script resumes it. If
  no session exists, the script creates a UUID and stores it.
- **Automatic self-heal.** If resuming the stored session fails because the id
  stopped working (expired/unknown), the helper retries once with a fresh UUID,
  adopts it, and prints a `note: stored session could not be resumed...` line.
  You do not need to do anything — continuity is restored automatically going
  forward.
- `--new-session` (explicit user request only) creates and stores a new UUID.
- Always include the essential context again on every review. Model-visible
  history should not be treated as guaranteed.

## Claude Command Shape

The helper invokes Claude like this:

```bash
claude -p \
  --output-format json \
  --session-id <uuid> \
  --add-dir /Users/ryan/dev/nutorch \
  --permission-mode dontAsk \
  --tools "Read,Grep,Glob" \
  "<prompt>"
```

For follow-up turns, the helper replaces `--session-id <uuid>` with
`--resume <uuid>`.

The default tool set is review-oriented. Claude may inspect files using
Read/Grep/Glob. Bash is disabled by default because Bash is not read-only when
paired with `--permission-mode dontAsk`.

If a review truly needs Bash, opt in explicitly and include the reason in the
user-facing summary:

```bash
python3 skills/claude-review/scripts/claude_review.py --allow-bash ...
```

If a review should use only supplied context, run with:

```bash
python3 skills/claude-review/scripts/claude_review.py --no-tools ...
```

## Prompt Template

Use this structure for substantial reviews:

```text
You are reviewing Nutorch work. Take a code-review stance: findings first,
ordered by severity, with file/line references where possible.

Task:
<what Claude should review>

Context:
<issue, experiment, diff, logs, test output>

Questions:
1. Is the design/implementation correct?
2. What are the concrete risks or missed cases?
3. What should be changed before implementation/commit?
4. If nothing should change, say that clearly.
```

## Automatic Use in Large Experiments

For large experiments, run Claude review at one or both checkpoints:

1. **After designing the experiment, before implementation.** Ask Claude to
   audit the design for contradictions, missing verification, and scope creep.

2. **After implementation, before final result or commit.** Ask Claude to review
   the diff, logs, and result language.

Do not block forever waiting for Claude. The 15-minute timeout is the maximum.
If Claude times out, record that and continue with normal engineering judgment.

## Smoke Test

After creating or changing this skill, test the helper without giving Claude
repo-changing authority:

```bash
python3 skills/claude-review/scripts/claude_review.py --new-session --no-tools \
  "Smoke test: reply with OK and one sentence describing what session continuity means."
```

Then run a second call without `--new-session`:

```bash
python3 skills/claude-review/scripts/claude_review.py --no-tools \
  "In one sentence, say whether this is the same review session."
```

Both outputs should be saved under `logs/claude-review/`. The printed
`session_id` should match between the two runs. The second answer does not need
to prove memory of the first prompt; the important smoke-test properties are
exit code 0, saved artifacts, and stable session ID reuse.
