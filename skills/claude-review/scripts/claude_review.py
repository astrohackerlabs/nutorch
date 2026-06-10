#!/usr/bin/env python3
"""Run a Claude review with a persistent session and a 15 minute timeout."""

from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
import sys
import uuid
from datetime import datetime
from pathlib import Path


TIMEOUT_SECONDS = 15 * 60
SCRIPT_ROOT = Path(__file__).resolve().parents[3]
try:
    ROOT = Path(
        subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=SCRIPT_ROOT,
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    )
except (subprocess.CalledProcessError, FileNotFoundError):
    ROOT = SCRIPT_ROOT
LOG_DIR = ROOT / "logs" / "claude-review"
SESSION_FILE = LOG_DIR / "current-session-id"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run claude -p for review.")
    parser.add_argument("prompt", nargs="*", help="Review prompt text.")
    parser.add_argument(
        "--context",
        action="append",
        default=[],
        help="Context file to append to the prompt. May be repeated.",
    )
    parser.add_argument("--prompt-file", help="Read the main prompt from a file.")
    parser.add_argument(
        "--new-session",
        action="store_true",
        help="Create and store a fresh Claude session UUID.",
    )
    parser.add_argument(
        "--no-tools",
        action="store_true",
        help="Disable Claude tools; use only supplied prompt/context.",
    )
    parser.add_argument(
        "--allow-bash",
        action="store_true",
        help="Allow Claude to use Bash in addition to Read/Grep/Glob.",
    )
    return parser.parse_args()


def ensure_log_dir() -> None:
    LOG_DIR.mkdir(parents=True, exist_ok=True)


def session_id(new_session: bool) -> tuple[str, bool]:
    if new_session or not SESSION_FILE.exists():
        sid = str(uuid.uuid4())
        return sid, True
    sid = SESSION_FILE.read_text().strip()
    try:
        uuid.UUID(sid)
        return sid, False
    except ValueError:
        print(
            f"Ignoring invalid Claude review session id in {SESSION_FILE}: {sid}",
            file=sys.stderr,
        )
        return str(uuid.uuid4()), True


def read_text(path: str) -> str:
    return Path(path).read_text(errors="replace")


def build_prompt(args: argparse.Namespace) -> str:
    pieces: list[str] = []
    if args.prompt_file:
        pieces.append(read_text(args.prompt_file).rstrip())
    if args.prompt:
        pieces.append(" ".join(args.prompt).rstrip())
    if not pieces:
        if sys.stdin.isatty():
            raise SystemExit(
                "No prompt provided. Pass prompt text, --prompt-file, or stdin."
            )
        pieces.append(sys.stdin.read().rstrip())

    for context_path in args.context:
        context = read_text(context_path).rstrip()
        pieces.append(
            f"## Context: {context_path}\n\n"
            "```text\n"
            f"{context}\n"
            "```"
        )

    prompt = "\n\n".join(piece for piece in pieces if piece)
    if not prompt.strip():
        raise SystemExit("Prompt is empty.")
    return prompt


def claude_command(
    sid: str, use_session_id: bool, prompt: str, no_tools: bool, allow_bash: bool
) -> list[str]:
    cmd = [
        "claude",
        "-p",
        "--output-format",
        "json",
        "--add-dir",
        str(ROOT),
        "--permission-mode",
        "dontAsk",
    ]
    if use_session_id:
        cmd.extend(["--session-id", sid])
    else:
        cmd.extend(["--resume", sid])
    if no_tools:
        cmd.append("--tools=")
    elif allow_bash:
        cmd.append("--tools=Read,Grep,Glob,Bash")
    else:
        cmd.append("--tools=Read,Grep,Glob")
    cmd.append(prompt)
    return cmd


def extract_text(stdout: str) -> str:
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError:
        return stdout.strip()

    for key in ("result", "text", "content", "message"):
        value = payload.get(key)
        if isinstance(value, str):
            return value.strip()
        if isinstance(value, list):
            texts = []
            for item in value:
                if isinstance(item, str):
                    texts.append(item)
                elif isinstance(item, dict) and isinstance(item.get("text"), str):
                    texts.append(item["text"])
            if texts:
                return "\n".join(texts).strip()

    return json.dumps(payload, indent=2)


def update_session_from_stdout(stdout: str, fallback: str) -> str:
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError:
        return fallback
    sid = payload.get("session_id")
    if isinstance(sid, str):
        uuid.UUID(sid)
        SESSION_FILE.write_text(sid + "\n")
        return sid
    return fallback


def terminate_process_group(proc: subprocess.Popen[str]) -> None:
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        proc.wait(timeout=5)
        return
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    proc.wait()


def run_subprocess(cmd: list, stdout_path: Path, stderr_path: Path) -> int:
    """Run `cmd`, capturing stdout/stderr to files. Truncates both each call.

    Returns the exit code, or 124 on timeout, 130 on interrupt, 127 on launch
    failure.
    """
    try:
        with stdout_path.open("w") as stdout_file, stderr_path.open("w") as stderr_file:
            proc = subprocess.Popen(
                cmd,
                cwd=ROOT,
                text=True,
                stdout=stdout_file,
                stderr=stderr_file,
                start_new_session=True,
            )
            try:
                return proc.wait(timeout=TIMEOUT_SECONDS)
            except subprocess.TimeoutExpired:
                stderr_file.write(f"\nTimed out after {TIMEOUT_SECONDS} seconds.\n")
                stderr_file.flush()
                terminate_process_group(proc)
                return 124
            except KeyboardInterrupt:
                stderr_file.write("\nInterrupted by user.\n")
                stderr_file.flush()
                terminate_process_group(proc)
                return 130
    except OSError as exc:
        stderr_path.write_text(f"Failed to launch Claude: {exc}\n")
        return 127


def main() -> int:
    args = parse_args()
    ensure_log_dir()
    sid, is_new_session = session_id(args.new_session)
    prompt = build_prompt(args)
    stamp = datetime.now().strftime("%Y%m%d-%H%M%S-%f")
    prompt_path = LOG_DIR / f"{stamp}-prompt.md"
    stdout_path = LOG_DIR / f"{stamp}-stdout.json"
    stderr_path = LOG_DIR / f"{stamp}-stderr.log"
    prompt_path.write_text(prompt)

    cmd = claude_command(sid, is_new_session, prompt, args.no_tools, args.allow_bash)
    return_code = run_subprocess(cmd, stdout_path, stderr_path)

    # Self-heal: if resuming a stored session failed (not a timeout/interrupt),
    # the stored id has most likely expired. Default policy is to maintain
    # continuity, so rather than failing, retry once with a fresh session id.
    fell_back = False
    if not is_new_session and return_code not in (0, 124, 130):
        first_error = stderr_path.read_text(errors="replace").strip()
        sid = str(uuid.uuid4())
        is_new_session = True
        cmd = claude_command(sid, is_new_session, prompt, args.no_tools, args.allow_bash)
        return_code = run_subprocess(cmd, stdout_path, stderr_path)
        fell_back = True

    stdout = stdout_path.read_text(errors="replace")
    if return_code == 0:
        sid = update_session_from_stdout(stdout, sid)
        SESSION_FILE.write_text(sid + "\n")

    if fell_back:
        print(
            "note: stored session could not be resumed; started a fresh session."
        )
        if first_error:
            print(f"resume_error={first_error.splitlines()[-1]}")
    print(f"session_id={sid}")
    print(f"prompt={prompt_path}")
    print(f"stdout={stdout_path}")
    print(f"stderr={stderr_path}")
    print(f"exit_code={return_code}")
    print()
    if return_code == 124:
        print(f"Claude review timed out after {TIMEOUT_SECONDS} seconds.")
    elif return_code == 130:
        print("Claude review was interrupted.")
    elif return_code == 127:
        print(stderr_path.read_text(errors="replace").strip())
    else:
        print(extract_text(stdout))
    return return_code


if __name__ == "__main__":
    raise SystemExit(main())
