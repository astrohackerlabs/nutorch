import { afterAll, beforeAll, expect, test } from "bun:test";
import { chromium } from "playwright";
import type { Browser, BrowserContext, Page } from "playwright";
import { resolve } from "node:path";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { UMAMI_SCRIPT_URL, UMAMI_WEBSITE_ID } from "./umami";

const root = resolve(import.meta.dir, "../..");
const production = "http://127.0.0.1:4401";
const development = "http://127.0.0.1:4402";
const debug = "http://127.0.0.1:9227";
let profile: string | undefined;
let browser: Browser | undefined;
let trackerSource: string;
const servers: ReturnType<typeof Bun.spawn>[] = [];
const contexts: BrowserContext[] = [];

beforeAll(async () => {
  for (const url of [production, development, `${debug}/json/version`]) {
    if (
      await fetch(url).then(
        () => true,
        () => false,
      )
    ) {
      throw new Error(`Test port already occupied: ${url}`);
    }
  }
  const response = await fetch(UMAMI_SCRIPT_URL);
  expect(response.ok).toBe(true);
  trackerSource = await response.text();
  console.log("Tracker evidence", {
    retrieved: new Date().toISOString(),
    sha256: new Bun.CryptoHasher("sha256").update(trackerSource).digest("hex"),
  });
  servers.push(
    Bun.spawn(["bun", "server.ts", "--port", "4401"], {
      cwd: root,
      stdout: "ignore",
      stderr: "ignore",
    }),
    Bun.spawn(
      [
        resolve(root, "node_modules/.bin/react-router"),
        "dev",
        "--host",
        "127.0.0.1",
        "--port",
        "4402",
      ],
      {
        cwd: root,
        stdout: "ignore",
        stderr: "ignore",
      },
    ),
  );
  for (const url of [production, development]) {
    let ready = false;
    for (let i = 0; i < 100; i++) {
      if (
        await fetch(url).then(
          (res) => res.ok,
          () => false,
        )
      ) {
        ready = true;
        break;
      }
      await Bun.sleep(100);
    }
    expect(ready).toBe(true);
  }
  // Own the process directly: Playwright's child-process shutdown hangs under
  // this Bun version. CDP keeps browser cleanup under the test's control.
  profile = mkdtempSync(resolve(tmpdir(), "ntcom-analytics-"));
  servers.push(
    Bun.spawn(
      [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "--headless",
        "--remote-debugging-port=9227",
        `--user-data-dir=${profile}`,
        "--no-first-run",
        "about:blank",
      ],
      { stdout: "ignore", stderr: "ignore" },
    ),
  );
  for (let i = 0; i < 100; i++) {
    if (
      await fetch(`${debug}/json/version`).then(
        (res) => res.ok,
        () => false,
      )
    )
      break;
    await Bun.sleep(100);
  }
  browser = await chromium.connectOverCDP(debug);
}, 30_000);

afterAll(async () => {
  await Promise.all(contexts.map((context) => context.close()));
  await browser?.close();
  for (const server of servers) {
    server.kill("SIGKILL");
    await server.exited;
  }
  if (profile) rmSync(profile, { recursive: true, force: true });
}, 15_000);

interface Event {
  type: string;
  payload: { website: string; url: string; title: string; referrer: string };
}

async function fixture(
  options: {
    delayed?: Promise<void>;
    block?: boolean;
    throwing?: boolean;
    collectorFailure?: boolean;
  } = {},
): Promise<{ page: Page; events: Event[]; errors: string[] }> {
  if (!browser) throw new Error("Browser setup did not complete");
  const context = await browser.newContext({
    permissions: ["clipboard-read", "clipboard-write"],
  });
  contexts.push(context);
  const page = await context.newPage();
  page.setDefaultTimeout(8_000);
  const events: Event[] = [];
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.route(UMAMI_SCRIPT_URL, async (route) => {
    await options.delayed;
    if (options.block) return route.abort();
    return route.fulfill({
      contentType: "application/javascript",
      body: options.throwing
        ? "window.umami = {track() { throw new Error('fixture tracker failure'); }};"
        : trackerSource,
    });
  });
  await page.route("https://umami.astrohacker.com/api/send", async (route) => {
    if (route.request().method() === "POST") {
      events.push(route.request().postDataJSON() as Event);
    }
    if (options.collectorFailure) return route.abort();
    return route.fulfill({
      contentType: "application/json",
      body: "{}",
      headers: {
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Headers": "*",
      },
    });
  });
  return { page, events, errors };
}

async function hydrated(page: Page): Promise<void> {
  await page.waitForFunction(() => {
    const link = document.querySelector("header a");
    return (
      link && Object.keys(link).some((key) => key.startsWith("__reactProps$"))
    );
  });
}

async function settle(): Promise<void> {
  // The actual script schedules automatic history pageviews after 300 ms.
  await Bun.sleep(700);
}

function urls(events: Event[]): string[] {
  for (const event of events) {
    expect(event.type).toBe("event");
    expect(event.payload.website).toBe(UMAMI_WEBSITE_ID);
    expect(event.payload.title.length).toBeGreaterThan(0);
  }
  return events.map((event) => {
    const url = new URL(event.payload.url);
    return `${url.pathname}${url.search}`;
  });
}

test("all production HTML has exactly one correctly configured script", async () => {
  const glob = new Bun.Glob("**/*.html");
  const files = await Array.fromAsync(glob.scan(resolve(root, "build/client")));
  expect(files.length).toBeGreaterThan(20);
  for (const path of files) {
    const html = await Bun.file(resolve(root, "build/client", path)).text();
    const scripts = html.match(/<script\b[^>]*data-umami-tracker[^>]*>/g) ?? [];
    expect(scripts.length).toBe(1);
    expect(scripts[0]).toContain(`src="${UMAMI_SCRIPT_URL}"`);
    expect(scripts[0]).toContain(`data-website-id="${UMAMI_WEBSITE_ID}"`);
    expect(scripts[0]).toContain('data-auto-pageview="false"');
    expect(scripts[0]).toMatch(/\bdefer(?:="")?[\s>]/);
  }
  console.log("Production HTML documents checked:", files.length);
});

test("real tracker counts load, routes, query, back and forward exactly once", async () => {
  const { page, events, errors } = await fixture();
  await page.goto(production);
  await hydrated(page);
  await settle();
  expect(urls(events)).toEqual(["/"]);
  await page
    .locator("header")
    .getByRole("link", { name: "Docs", exact: true })
    .click();
  await page.waitForURL(`${production}/docs/`);
  await settle();
  await page.locator('a[href="/docs/getting-started/"]').first().click();
  await page.waitForURL(`${production}/docs/getting-started/`);
  await settle();
  await page.evaluate(() => {
    history.pushState(null, "", "?analytics=experiment1");
    dispatchEvent(new PopStateEvent("popstate"));
  });
  await settle();
  await page.goBack();
  await settle();
  await page.goForward();
  await settle();
  const expected = [
    "/",
    "/docs/",
    "/docs/getting-started/",
    "/docs/getting-started/?analytics=experiment1",
    "/docs/getting-started/",
    "/docs/getting-started/?analytics=experiment1",
  ];
  expect(urls(events)).toEqual(expected);
  await page.evaluate(() => {
    location.hash = "analytics-hash";
  });
  await page.getByRole("radio", { name: "No motion", exact: true }).click();
  await settle();
  expect(urls(events)).toEqual(expected);
  expect(errors).toEqual([]);

  const direct = await fixture();
  await direct.page.goto(`${production}/docs/getting-started/`);
  await settle();
  expect(urls(direct.events)).toEqual(["/docs/getting-started/"]);
}, 30_000);

test("delayed script preserves visits made after hydration", async () => {
  let release!: () => void;
  const delayed = new Promise<void>((resolve) => {
    release = resolve;
  });
  const { page, events, errors } = await fixture({ delayed });
  try {
    await page.goto(production, { waitUntil: "commit" });
    await hydrated(page);
    const firstTitle = await page.title();
    await page
      .locator("header")
      .getByRole("link", { name: "Docs", exact: true })
      .click();
    await page.waitForURL(`${production}/docs/`, { waitUntil: "commit" });
    await settle();
    const secondTitle = await page.title();
    expect(events).toEqual([]);
    release();
    await settle();
    expect(urls(events)).toEqual(["/", "/docs/"]);
    expect(events.map((event) => event.payload.title)).toEqual([
      firstTitle,
      secondTitle,
    ]);
    expect(events[1]?.payload.referrer).toBe(`${production}/`);
    expect(errors).toEqual([]);
  } finally {
    release();
  }
}, 30_000);

test("script before hydration waits for the router's initial visit", async () => {
  const { page, events, errors } = await fixture();
  let release!: () => void;
  const delayed = new Promise<void>((resolve) => {
    release = resolve;
  });
  await page.route("**/assets/*.js", async (route) => {
    await delayed;
    await route.continue();
  });
  try {
    await page.goto(production, { waitUntil: "commit" });
    await page.waitForFunction(() => typeof window.umami?.track === "function");
    await settle();
    expect(events).toEqual([]);
    release();
    await hydrated(page);
    await settle();
    expect(urls(events)).toEqual(["/"]);
    expect(errors).toEqual([]);
  } finally {
    release();
  }
}, 30_000);

test("tracker failures preserve navigation, search and copy", async () => {
  for (const options of [
    { block: true },
    { throwing: true },
    { collectorFailure: true },
  ]) {
    const { page, errors } = await fixture(options);
    await page.goto(production);
    await hydrated(page);
    const copy = page.locator("[data-copy]").first();
    const expected = await copy.getAttribute("data-copy");
    if (expected === null) throw new Error("Missing copy payload");
    await copy.click();
    expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(
      expected,
    );
    await page
      .locator("header")
      .getByRole("link", { name: "Docs", exact: true })
      .click();
    await page.waitForURL(`${production}/docs/`);
    const search = page.locator(".pagefind-ui__search-input");
    await search.fill("tensor");
    await page.locator(".pagefind-ui__result-link").first().waitFor();
    await page
      .locator("header")
      .getByRole("link", { name: "Docs", exact: true })
      .click();
    await page.waitForURL(`${production}/docs/`);
    await settle();
    expect(errors).toEqual([]);
  }
}, 30_000);

test("development load and navigation never load or send analytics", async () => {
  const { page, events, errors } = await fixture();
  // Vite does not serve generated Pagefind assets. Serve the existing build's
  // search assets for this test so its HTML fallback is not executed as JS.
  await page.route(`${development}/pagefind/**`, async (route) => {
    const path = new URL(route.request().url()).pathname;
    const file = Bun.file(resolve(root, `build/client${path}`));
    await route.fulfill({
      body: Buffer.from(await file.arrayBuffer()),
      contentType: file.type,
    });
  });
  const analyticsRequests: string[] = [];
  page.on("request", (request) => {
    if (request.url().includes("umami.astrohacker.com"))
      analyticsRequests.push(request.url());
  });
  await page.goto(development);
  await hydrated(page);
  await page
    .locator("header")
    .getByRole("link", { name: "Docs", exact: true })
    .click();
  await page.waitForURL(`${development}/docs/`);
  await settle();
  expect(await page.locator("script[data-umami-tracker]").count()).toBe(0);
  expect(analyticsRequests).toEqual([]);
  expect(events).toEqual([]);
  expect(errors).toEqual([]);
}, 30_000);

// Explicit operator opt-in: this test sends three identified real pageviews.
test.skipIf(process.env.NUTORCH_TEST_ANALYTICS_LIVE !== "1")(
  "published site sends the intended NuTorch pageviews to Umami",
  async () => {
    if (!browser) throw new Error("Browser setup did not complete");
    const context = await browser.newContext();
    contexts.push(context);
    const page = await context.newPage();
    page.setDefaultTimeout(10_000);
    const events: Event[] = [];
    const errors: string[] = [];
    const receipts: Promise<{ status: number; disabled: boolean }>[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    page.on("response", (response) => {
      if (
        response.url() !== "https://umami.astrohacker.com/api/send" ||
        response.request().method() !== "POST"
      )
        return;
      events.push(response.request().postDataJSON() as Event);
      receipts.push(
        (async (): Promise<{ status: number; disabled: boolean }> => {
          const data = (await response.json()) as { disabled?: boolean };
          return {
            status: response.status(),
            disabled: data.disabled === true,
          };
        })(),
      );
    });
    const marker = `26091411157202-exp1-${String(Date.now())}`;
    const home = `/?analytics-check=${marker}`;
    const direct = `/docs/getting-started/?analytics-check=${marker}`;
    await page.goto(`https://nutorch.com${home}`);
    await hydrated(page);
    const tracker = page.locator("script[data-umami-tracker]");
    expect(await tracker.count()).toBe(1);
    expect(await tracker.getAttribute("src")).toBe(UMAMI_SCRIPT_URL);
    expect(await tracker.getAttribute("data-website-id")).toBe(
      UMAMI_WEBSITE_ID,
    );
    expect(await tracker.getAttribute("data-auto-pageview")).toBe("false");
    await settle();
    await page
      .locator("header")
      .getByRole("link", { name: "Docs", exact: true })
      .click();
    await page.waitForURL("https://nutorch.com/docs/");
    await settle();
    await page.goto(`https://nutorch.com${direct}`);
    await hydrated(page);
    await settle();
    expect(urls(events)).toEqual([home, "/docs/", direct]);
    const responses = await Promise.all(receipts);
    expect(responses).toEqual([
      { status: 200, disabled: false },
      { status: 200, disabled: false },
      { status: 200, disabled: false },
    ]);
    expect(errors).toEqual([]);
    console.log("Published analytics evidence", {
      marker,
      checked: new Date().toISOString(),
      website: UMAMI_WEBSITE_ID,
      paths: urls(events),
      responses,
    });
  },
  30_000,
);
