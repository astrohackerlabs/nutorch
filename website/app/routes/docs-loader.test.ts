import { expect, test } from "bun:test";
import { RouterContextProvider } from "react-router";
import { loader } from "./docs";

test("docs default and normalized slugs retain content and indexability", async () => {
  for (const wildcard of [
    undefined,
    "",
    "getting-started",
    "getting-started/",
  ]) {
    const data = await loader({
      params: wildcard === undefined ? {} : { "*": wildcard },
      request: new Request("http://localhost/docs"),
      url: new URL("http://localhost/docs"),
      pattern: "/docs/*",
      context: new RouterContextProvider(),
    });
    expect(data.entry.slug).toBe("getting-started");
    expect(data.html.length).toBeGreaterThan(0);
    expect(data.indexable).toBe(Boolean(wildcard));
  }
});

test("unknown docs slugs return 404", async () => {
  try {
    await loader({
      params: { "*": "not-a-real-document" },
      request: new Request("http://localhost/docs/not-a-real-document"),
      url: new URL("http://localhost/docs/not-a-real-document"),
      pattern: "/docs/*",
      context: new RouterContextProvider(),
    });
    throw new Error("Expected missing document response");
  } catch (error) {
    expect(error).toBeInstanceOf(Response);
    if (!(error instanceof Response)) throw error;
    expect(error.status).toBe(404);
  }
});
