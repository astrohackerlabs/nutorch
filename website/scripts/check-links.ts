// Link integrity gate (issue 0012 exp 4): every internal href in built
// HTML must resolve to a built file; anchors must exist as ids on the
// target page. External links are listed, never fetched (local-only).
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import assert from "node:assert/strict";

const DIST = new URL("../build/client/", import.meta.url).pathname;
let failed = false;
const external = new Set<string>();

function htmlFiles(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) out.push(...htmlFiles(path));
    else if (name.endsWith(".html")) out.push(path);
  }
  return out;
}

function targetFile(route: string): string | undefined {
  for (const candidate of [
    join(DIST, route),
    join(DIST, route, "index.html"),
    join(DIST, `${route.replace(/\/$/, "")}.html`),
  ]) {
    if (existsSync(candidate) && statSync(candidate).isFile()) {
      return candidate;
    }
  }
  return undefined;
}

const files = htmlFiles(DIST);
// Frozen accepted route/metadata oracle; never compare a build with itself.
import baseline from "./accepted-site.json";
function decode(value: string): string {
  return value
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&#(?:39|x27);/g, "'")
    .replace(/&#x([0-9a-f]+);/gi, (_: string, n: string) =>
      String.fromCodePoint(parseInt(n, 16)),
    );
}
function metadata(html: string): Record<string, string> {
  const result: Record<string, string> = {
    title: decode(/<title>(.*?)<\/title>/s.exec(html)?.[1] ?? ""),
  };
  for (const tag of html.matchAll(/<(?:meta|link)\b[^>]*>/g)) {
    const attrs = Object.fromEntries(
      [...tag[0].matchAll(/([\w:-]+)="([^"]*)"/g)].map((m) => {
        assert(
          m[1] !== undefined && m[2] !== undefined,
          "Missing attribute capture",
        );
        return [m[1], decode(m[2])] as const;
      }),
    );
    const key =
      attrs.name ??
      attrs.property ??
      (attrs.rel === "canonical" ? "canonical" : undefined);
    if (
      key &&
      (key === "description" ||
        key === "canonical" ||
        key.startsWith("og:") ||
        key.startsWith("twitter:"))
    ) {
      const value = attrs.content ?? attrs.href;
      assert(value !== undefined, `Missing metadata value for ${key}`);
      result[key] = value;
    }
  }
  return result;
}
const expectedRoutes = Object.keys(baseline.routes).sort();
if (
  JSON.stringify(files.map((file) => file.slice(DIST.length)).sort()) !==
  JSON.stringify(expectedRoutes)
) {
  console.error("FAIL: accepted HTML route set differs");
  failed = true;
}
for (const [relative, expected] of Object.entries(baseline.routes)) {
  const candidate = join(DIST, relative);
  if (!existsSync(candidate)) {
    console.error(`FAIL: missing baseline route ${relative}`);
    failed = true;
    continue;
  }
  const a = metadata(expected);
  const b = metadata(readFileSync(candidate, "utf8"));
  for (const [key, value] of Object.entries(a))
    if (b[key] !== value) {
      console.error(
        `FAIL: ${relative} ${key}: ${JSON.stringify(b[key])} != ${JSON.stringify(value)}`,
      );
      failed = true;
    }
}
for (const file of files) {
  const html = readFileSync(file, "utf8");
  for (const pre of html.matchAll(/<pre\b[^>]*>/g)) {
    if (!/class="[^"]*\bastro-code\b/.test(pre[0])) {
      console.error(`FAIL: unstyled code block in ${file}`);
      failed = true;
    }
  }
  for (const asset of html.matchAll(
    /(?:src|href)="(\/(?:assets|images|pagefind)\/[^"#?]+)"/g,
  )) {
    assert(asset[1] !== undefined, "Missing asset capture");
    if (!targetFile(asset[1])) {
      console.error(`FAIL: missing asset ${asset[1]}`);
      failed = true;
    }
  }
  for (const match of html.matchAll(/href="([^"]+)"/g)) {
    const href = match[1];
    assert(href !== undefined, "Missing href capture");
    if (/^(https?:|mailto:)/.test(href)) {
      external.add(href);
      continue;
    }
    if (href.startsWith("#")) {
      if (!html.includes(`id="${href.slice(1)}"`)) {
        console.error(
          `FAIL: ${file.replace(DIST, "")}: missing anchor ${href}`,
        );
        failed = true;
      }
      continue;
    }
    const [route, anchor] = href.split("#");
    assert(route !== undefined, "Missing href path");
    const target = targetFile(route);
    if (!target) {
      console.error(`FAIL: ${file.replace(DIST, "")}: dead link ${href}`);
      failed = true;
      continue;
    }
    if (anchor && !readFileSync(target, "utf8").includes(`id="${anchor}"`)) {
      console.error(`FAIL: ${file.replace(DIST, "")}: missing anchor ${href}`);
      failed = true;
    }
  }
}

if (failed) process.exit(1);
console.log(
  `links ok: ${String(files.length)} pages checked, ${String(external.size)} external links (not fetched)`,
);
