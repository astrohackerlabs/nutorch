// Mirror gate (issue 0017 exp 2): every bash+nu pair — docs fences and the
// hero's template literals — must have EQUAL command-line counts (blank and
// comment-only lines excluded), modulo the documented exceptions below for
// shell-forced divergences. Forms alignment is the by-eye criterion; this
// gate stops silent structural drift.
import { readFileSync, readdirSync, statSync } from "node:fs";

const DOCS = new URL("../src/content/docs/", import.meta.url).pathname;
const INDEX = new URL("../src/pages/index.astro", import.meta.url).pathname;

// Shell-forced extra nu lines, documented per pair (file:pairIndex).
// neural-networks pair 1 (0-based; the training loop): nu needs a
// `mut loss` declaration line that bash has no peer for.
const EXCEPTIONS: Record<string, number> = {
  "neural-networks.md:1": 1,
};

let failed = false;

function commandLines(block: string): number {
  return block
    .split("\n")
    .filter((line) => {
      const t = line.trim();
      return t !== "" && !t.startsWith("#");
    }).length;
}

function checkPair(id: string, bash: string, nu: string) {
  const allowed = EXCEPTIONS[id] ?? 0;
  const b = commandLines(bash);
  const n = commandLines(nu);
  const ok = n === b + allowed;
  console.log(
    `${ok ? "ok  " : "FAIL"} ${id}: bash=${b} nu=${n}${allowed ? ` (+${allowed} exempt)` : ""}`,
  );
  if (!ok) failed = true;
}

// Docs pairs: a bash fence immediately followed (blank lines allowed
// between) by a nu fence — the same adjacency the rehype plugin uses.
// Recursive walk keyed by docs-root-relative path (issue 0017 exp 3 —
// the generated reference pairs join the gate, and autograd.md exists
// at two levels).
function docsMdFiles(dir: string, prefix = ""): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const path = `${dir}/${name}`;
    if (statSync(path).isDirectory()) {
      out.push(...docsMdFiles(path, `${prefix}${name}/`));
    } else if (name.endsWith(".md")) out.push(`${prefix}${name}`);
  }
  return out;
}

for (const file of docsMdFiles(DOCS)) {
  const text = readFileSync(`${DOCS}/${file}`, "utf8");
  const fences = [
    ...text.matchAll(/```(bash|nu)\n([\s\S]*?)\n```/g),
  ].map((m) => ({ lang: m[1], body: m[2], end: m.index! + m[0].length }));
  let pairIndex = 0;
  for (let i = 0; i < fences.length - 1; i++) {
    if (fences[i].lang === "bash" && fences[i + 1].lang === "nu") {
      const between = text.slice(
        fences[i].end,
        text.indexOf("```", fences[i].end),
      );
      if (between.trim() === "") {
        checkPair(`${file}:${pairIndex}`, fences[i].body, fences[i + 1].body);
        pairIndex++;
        i++;
      }
    }
  }
}

// The hero pair.
const astro = readFileSync(INDEX, "utf8");
const hero = astro.match(/const heroDemo = `([\s\S]*?)`;/)?.[1];
const heroNu = astro.match(/const heroDemoNu = `([\s\S]*?)`;/)?.[1];
if (!hero || !heroNu) {
  console.error("FAIL: hero demo literals not found");
  failed = true;
} else {
  checkPair("index.astro:hero", hero, heroNu);
}

if (failed) process.exit(1);
console.log("mirror gate passed");
