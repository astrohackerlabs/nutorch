import { describe, expect, test } from "bun:test";
import { createPageviews } from "./umami";
import type { Pageview, UmamiTracker } from "./umami";

const page = (url: string): Pageview => ({
  url,
  title: `Title ${url}`,
  referrer: "https://example.com/",
});

function recorder(): { events: Record<string, unknown>[]; api: UmamiTracker } {
  const events: Record<string, unknown>[] = [];
  return {
    events,
    api: {
      track(payload): void {
        events.push(payload({ website: "test", url: "stale", title: "stale" }));
      },
    },
  };
}

describe("router-owned pageviews", () => {
  test("deduplicates replay, but counts returning visits and query changes", () => {
    const views = createPageviews();
    const { api, events } = recorder();
    views.ready(api);
    for (const url of ["/", "/", "/docs/", "/docs/?q=1", "/docs/", "/"]) {
      views.visit(page(url));
    }
    expect(events.map((event) => event.url)).toEqual([
      "/",
      "/docs/",
      "/docs/?q=1",
      "/docs/",
      "/",
    ]);
    expect(events[0]?.referrer).toBe("https://example.com/");
    expect(events[2]?.referrer).toBe("/docs/");
  });

  test("delayed readiness preserves captured payloads and flushes only once", () => {
    const views = createPageviews();
    const { api, events } = recorder();
    views.visit(page("/"));
    views.visit(page("/docs/"));
    views.ready(api);
    views.ready(api);
    expect(events).toEqual([
      { website: "test", ...page("/") },
      { website: "test", ...page("/docs/"), referrer: "/" },
    ]);
  });

  test("load failure discards the queue and ignores later readiness", () => {
    const views = createPageviews();
    const { api, events } = recorder();
    views.visit(page("/"));
    views.fail();
    views.ready(api);
    views.visit(page("/docs/"));
    expect(events).toEqual([]);
  });

  test("throwing or rejecting trackers cannot escape into the application", async () => {
    for (const reject of [false, true]) {
      const views = createPageviews();
      let calls = 0;
      views.ready({
        track(): unknown {
          calls++;
          if (reject) return Promise.reject(new Error("offline"));
          throw new Error("offline");
        },
      });
      expect(() => {
        views.visit(page("/"));
      }).not.toThrow();
      await Promise.resolve();
      views.visit(page("/docs/"));
      expect(calls).toBe(1);
    }
  });

  test("a never-ready tracker cannot accumulate an unbounded queue", () => {
    const views = createPageviews();
    const { api, events } = recorder();
    for (let i = 0; i < 130; i++) views.visit(page(`/docs/?q=${String(i)}`));
    views.ready(api);
    expect(events).toEqual([]);
  });
});
