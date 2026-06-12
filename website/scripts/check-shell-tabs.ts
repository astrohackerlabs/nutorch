// Shell-tabs matrix gate (issue 0015 exp 1): drives a served build over CDP
// to prove the site-wide shell preference — default bash, same-page
// multi-group flip, cross-page persistence, hero on the shared key, and
// legacy-key migration.
// Requires the preview server: `bun run preview --port 4399`.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";

declare const Bun: any;

const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const PORT = 9226;
const DOCS = "http://localhost:4399/docs/getting-started/";
const HOME = "http://localhost:4399/";

const proc = Bun.spawn(
  [CHROME, "--headless", "--disable-gpu",
    `--remote-debugging-port=${PORT}`,
    `--user-data-dir=${mkdtempSync(`${tmpdir()}/nutorch-shelltabs-`)}`,
    "about:blank"],
  { stdout: "ignore", stderr: "ignore" },
);
for (let i = 0; i < 30; i++) {
  try {
    await fetch(`http://localhost:${PORT}/json/version`);
    break;
  } catch {
    await new Promise((r) => setTimeout(r, 500));
  }
}

let failed = false;
function check(name: string, ok: boolean, detail = "") {
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? ` (${detail})` : ""}`);
  if (!ok) failed = true;
}

try {
  const list = (await (
    await fetch(`http://localhost:${PORT}/json/list`)
  ).json()) as any[];
  const ws = new WebSocket(
    list.find((t) => t.type === "page").webSocketDebuggerUrl,
  );
  let id = 0;
  const pending = new Map<number, (v: any) => void>();
  ws.addEventListener("message", (e) => {
    const m = JSON.parse(String(e.data));
    if (m.id && pending.has(m.id)) {
      pending.get(m.id)!(m.result);
      pending.delete(m.id);
    }
  });
  await new Promise((r) => ws.addEventListener("open", r, { once: true }));
  const send = (method: string, params: object = {}) =>
    new Promise<any>((res) => {
      pending.set(++id, res);
      ws.send(JSON.stringify({ id, method, params }));
    });
  const evl = async (expression: string) =>
    (await send("Runtime.evaluate", { expression, returnByValue: true }))
      ?.result?.value;
  const nav = async (u: string) => {
    await send("Page.navigate", { url: u });
    await new Promise((r) => setTimeout(r, 800));
  };
  const state = () =>
    evl(`({
      groups: [...document.querySelectorAll(".shell-tabs")].map((g) => ({
        nuSelected: g.querySelector('[data-shell-tab="nu"]').getAttribute("aria-selected"),
        posixHidden: g.querySelector('[data-shell-panel="posix"]').hidden,
        nuHidden: g.querySelector('[data-shell-panel="nu"]').hidden,
      })),
      rootShell: document.documentElement.dataset.shell ?? null,
      stored: (() => { try { return localStorage.getItem("shell"); } catch { return "ERR"; } })(),
      legacy: (() => { try { return localStorage.getItem("hero-shell"); } catch { return "ERR"; } })(),
    })`);

  await send("Page.enable");

  // Page sweep: every page's tab-group count must equal the inventory
  // (issue 0015 exp 2 — the executable exemption table). Non-inventoried
  // pages assert zero.
  const EXPECTED: Record<string, number> = {
    "": 1, // the hero
    "docs/": 3, // renders getting-started
    "docs/getting-started/": 3,
    "docs/daemon/": 1,
    "docs/tensors/": 3,
    "docs/ops/": 2,
    "docs/autograd/": 2,
    "docs/neural-networks/": 3,
    "docs/nushell/": 0,
    "docs/install-from-source/": 0,
    // Reference pages: one generated pair per op (issue 0017 exp 3) —
    // the counts are the category sizes from the op table.
    "docs/reference/creation/": 14,
    "docs/reference/pointwise/": 71,
    "docs/reference/comparison/": 25,
    "docs/reference/reduction/": 21,
    "docs/reference/linalg/": 17,
    "docs/reference/shape/": 23,
    "docs/reference/loss/": 9,
    "docs/reference/autograd/": 4,
    "docs/reference/utility/": 1,
    "404.html": 0,
  };
  for (const [page, expected] of Object.entries(EXPECTED)) {
    const html = await (
      await fetch(`http://localhost:4399/${page}`)
    ).text();
    const count = (html.match(/class="shell-tabs/g) ?? []).length;
    check(
      `count map: /${page || "(home)"} has ${expected} group(s)`,
      count === expected,
      `found ${count}`,
    );
  }

  // Fresh visit: bash everywhere, nothing stored.
  await nav(DOCS);
  await evl("localStorage.clear()");
  await nav(DOCS);
  let s = await state();
  check(
    "docs page has three groups",
    s.groups.length === 3,
    `groups=${s.groups.length}`,
  );
  check(
    "fresh visit: bash everywhere, nothing stored",
    s.stored === null
      && s.groups.every(
        (g: any) => g.nuSelected === "false" && g.nuHidden && !g.posixHidden,
      ),
    JSON.stringify(s),
  );

  // Click Nushell on the FIRST group: BOTH groups flip in the same action.
  await evl(
    `document.querySelectorAll('.shell-tabs [data-shell-tab="nu"]')[0].click(), true`,
  );
  s = await state();
  check(
    "one click flips ALL groups",
    s.groups.length === 3
      && s.groups.every(
        (g: any) => g.nuSelected === "true" && !g.nuHidden && g.posixHidden,
      ),
    JSON.stringify(s.groups),
  );
  check("shell=nu stored", s.stored === "nu" && s.rootShell === "nu");

  // Cross-page: the hero follows.
  await nav(HOME);
  s = await state();
  check(
    "hero follows on the homepage",
    s.groups.length === 1
      && s.groups[0].nuSelected === "true"
      && !s.groups[0].nuHidden,
    JSON.stringify(s.groups),
  );

  // Click back to bash on the hero.
  await evl(
    `document.querySelector('.shell-tabs [data-shell-tab="posix"]').click(), true`,
  );
  s = await state();
  check(
    "click back: posix stored, hero flips",
    s.stored === "posix" && s.rootShell === null
      && s.groups[0].posixHidden === false && s.groups[0].nuHidden === true,
    JSON.stringify(s),
  );

  // Legacy-key migration.
  await evl(
    `localStorage.clear(), localStorage.setItem("hero-shell", "nu"), true`,
  );
  await nav(DOCS);
  s = await state();
  check(
    "legacy hero-shell=nu migrates to shell=nu",
    s.stored === "nu" && s.legacy === null
      && s.groups.every((g: any) => !g.nuHidden && g.posixHidden),
    JSON.stringify(s),
  );

  ws.close();
} finally {
  proc.kill();
}

if (failed) process.exit(1);
console.log("shell-tabs matrix passed");
