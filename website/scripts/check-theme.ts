// Experiment 11: dark-only Austin Night, adaptive PNG favicons, and the
// inherited navigation/search/copy/shell/mobile regression checks.
// Requires the built preview or local Pages on 127.0.0.1:4398.
// Optional --screenshots <directory> records representative visual evidence.
// --menu-only runs the Experiment 12 navigation matrix without the theme sweep.
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import assert from "node:assert/strict";

function record(value: unknown): Record<string, unknown> {
  assert(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "Expected CDP object",
  );
  return value as Record<string, unknown>;
}
function num(value: unknown): number {
  assert(
    typeof value === "number" && Number.isFinite(value),
    "Expected finite CDP number",
  );
  return value;
}
function str(value: unknown): string {
  assert(typeof value === "string", "Expected CDP string");
  return value;
}
function flag(value: unknown): boolean {
  assert(typeof value === "boolean", "Expected CDP boolean");
  return value;
}
function records(value: unknown): Record<string, unknown>[] {
  assert(Array.isArray(value), "Expected CDP array");
  const items: unknown[] = value;
  return items.map(record);
}

const CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const PORT = 9324;
const urlFlag = process.argv.indexOf("--url");
const URL_UNDER_TEST =
  urlFlag < 0 ? "http://127.0.0.1:4398/" : process.argv[urlFlag + 1];
if (!URL_UNDER_TEST || new URL(URL_UNDER_TEST).hostname !== "127.0.0.1")
  throw new Error("Local preview URL required");

const proc = Bun.spawn(
  [
    CHROME,
    "--headless",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    `--remote-debugging-port=${String(PORT)}`,
    // Isolated profile: without it, launch can delegate to a running
    // Chrome and exit, leaving nothing listening on the port.
    `--user-data-dir=${mkdtempSync(`${tmpdir()}/nutorch-theme-`)}`,
    "about:blank",
  ],
  { stdout: "ignore", stderr: "ignore" },
);
// Poll for the DevTools endpoint (startup time varies; a fixed sleep races).
async function waitForDevtools(): Promise<void> {
  for (let i = 0; i < 30; i++) {
    try {
      await fetch(`http://localhost:${String(PORT)}/json/version`);
      return;
    } catch {
      await new Promise((r) => setTimeout(r, 500));
    }
  }
  throw new Error("DevTools endpoint never came up");
}
await waitForDevtools();

const outcome = { failed: false };
function check(name: string, value: unknown, detail = ""): void {
  const ok = flag(value);
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? ` (${detail})` : ""}`);
  if (!ok) outcome.failed = true;
}

try {
  const list = records(
    await (await fetch(`http://localhost:${String(PORT)}/json/list`)).json(),
  );
  const page = list.find((t) => t.type === "page");
  if (!page) throw new Error("no page target");
  const ws = new WebSocket(str(page.webSocketDebuggerUrl));
  let id = 0;
  const browserErrors: unknown[] = [];
  const pending = new Map<number, (v: unknown) => void>();
  ws.addEventListener("message", (event) => {
    const msg = record(JSON.parse(str(event.data)) as unknown);
    if (
      msg.method === "Runtime.exceptionThrown" ||
      (msg.method === "Runtime.consoleAPICalled" &&
        record(msg.params).type === "error")
    )
      browserErrors.push(msg.params);
    if (typeof msg.id === "number" && pending.has(msg.id)) {
      pending.get(msg.id)?.(msg);
      pending.delete(msg.id);
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
      await new Promise<unknown>((resolve) => {
        pending.set(++id, resolve);
        ws.send(JSON.stringify({ id, method, params }));
      }),
    );
    if (response.error !== undefined)
      throw new Error(`CDP ${method}: ${JSON.stringify(response.error)}`);
    return record(response.result);
  };
  const evaluate = async (expression: string): Promise<unknown> => {
    const response = await send("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (response.exceptionDetails !== undefined)
      throw new Error(JSON.stringify(response.exceptionDetails));
    return record(response.result).value;
  };
  const evaluateRecord = async (
    expression: string,
  ): Promise<Record<string, unknown>> => record(await evaluate(expression));
  const evaluateFlag = async (expression: string): Promise<boolean> =>
    flag(await evaluate(expression));
  const evaluateNumber = async (expression: string): Promise<number> =>
    num(await evaluate(expression));
  const emulate = (
    scheme: "light" | "dark",
  ): Promise<Record<string, unknown>> =>
    send("Emulation.setEmulatedMedia", {
      features: [{ name: "prefers-color-scheme", value: scheme }],
    });
  const tabIconIs = async (scheme: string): Promise<boolean> => {
    for (let attempt = 0; attempt < 30; attempt++) {
      const targets = records(
        await (
          await fetch(`http://localhost:${String(PORT)}/json/list`)
        ).json(),
      );
      const current = targets.find(
        (target) => target.webSocketDebuggerUrl === page.webSocketDebuggerUrl,
      );
      if (current?.faviconUrl === `${URL_UNDER_TEST}favicon-${scheme}.png`)
        return true;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    return false;
  };
  const navigate = async (url: string): Promise<void> => {
    await send("Page.navigate", { url });
    await new Promise((r) => setTimeout(r, 800));
    for (let attempt = 0; attempt < 50; attempt++) {
      if (
        flag(
          await evaluate(
            `location.href === ${JSON.stringify(url)} && document.readyState === 'complete' && !!document.querySelector('main h1')`,
          ),
        )
      )
        return;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    throw new Error(`Navigation did not reach a rendered page: ${url}`);
  };
  const state = async (): Promise<Record<string, unknown>> =>
    record(
      await evaluate(`({
      setting: document.documentElement.dataset.themeSetting ?? null,
      theme: document.documentElement.dataset.theme ?? null,
      stored: (() => { try { return localStorage.getItem("theme"); } catch { return "ERR"; } })(),
      background: getComputedStyle(document.documentElement).backgroundColor,
      colorScheme: getComputedStyle(document.documentElement).colorScheme,
      font: getComputedStyle(document.body).fontFamily,
      headingFont: getComputedStyle(document.querySelector('h1')).fontFamily,
      toggle: !!document.querySelector('#theme-toggle'),
      rain: document.querySelectorAll('[data-space-rain-canvas]').length,
    })`),
    );

  await send("Page.enable");
  await send("Runtime.enable");

  const waitFor = async (expression: string): Promise<boolean> => {
    for (let attempt = 0; attempt < 50; attempt++) {
      if (await evaluate(expression)) return true;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    return false;
  };

  // Experiment 12: exercise real scrolling and keyboard navigation, not classes.
  const rail = 'nav[aria-label="Documentation"] > div:last-child';
  const pause = (): Promise<void> =>
    new Promise((resolve) => {
      setTimeout(resolve, 250);
    });
  const metrics = (
    width: number,
    height: number,
  ): Promise<Record<string, unknown>> =>
    send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: false,
    });
  const key = async (key: string, code: number): Promise<void> => {
    await send("Input.dispatchKeyEvent", {
      type: "keyDown",
      key,
      windowsVirtualKeyCode: code,
    });
    await send("Input.dispatchKeyEvent", {
      type: "keyUp",
      key,
      windowsVirtualKeyCode: code,
    });
    await pause();
  };
  const clickLast = async (selector: string): Promise<string> => {
    const point = await evaluateRecord(
      `(() => { const a = document.querySelector(${JSON.stringify(selector)}).querySelector('a:last-of-type'); const links = document.querySelector(${JSON.stringify(selector)}).querySelectorAll('a'); const last = links[links.length-1]; last.scrollIntoView({block:'nearest',behavior:'instant'}); const r=last.getBoundingClientRect(); return {x:r.x+r.width/2,y:r.y+r.height/2,path:new URL(last.href).pathname}; })()`,
    );
    await send("Input.dispatchMouseEvent", {
      type: "mousePressed",
      x: num(point.x),
      y: num(point.y),
      button: "left",
      clickCount: 1,
    });
    await send("Input.dispatchMouseEvent", {
      type: "mouseReleased",
      x: num(point.x),
      y: num(point.y),
      button: "left",
      clickCount: 1,
    });
    await pause();
    return str(point.path);
  };
  for (const route of ["docs/reference/utility/", "docs/reference/linalg/"]) {
    for (const [width, height] of [
      [1440, 900],
      [1280, 400],
    ] as const) {
      await metrics(width, height);
      await navigate(`${URL_UNDER_TEST}${route}`);
      await evaluate(`document.fonts.ready`);
      // Wait for hydration before overriding React Router's restored position.
      for (let attempt = 0; attempt < 40; attempt++) {
        if (await evaluate(`!!document.querySelector('.pagefind-ui input')`))
          break;
        await pause();
      }
      await evaluate(`new Promise(resolve => setTimeout(resolve, 1000))`);
      await evaluate(
        `scrollTo({top:0,behavior:'instant'}); document.querySelector('${rail}').scrollTop=0; true`,
      );
      await pause();
      const bounds = await evaluateRecord(
        `(() => { const e=document.querySelector('${rail}'); const r=e.getBoundingClientRect(); return {height:e.clientHeight,content:e.scrollHeight,bottom:r.bottom,overflow:getComputedStyle(e).overflowY}; })()`,
      );
      check(
        `menu bounded ${route} ${String(height)}`,
        num(bounds.bottom) <= height && bounds.overflow === "auto",
        JSON.stringify(bounds),
      );
      if (height === 400)
        check(
          "short menu overflows internally",
          num(bounds.content) > num(bounds.height),
        );
      const point = await evaluateRecord(
        `(() => {const r=document.querySelector('${rail}').getBoundingClientRect(); return {x:r.x+20,y:r.y+30,page:scrollY};})()`,
      );
      await send("Input.dispatchMouseEvent", {
        type: "mouseMoved",
        x: num(point.x),
        y: num(point.y),
      });
      await pause();
      await send("Input.dispatchMouseEvent", {
        type: "mouseWheel",
        x: num(point.x),
        y: num(point.y),
        deltaX: 0,
        deltaY: 150,
      });
      await pause();
      if (num(bounds.content) > num(bounds.height)) {
        const wheel = await evaluateRecord(
          `({menu:document.querySelector('${rail}').scrollTop,page:scrollY})`,
        );
        check(
          "wheel scrolls menu independently",
          num(wheel.menu) > 0 && num(wheel.page) === num(point.page),
          JSON.stringify({ before: point.page, after: wheel }),
        );
      }
      await evaluate(`document.querySelector('${rail} a').focus(); true`);
      const count = await evaluateNumber(
        `document.querySelectorAll('${rail} a').length`,
      );
      let allVisible = true;
      for (let i = 0; i < count; i++) {
        allVisible &&= await evaluateFlag(
          `(() => {const e=document.querySelector('${rail}'); const a=document.activeElement; const r=a.getBoundingClientRect(), p=e.getBoundingClientRect(); return e.contains(a) && r.top>=p.top && r.bottom<=p.bottom && r.bottom<=innerHeight && getComputedStyle(a).outlineStyle!=='none';})()`,
        );
        if (i < count - 1) await key("Tab", 9);
      }
      check(`all menu links keyboard reachable ${String(height)}`, allVisible);
      const target = await evaluate(
        "new URL(document.activeElement.href).pathname",
      );
      await key("Enter", 13);
      check(
        "last link Enter navigation and active state",
        await evaluate(
          `location.pathname===${JSON.stringify(target)} && document.querySelector('${rail} a[aria-current="page"]').pathname===location.pathname`,
        ),
      );
      await navigate(`${URL_UNDER_TEST}${route}`);
      const clicked = await clickLast(rail);
      check(
        "last link pointer navigation",
        await evaluate(`location.pathname===${JSON.stringify(clicked)}`),
      );
      await navigate(`${URL_UNDER_TEST}${route}`);
      // Stay within the rail's sticky travel; at the footer it must yield to
      // its containing block (as tscom does), not overlap the footer.
      await evaluate(
        `(() => {const e=document.querySelector('${rail}'); const slot=e.parentElement; const travel=slot.getBoundingClientRect().bottom+scrollY-e.clientHeight-96; scrollTo({top:Math.max(0,Math.min(200,travel-8)),behavior:'instant'});})()`,
      );
      await pause();
      check(
        "menu stays below header during article scrolling",
        await evaluate(
          `document.querySelector('${rail}').getBoundingClientRect().top>=64`,
        ),
      );
      await evaluate(
        `scrollTo({top:document.documentElement.scrollHeight,behavior:'instant'}); true`,
      );
      await pause();
      check(
        "rail does not overlap footer at article bottom",
        await evaluate(
          `(() => {const r=document.querySelector('${rail}').getBoundingClientRect();return r.bottom<=document.querySelector('footer').getBoundingClientRect().top;})()`,
        ),
      );
    }
  }
  await metrics(1440, 900);
  await navigate(`${URL_UNDER_TEST}docs/`);
  await metrics(1280, 400);
  await pause();
  check(
    "resize bounds menu without reload",
    await evaluate(
      `document.querySelector('${rail}').getBoundingClientRect().bottom<=innerHeight`,
    ),
  );
  const captures = process.argv.indexOf("--screenshots");
  if (captures >= 0) {
    const directory = process.argv[captures + 1];
    assert(directory, "Missing --screenshots directory");
    mkdirSync(directory, { recursive: true });
    for (const [name, position] of [
      ["first", 0],
      ["last", 10000],
    ] as const) {
      await evaluate(
        `document.querySelector('${rail}').scrollTop=${String(position)}; true`,
      );
      await pause();
      const shot = await send("Page.captureScreenshot", {
        format: "png",
        captureBeyondViewport: false,
      });
      writeFileSync(
        join(directory, `menu-${name}-1280x400.png`),
        Buffer.from(str(shot.data), "base64"),
      );
    }
  }
  for (const height of [400, 844]) {
    await metrics(390, height);
    await navigate(`${URL_UNDER_TEST}docs/`);
    await evaluate(
      `document.querySelector('[data-slot=sheet-trigger]').click()`,
    );
    await pause();
    const target = await clickLast("[data-slot=sheet-content]");
    check(
      `mobile last link closes and navigates ${String(height)}`,
      await waitFor(
        `location.pathname===${JSON.stringify(target)} && !document.querySelector('[data-slot=sheet-content]')`,
      ),
    );
    await evaluate(
      `document.querySelector('[data-slot=sheet-trigger]').click()`,
    );
    await pause();
    await key("Escape", 27);
    check(
      `mobile Escape focus and no overflow ${String(height)}`,
      await evaluate(
        `!document.querySelector('[data-slot=sheet-content]') && document.activeElement.dataset.slot==='sheet-trigger' && document.documentElement.scrollWidth===innerWidth`,
      ),
    );
  }
  await metrics(1440, 900);

  if (process.argv.includes("--menu-only")) {
    check(
      "menu matrix has no runtime errors",
      browserErrors.length === 0,
      JSON.stringify(browserErrors),
    );
    ws.close();
    proc.kill();
    console.log(outcome.failed ? "menu matrix failed" : "menu matrix passed");
    process.exit(outcome.failed ? 1 : 0);
  }

  await navigate(URL_UNDER_TEST);
  for (const stored of [null, "light", "system", "dark"]) {
    await evaluate(
      `localStorage.clear(); ${stored ? `localStorage.setItem('theme', '${stored}');` : ""}`,
    );
    for (const scheme of ["light", "dark"] as const) {
      await emulate(scheme);
      await navigate(URL_UNDER_TEST);
      const s = await state();
      check(
        `dark-only, OS ${scheme}, saved ${String(stored)}`,
        s.theme === "dark" &&
          s.setting === null &&
          s.stored === stored &&
          s.background === "rgb(17, 18, 25)" &&
          s.colorScheme === "dark" &&
          !flag(s.toggle),
        JSON.stringify(s),
      );
      check(
        "shared fonts and one rain canvas",
        str(s.font).includes("Space Grotesk") &&
          str(s.headingFont).includes("Space Grotesk") &&
          s.rain === 1,
      );
      const icon = await evaluateRecord(`(async () => {
        const links = [...document.querySelectorAll('link[rel=icon]')];
        const selected = links.filter(l => !l.media || matchMedia(l.media).matches).at(-1);
        const image = new Image(); image.src = selected.href; await image.decode();
        const canvas = document.createElement('canvas'); canvas.width = canvas.height = 32;
        const ctx = canvas.getContext('2d'); ctx.drawImage(image, 0, 0);
        const pixels = ctx.getImageData(0, 0, 32, 32).data;
        const colors = new Set(); let transparent = false;
        for (let i=0;i<pixels.length;i+=4) { if(pixels[i+3]===255) colors.add([...pixels.slice(i,i+3)].join(',')); if(pixels[i+3]===0) transparent=true; }
        return {path: new URL(selected.href).pathname, width: image.width, height: image.height, colors: [...colors], transparent};
      })()`);
      check(
        `actual browser tab selects ${scheme} PNG`,
        await tabIconIs(scheme),
      );
      check(
        `PNG favicon ${scheme} fetched pixels`,
        icon.path === `/favicon-${scheme}.png` &&
          icon.width === 32 &&
          icon.height === 32 &&
          flag(icon.transparent) &&
          Array.isArray(icon.colors) &&
          icon.colors.length === 1 &&
          icon.colors[0] === (scheme === "dark" ? "27,254,255" : "46,125,233"),
        JSON.stringify(icon),
      );
    }
  }
  await emulate("light");
  check(
    "actual browser tab switches to light PNG live",
    await tabIconIs("light"),
  );
  check(
    "live OS switch leaves page dark",
    (await state()).background === "rgb(17, 18, 25)",
  );
  check(
    "live OS switch selects light favicon",
    await evaluate(
      `new URL([...document.querySelectorAll('link[rel=icon]')].filter(l=>!l.media||matchMedia(l.media).matches).at(-1).href).pathname === '/favicon-light.png'`,
    ),
  );
  await send("Emulation.setScriptExecutionDisabled", { value: true });
  await navigate(URL_UNDER_TEST);
  check(
    "pre-hydration page already dark",
    (await state()).background === "rgb(17, 18, 25)",
  );
  await send("Emulation.setScriptExecutionDisabled", { value: false });
  await navigate(URL_UNDER_TEST);

  // React navigation must retain one search instance and live controls.
  check(
    "Space Rain renderer runs",
    await waitFor(
      `document.querySelector('[data-space-rain-canvas]').dataset.spaceRainStatus === 'running'`,
    ),
  );
  await send("Emulation.setEmulatedMedia", {
    features: [{ name: "prefers-reduced-motion", value: "reduce" }],
  });
  check(
    "reduced motion pauses rain",
    await waitFor(
      `document.querySelector('[data-space-rain-canvas]').dataset.spaceRainStatus === 'paused'`,
    ),
  );
  await send("Emulation.setEmulatedMedia", {
    features: [{ name: "prefers-reduced-motion", value: "no-preference" }],
  });
  check(
    "rain resumes after preference change",
    await waitFor(
      `document.querySelector('[data-space-rain-canvas]').dataset.spaceRainStatus === 'running'`,
    ),
  );
  await navigate(`${URL_UNDER_TEST}docs/nushell/`);
  await evaluate(
    `window.__rainCanvas = document.querySelector('[data-space-rain-canvas]'); true`,
  );
  check(
    "search mounts in built docs",
    await waitFor(
      'document.querySelectorAll(".pagefind-ui input").length === 1',
    ),
  );
  await evaluate('window.__navigationProbe = "retained"');
  await evaluate(
    '(()=>{const input=document.querySelector(".pagefind-ui input");input.value="NU_LIB_DIRS";input.dispatchEvent(new Event("input",{bubbles:true}));})()',
  );
  check(
    "search finds native shell setup",
    await waitFor(
      '[...document.querySelectorAll("#docs-search a")].some(a=>a.getAttribute("href")==="/docs/nushell/#setup")',
    ),
  );
  await evaluate(
    '[...document.querySelectorAll("#docs-search a")].find(a=>a.getAttribute("href")==="/docs/nushell/#setup").click()',
  );
  check(
    "search result reaches heading",
    await waitFor(
      'location.hash === "#setup" && document.getElementById("setup").getBoundingClientRect().top >= 64 && document.getElementById("setup").getBoundingClientRect().top < 90',
    ),
  );
  await evaluate(
    '[...document.querySelectorAll("nav a")].find(a=>a.getAttribute("href")==="/docs/getting-started/").click()',
  );
  check(
    "sidebar uses client navigation",
    await waitFor(
      'location.pathname === "/docs/getting-started/" && window.__navigationProbe === "retained"',
    ),
  );
  check(
    "search remounts once",
    await waitFor('document.querySelectorAll(".pagefind-ui").length === 1'),
  );
  check(
    "native examples remain visible after client navigation",
    await evaluate(
      '[...document.querySelectorAll("pre")].some(p => p.textContent.startsWith("use torch")) && [...document.querySelectorAll("pre")].every(p => p.getBoundingClientRect().height > 0) && !document.querySelector("[data-shell-tab]")',
    ),
  );
  await evaluate("history.back()");
  check(
    "back restores docs and one search",
    await waitFor(
      'location.pathname === "/docs/nushell/" && document.querySelectorAll(".pagefind-ui").length === 1',
    ),
  );
  await evaluate("history.forward()");
  check(
    "forward restores docs and one search",
    await waitFor(
      'location.pathname === "/docs/getting-started/" && document.querySelectorAll(".pagefind-ui").length === 1',
    ),
  );
  await evaluate(
    '[...document.querySelectorAll("header a")].find(a=>a.getAttribute("href")==="/").click()',
  );
  check(
    "header navigates without document replacement",
    await waitFor(
      'location.pathname === "/" && window.__navigationProbe === "retained"',
    ),
  );
  check(
    "navigation retains a single live rain renderer",
    await evaluate(
      `document.querySelectorAll('[data-space-rain-canvas]').length===1 && document.querySelector('[data-space-rain-canvas]')===window.__rainCanvas && window.__rainCanvas.dataset.spaceRainStatus==='running'`,
    ),
  );
  await send("Emulation.setFocusEmulationEnabled", { enabled: true });
  await send("Browser.grantPermissions", {
    origin: URL_UNDER_TEST.slice(0, -1),
    permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"],
  });
  await evaluate('document.getElementById("copy-install").click()');
  check(
    "copy button reports success",
    await waitFor(
      'document.getElementById("copy-install").textContent === "copied"',
    ),
  );
  const clipboard = await send("Runtime.evaluate", {
    expression: "navigator.clipboard.readText()",
    awaitPromise: true,
    returnByValue: true,
  });
  check(
    "copy payload is exact",
    record(clipboard.result).value ===
      "./code/nutorch/rs/target/release/nutorch",
  );
  await send("Emulation.setDeviceMetricsOverride", {
    width: 390,
    height: 900,
    deviceScaleFactor: 1,
    mobile: false,
  });
  await navigate(`${URL_UNDER_TEST}docs/nushell/`);
  await evaluate('document.querySelector("[data-slot=sheet-trigger]").click()');
  check(
    "mobile navigation opens",
    await waitFor('!!document.querySelector("[data-slot=sheet-content]")'),
  );
  check(
    "mobile has no page overflow",
    await evaluate("document.documentElement.scrollWidth === innerWidth"),
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
  check(
    "mobile Sheet closes with Escape and restores focus",
    await waitFor(
      `!document.querySelector('[data-slot=sheet-content]') && document.activeElement?.dataset.slot==='sheet-trigger'`,
    ),
  );
  const blocked = await send("Page.addScriptToEvaluateOnNewDocument", {
    source: `Object.defineProperty(window,'localStorage',{get(){throw new Error('Storage blocked by fixture')}})`,
  });
  await navigate(`${URL_UNDER_TEST}docs/getting-started/`);
  check(
    "blocked storage retains dark page and visible native examples",
    await evaluate(
      `document.documentElement.dataset.theme==='dark' && [...document.querySelectorAll('pre')].some(p => p.textContent.startsWith('use torch')) && [...document.querySelectorAll('pre')].every(p => p.getBoundingClientRect().height > 0) && !document.querySelector('[data-shell-tab]')`,
    ),
  );
  await send("Page.removeScriptToEvaluateOnNewDocument", {
    identifier: blocked.identifier,
  });
  await navigate(URL_UNDER_TEST);
  check(
    "no hydration or runtime errors",
    browserErrors.length === 0,
    JSON.stringify(browserErrors),
  );
  for (const path of ["unknown-route", "docs/unknown-document/"]) {
    const response = await fetch(`${URL_UNDER_TEST}${path}`);
    check(
      `static 404: ${path}`,
      response.status === 404 &&
        (await response.text()).includes("unknown handle:"),
    );
  }
  const screenshotFlag = process.argv.indexOf("--screenshots");
  if (screenshotFlag !== -1) {
    const directory = process.argv[screenshotFlag + 1];
    if (!directory) throw new Error("--screenshots requires a directory");
    mkdirSync(directory, { recursive: true });
    for (const width of [1440, 390]) {
      await send("Emulation.setDeviceMetricsOverride", {
        width,
        height: 900,
        deviceScaleFactor: 1,
        mobile: false,
      });
      for (const theme of ["light", "dark"]) {
        await emulate(theme as "light" | "dark");
        await evaluate(
          `localStorage.setItem("theme", ${JSON.stringify(theme)}); localStorage.setItem("shell", "posix")`,
        );
        for (const route of [
          "",
          "docs/",
          "docs/nushell/",
          "docs/reference/linalg/",
          "404/",
        ]) {
          await navigate(`${URL_UNDER_TEST}${route}`);
          await send("Runtime.evaluate", {
            expression: "document.fonts.ready",
            awaitPromise: true,
          });
          check(
            `no overflow: ${route || "home"} ${String(width)}`,
            await evaluate(
              "document.documentElement.scrollWidth === innerWidth",
            ),
          );
          const screenshot = await send("Page.captureScreenshot", {
            format: "png",
            captureBeyondViewport: false,
          });
          writeFileSync(
            join(
              directory,
              `${route ? route.replaceAll("/", "-") : "home"}-${theme}-${String(width)}.png`,
            ),
            Buffer.from(str(screenshot.data), "base64"),
          );
          if (route === "" && theme === "dark") {
            await evaluate(
              "scrollTo({top:document.documentElement.scrollHeight,behavior:'instant'}); true",
            );
            const footer = await send("Page.captureScreenshot", {
              format: "png",
              captureBeyondViewport: false,
            });
            writeFileSync(
              join(directory, `home-footer-${String(width)}.png`),
              Buffer.from(str(footer.data), "base64"),
            );
          }
        }
      }
    }
    check(
      "no hydration or runtime errors across visual route sweep",
      browserErrors.length === 0,
      JSON.stringify(browserErrors),
    );
    if (process.argv.includes("--references")) {
      for (const [name, url] of [
        ["termsurf-com", "http://127.0.0.1:3510/"],
        ["astrohacker-com", "http://127.0.0.1:5173/"],
      ] as const) {
        for (const width of [1440, 390]) {
          await send("Emulation.setDeviceMetricsOverride", {
            width,
            height: 900,
            deviceScaleFactor: 1,
            mobile: false,
          });
          await navigate(url);
          if (!(await waitFor("!!document.querySelector('h1')")))
            throw new Error(`${name} reference did not render its heading`);
          await evaluate("document.fonts.ready");
          const reference = await evaluateRecord(
            `({background:getComputedStyle(document.body).backgroundColor,font:getComputedStyle(document.body).fontFamily,heading:getComputedStyle(document.querySelector('h1')).fontFamily})`,
          );
          check(
            `${name} shares ntcom ground and fonts`,
            reference.background === "rgb(17, 18, 25)" &&
              str(reference.font).includes("Space Grotesk") &&
              str(reference.heading).includes("Space Grotesk"),
            JSON.stringify(reference),
          );
          const screenshot = await send("Page.captureScreenshot", {
            format: "png",
            captureBeyondViewport: false,
          });
          writeFileSync(
            join(directory, `${name}-${String(width)}.png`),
            Buffer.from(str(screenshot.data), "base64"),
          );
        }
      }
    }
    console.log(`Visual evidence: ${directory}`);
  }
  ws.close();
} finally {
  proc.kill();
}

if (outcome.failed) process.exit(1);
console.log("theme matrix passed");
