// Three-state theme matrix gate (issue 0013 exp 4): drives a served build
// through system/light/dark over CDP, emulating the OS color scheme to
// prove system mode follows it LIVE while pinned modes ignore it — the
// assertion screenshots cannot make.
// Requires the preview server: `bun run preview --port 4399`.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";

declare const Bun: any;

const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const PORT = 9224;
const URL_UNDER_TEST = "http://localhost:4399/";

const proc = Bun.spawn(
  [CHROME, "--headless", "--disable-gpu",
    `--remote-debugging-port=${PORT}`,
    // Isolated profile: without it, launch can delegate to a running
    // Chrome and exit, leaving nothing listening on the port.
    `--user-data-dir=${mkdtempSync(`${tmpdir()}/nutorch-theme-`)}`,
    "about:blank"],
  { stdout: "ignore", stderr: "ignore" },
);
// Poll for the DevTools endpoint (startup time varies; a fixed sleep races).
async function waitForDevtools(): Promise<void> {
  for (let i = 0; i < 30; i++) {
    try {
      await fetch(`http://localhost:${PORT}/json/version`);
      return;
    } catch {
      await new Promise((r) => setTimeout(r, 500));
    }
  }
  throw new Error("DevTools endpoint never came up");
}
await waitForDevtools();

let failed = false;
function check(name: string, ok: boolean, detail = "") {
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? ` (${detail})` : ""}`);
  if (!ok) failed = true;
}

try {
  const list = (await (
    await fetch(`http://localhost:${PORT}/json/list`)
  ).json()) as { webSocketDebuggerUrl: string; type: string }[];
  const page = list.find((t) => t.type === "page");
  if (!page) throw new Error("no page target");
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  let id = 0;
  const pending = new Map<number, (v: any) => void>();
  ws.addEventListener("message", (event) => {
    const msg = JSON.parse(String(event.data));
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)!(msg.result);
      pending.delete(msg.id);
    }
  });
  await new Promise((r) => ws.addEventListener("open", r, { once: true }));
  const send = (method: string, params: object = {}) =>
    new Promise<any>((resolve) => {
      pending.set(++id, resolve);
      ws.send(JSON.stringify({ id, method, params }));
    });
  const evaluate = async (expression: string) =>
    (await send("Runtime.evaluate", { expression, returnByValue: true }))
      ?.result?.value;
  const emulate = (scheme: "light" | "dark") =>
    send("Emulation.setEmulatedMedia", {
      features: [{ name: "prefers-color-scheme", value: scheme }],
    });
  const navigate = async (url: string) => {
    await send("Page.navigate", { url });
    await new Promise((r) => setTimeout(r, 800));
  };
  const state = () =>
    evaluate(`({
      setting: document.documentElement.dataset.themeSetting ?? null,
      theme: document.documentElement.dataset.theme ?? null,
      stored: (() => { try { return localStorage.getItem("theme"); } catch { return "ERR"; } })(),
    })`);
  const click = () =>
    evaluate(`document.getElementById("theme-toggle").click(), true`);

  await send("Page.enable");

  // Fresh visit under emulated LIGHT OS, clean storage.
  await emulate("light");
  await navigate(URL_UNDER_TEST);
  await evaluate("localStorage.clear()");
  await navigate(URL_UNDER_TEST);
  let s = await state();
  check("fresh visit defaults to system", s.setting === "system" && s.stored === null, JSON.stringify(s));
  check("system resolves to emulated light", s.theme === "light");

  // OS flip while in system mode: live, no reload.
  await emulate("dark");
  await new Promise((r) => setTimeout(r, 300));
  s = await state();
  check("system follows OS flip live (no reload)", s.theme === "dark", JSON.stringify(s));

  // Click 1 → light (pinned).
  await click();
  s = await state();
  check("click 1 pins light", s.setting === "light" && s.theme === "light" && s.stored === "light", JSON.stringify(s));
  await emulate("light");
  await emulate("dark");
  await new Promise((r) => setTimeout(r, 300));
  s = await state();
  check("pinned light ignores OS flips", s.theme === "light");
  await navigate(URL_UNDER_TEST);
  s = await state();
  check("light persists across reload (OS dark)", s.setting === "light" && s.theme === "light", JSON.stringify(s));

  // Click 2 → dark (pinned).
  await click();
  s = await state();
  check("click 2 pins dark", s.setting === "dark" && s.theme === "dark" && s.stored === "dark", JSON.stringify(s));
  await emulate("light");
  await new Promise((r) => setTimeout(r, 300));
  s = await state();
  check("pinned dark ignores OS flips", s.theme === "dark");
  await navigate(URL_UNDER_TEST);
  s = await state();
  check("dark persists across reload (OS light)", s.setting === "dark" && s.theme === "dark", JSON.stringify(s));

  // Click 3 → back to system: tracks the emulated OS again, live.
  await click();
  s = await state();
  check("click 3 returns to system (stored explicitly)", s.setting === "system" && s.stored === "system", JSON.stringify(s));
  check("system re-resolves to emulated light", s.theme === "light");
  await emulate("dark");
  await new Promise((r) => setTimeout(r, 300));
  s = await state();
  check("system tracks OS again, live", s.theme === "dark");

  // Invariant: data-theme only ever holds a resolved mode.
  check("data-theme holds a resolved mode", s.theme === "light" || s.theme === "dark");

  // Accessibility: the button names its state.
  const aria = await evaluate(
    `document.getElementById("theme-toggle").getAttribute("aria-label")`,
  );
  check("aria-label names current setting", typeof aria === "string" && aria.includes("system"), String(aria));

  ws.close();
} finally {
  proc.kill();
}

if (failed) process.exit(1);
console.log("theme matrix passed");
