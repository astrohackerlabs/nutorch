// Browser acceptance for the native-shell website (Issue 26091215074416 Exp 5).
// The historical filename is retained while check:flow replaces the old tab gate.
import {
  existsSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import assert from "node:assert/strict";

function record(value: unknown): Record<string, unknown> {
  assert(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "Expected CDP object",
  );
  return value as Record<string, unknown>;
}

const MAC_BROWSERS = [
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Chromium.app/Contents/MacOS/Chromium",
  "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
  "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
];
const PATH_BROWSERS = [
  "google-chrome",
  "chromium",
  "chromium-browser",
  "msedge",
  "brave-browser",
];
const PORT = 9226;
const urlFlag = process.argv.indexOf("--url");
const SITE = urlFlag < 0 ? "http://127.0.0.1:4398/" : process.argv[urlFlag + 1];
const screenshotFlag = process.argv.indexOf("--screenshots");
const screenshotDirectory =
  screenshotFlag < 0 ? undefined : process.argv[screenshotFlag + 1];
if (!SITE || new URL(SITE).hostname !== "127.0.0.1")
  throw new Error("Local preview URL required");

function pathBinary(name: string): string | null {
  for (const dir of (process.env.PATH ?? "").split(delimiter)) {
    if (!dir) continue;
    const candidate = `${dir}/${name}`;
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

function playwrightBrowsers(): string[] {
  const root = `${String(process.env.HOME)}/Library/Caches/ms-playwright`;
  let dirs: string[] = [];
  try {
    dirs = readdirSync(root);
  } catch {
    return [];
  }
  return dirs
    .filter((dir) => dir.startsWith("chromium-"))
    .sort()
    .reverse()
    .flatMap((dir) => [
      `${root}/${dir}/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`,
      `${root}/${dir}/chrome-mac/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing`,
    ]);
}

function browserPath(): string {
  const candidates = [
    process.env.CHROME,
    ...MAC_BROWSERS,
    ...playwrightBrowsers(),
    ...PATH_BROWSERS.map(pathBinary),
  ].filter((p): p is string => Boolean(p));
  const found = candidates.find((p) => existsSync(p));
  if (found) return found;
  throw new Error(
    "check:flow needs Chrome/Chromium. Set CHROME or install one of: " +
      [
        ...MAC_BROWSERS,
        "~/Library/Caches/ms-playwright/chromium-*",
        ...PATH_BROWSERS,
      ].join(", "),
  );
}

assert(
  !(await fetch(`http://127.0.0.1:${String(PORT)}/json/version`).catch(
    () => null,
  )),
  "Debug port already occupied",
);
const profile = mkdtempSync(`${tmpdir()}/nutorch-flow-`);
const proc = Bun.spawn(
  [
    browserPath(),
    "--headless",
    "--disable-gpu",
    `--remote-debugging-port=${String(PORT)}`,
    `--user-data-dir=${profile}`,
    "about:blank",
  ],
  { stdout: "ignore", stderr: "ignore" },
);
for (let i = 0; i < 30; i++) {
  try {
    await fetch(`http://localhost:${String(PORT)}/json/version`);
    break;
  } catch {
    await new Promise((r) => setTimeout(r, 500));
  }
}

const outcome = { failed: false };
function check(name: string, ok: boolean, detail = ""): void {
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? ` (${detail})` : ""}`);
  if (!ok) outcome.failed = true;
}

try {
  const list: unknown = await (
    await fetch(`http://localhost:${String(PORT)}/json/list`)
  ).json();
  assert(Array.isArray(list), "Expected CDP target list");
  const targets: unknown[] = list;
  const target = targets.map(record).find((t) => t.type === "page");
  assert(
    typeof target?.webSocketDebuggerUrl === "string",
    "Missing page debugger URL",
  );
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  let id = 0;
  const errors: unknown[] = [];
  const pending = new Map<number, (v: unknown) => void>();
  ws.addEventListener("message", (e) => {
    assert(typeof e.data === "string", "Expected CDP text frame");
    const m = record(JSON.parse(e.data) as unknown);
    if (
      m.method === "Runtime.exceptionThrown" ||
      (m.method === "Runtime.consoleAPICalled" &&
        record(m.params).type === "error")
    )
      errors.push(m.params);
    if (typeof m.id === "number" && pending.has(m.id)) {
      pending.get(m.id)?.(m);
      pending.delete(m.id);
    }
  });
  await new Promise((r) => {
    ws.addEventListener("open", r, { once: true });
  });
  const send = async (
    method: string,
    params: object = {},
  ): Promise<Record<string, unknown>> => {
    const response = record(
      await new Promise<unknown>((res, reject) => {
        const timer = setTimeout(() => {
          reject(new Error(`CDP timeout: ${method}`));
        }, 15000);
        pending.set(++id, (value) => {
          clearTimeout(timer);
          res(value);
        });
        ws.send(JSON.stringify({ id, method, params }));
      }),
    );
    if (response.error !== undefined)
      throw new Error(`CDP ${method}: ${JSON.stringify(response.error)}`);
    return record(response.result);
  };
  const evl = async (expression: string): Promise<unknown> => {
    const result = await send("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
      userGesture: true,
    });
    if (result.exceptionDetails !== undefined)
      throw new Error(
        `Browser evaluation failed: ${JSON.stringify(result.exceptionDetails)}`,
      );
    return record(result.result).value;
  };
  const nav = async (u: string): Promise<void> => {
    await send("Page.navigate", { url: u });
    await new Promise((r) => setTimeout(r, 800));
  };
  await send("Page.enable");
  await send("Runtime.enable");
  const waitFor = async (expression: string): Promise<void> => {
    for (let i = 0; i < 60; i++) {
      if (await evl(expression)) return;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    throw new Error(`Browser condition timed out: ${expression}`);
  };
  const codeState = async (): Promise<Record<string, unknown>> =>
    record(
      await evl(`({
    heading: document.querySelector('h1')?.textContent.trim(),
    tabs: document.querySelectorAll('[data-shell-tab], [data-shell-panel], .shell-tabs').length,
    overflow: document.documentElement.scrollWidth > innerWidth + 1,
    codes: [...document.querySelectorAll('pre')].map(p => ({
      text: p.textContent, language: p.dataset.language,
      visible: p.getBoundingClientRect().height > 0 && getComputedStyle(p).display !== 'none'
    }))
  })`),
    );
  const accepted = JSON.parse(
    readFileSync(new URL("./accepted-site.json", import.meta.url), "utf8"),
  ) as { routes: Record<string, string> };
  for (const path of Object.keys(accepted.routes)) {
    const html = await (await fetch(`${SITE}${path}`)).text();
    check(
      `static HTML ${path}`,
      !/data-shell-tab|data-shell-panel|class="[^"]*shell-tabs|data-language="(?:bash|sh|zsh)"/.test(
        html,
      ),
    );
  }
  const pages = [
    "",
    "docs/getting-started/",
    "docs/nushell/",
    "docs/neural-networks/",
    "docs/reference/creation/",
  ];
  for (const width of process.argv.includes("--interactions-only")
    ? []
    : [1280, 390]) {
    await send("Emulation.setDeviceMetricsOverride", {
      width,
      height: 900,
      deviceScaleFactor: 1,
      mobile: false,
    });
    for (const preference of [
      null,
      "posix",
      "nu",
      "legacy-posix",
      "legacy-nu",
    ]) {
      await nav(SITE);
      await evl(
        `localStorage.clear(); ${preference === null ? "" : `localStorage.setItem("${preference.startsWith("legacy-") ? "hero-shell" : "shell"}", "${preference.replace("legacy-", "")}");`}`,
      );
      for (const page of pages) {
        await nav(`${SITE}${page}`);
        await evl("document.fonts.ready.then(() => true)");
        const s = await codeState();
        assert(Array.isArray(s.codes));
        const codes = s.codes.map(record);
        check(
          `${String(width)}/${preference ?? "empty"}/${page || "home"}`,
          s.tabs === 0 &&
            s.overflow === false &&
            codes.length > 0 &&
            codes.every(
              (c) =>
                c.visible === true &&
                c.language === "nu" &&
                typeof c.text === "string" &&
                !c.text.includes("nutorch -c"),
            ),
        );
        if (!page)
          check(
            "shell headline and first import",
            s.heading === "A shell for GPU computing" &&
              String(codes[0]?.text).startsWith("use torch"),
          );
        await send("Page.reload");
        await new Promise((resolve) => setTimeout(resolve, 250));
        check(
          "reload preserves code",
          JSON.stringify((await codeState()).codes) === JSON.stringify(s.codes),
        );
      }
    }
  }

  // The single example remains visible in the initial non-hydrated document.
  await send("Emulation.setScriptExecutionDisabled", { value: true });
  for (const page of process.argv.includes("--interactions-only")
    ? []
    : pages) {
    await nav(`${SITE}${page}`);
    const s = await codeState();
    assert(Array.isArray(s.codes));
    check(
      `no-JS ${page || "home"}`,
      s.tabs === 0 &&
        s.codes.length > 0 &&
        s.codes.map(record).every((c) => c.visible === true),
    );
  }
  await send("Emulation.setScriptExecutionDisabled", { value: false });

  // Verify both real clipboard handlers at desktop and narrow-screen widths.
  const installCommands =
    "brew trust astrohackerlabs/astrohacker\nbrew tap astrohackerlabs/astrohacker\nbrew install nutorch";
  for (const width of [1280, 390]) {
    await send("Emulation.setDeviceMetricsOverride", {
      width,
      height: 900,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await nav(SITE);
    await waitFor(
      "Object.keys(document.querySelector('[data-copy]')).some(k => k.startsWith('__reactProps'))",
    );
    await evl(
      "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
    );
    await send("Emulation.setFocusEmulationEnabled", { enabled: true });
    await send("Browser.grantPermissions", {
      origin: new URL(SITE).origin,
      permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"],
    });
    await evl("document.querySelector('a[href=\"#install\"]').click()");
    await waitFor("location.hash === '#install'");
    check(
      "Homebrew installation heading",
      (await evl(
        "document.querySelector('#install h2').textContent.trim()",
      )) === "Install NuTorch with Homebrew",
    );
    for (const [selector, expected] of [
      ["#copy-install", installCommands],
      ["#copy-launch", "nutorch"],
    ]) {
      assert(selector && expected);
      check(
        `displayed command matches copy payload ${selector}`,
        (await evl(`(() => {
    const button = document.querySelector(${JSON.stringify(selector)});
    const shown = button.parentElement.parentElement.querySelector('pre').textContent.trim();
    return shown === button.dataset.copy && shown === ${JSON.stringify(expected)};
  })()`)) === true,
      );
      await evl(`document.querySelector(${JSON.stringify(selector)}).focus()`);
      check(
        `${String(width)}/${selector} receives focus`,
        (await evl(
          `document.activeElement === document.querySelector(${JSON.stringify(selector)})`,
        )) === true,
      );
      await send("Input.dispatchKeyEvent", {
        type: "keyDown",
        key: "Enter",
        code: "Enter",
        text: "\r",
        windowsVirtualKeyCode: 13,
      });
      await send("Input.dispatchKeyEvent", {
        type: "keyUp",
        key: "Enter",
        code: "Enter",
        windowsVirtualKeyCode: 13,
      });
      await waitFor(
        `document.querySelector(${JSON.stringify(selector)}).textContent === 'copied'`,
      );
      check(
        `${String(width)}/${selector} copies expected commands`,
        (await evl("navigator.clipboard.readText()")) === expected,
      );
    }
    check(
      `${String(width)}/install has no page overflow`,
      (await evl("document.documentElement.scrollWidth <= innerWidth + 1")) ===
        true,
    );
    if (screenshotDirectory) {
      await evl("document.querySelector('#install').scrollIntoView()");
      const shot = await send("Page.captureScreenshot", { format: "png" });
      assert(typeof shot.data === "string");
      writeFileSync(
        join(screenshotDirectory, `homebrew-${String(width)}.png`),
        Buffer.from(shot.data, "base64"),
      );
    }
  }

  // Client navigation and the actual Pagefind search UI.
  await evl("document.querySelector('header a[href=\"/docs/\"]').click()");
  await waitFor(
    "location.pathname === '/docs/' && !!document.querySelector('.pagefind-ui__search-input')",
  );
  await evl(
    "const field = document.querySelector('.pagefind-ui__search-input'); field.value = 'use torch'; field.dispatchEvent(new Event('input', {bubbles:true}))",
  );
  await waitFor(
    "document.querySelectorAll('.pagefind-ui__result-link').length > 0",
  );
  check(
    "search returns documentation",
    (await evl(
      "[...document.querySelectorAll('.pagefind-ui__result-link')].some(a => new URL(a.href).pathname.startsWith('/docs/'))",
    )) === true,
  );

  // Narrow-screen navigation opens by keyboard and closes with Escape.
  await send("Emulation.setDeviceMetricsOverride", {
    width: 390,
    height: 844,
    deviceScaleFactor: 1,
    mobile: true,
  });
  await nav(`${SITE}docs/nushell/`);
  await waitFor("!!document.querySelector('.pagefind-ui__search-input')");
  await evl(
    "[...document.querySelectorAll('button')].find(b => b.textContent === 'Documentation').focus()",
  );
  await send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Enter",
    code: "Enter",
    text: "\r",
    windowsVirtualKeyCode: 13,
  });
  await send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key: "Enter",
    code: "Enter",
    windowsVirtualKeyCode: 13,
  });
  await waitFor("!!document.querySelector('[role=dialog]')");
  check(
    "mobile menu shows document links",
    (await evl(
      "document.querySelectorAll('[role=dialog] a[href^=\"/docs/\"]').length > 10",
    )) === true,
  );
  await send("Input.dispatchKeyEvent", {
    type: "keyDown",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key: "Escape",
    code: "Escape",
    windowsVirtualKeyCode: 27,
  });
  await waitFor("!document.querySelector('[role=dialog]')");
  check("no browser exceptions", errors.length === 0, JSON.stringify(errors));

  ws.close();
} finally {
  proc.kill();
  await proc.exited;
  rmSync(profile, { recursive: true, force: true });
}

if (outcome.failed) process.exit(1);
console.log("native-shell website flow passed");
