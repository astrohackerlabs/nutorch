// Content honesty checks (issue 0012 exp 2):
// 1. Homepage and onboarding describe the released Homebrew shell.
// 2. Every `torch <op>` used in docs fences is a real table op or a known
//    native command, per metadata from the installed release shell.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import assert from "node:assert/strict";

const DOCS = new URL("../content/docs/", import.meta.url).pathname;
let failed = false;

// 1. The public onboarding uses the released installation path.
const gettingStarted = readFileSync(`${DOCS}/getting-started.md`, "utf8");
if (
  !gettingStarted.includes(
    "brew install astrohackerlabs/astrohacker/nutorch",
  ) ||
  !gettingStarted.includes("```nu\nnutorch\n```") ||
  /unreleased|earlier tensor-tool release/.test(gettingStarted)
) {
  console.error(
    "FAIL: getting-started must explain Homebrew installation and installed shell launch",
  );
  failed = true;
}

// 2. Op-name membership. Non-op verbs are the native command surface,
// verified live in the experiment's verification.
const NON_OP_VERBS = new Set([
  "tensor",
  "value",
  "shape",
  "tolist",
  "forward",
  "step",
  "nn",
  "ops",
  "--version",
]);
const ops = new Set(
  (
    JSON.parse(
      execFileSync("nutorch", [
        "--no-config-file",
        "--no-history",
        "-c",
        "use torch; torch ops | to json --raw",
      ]).toString(),
    ) as { name: string }[]
  ).map((o) => o.name),
);
// Metadata inspection creates no tensors and starts no daemon.

// Recursive walk (issue 0017 exp 3 — the reference subdir joins the scan),
// keyed by docs-root-relative path (autograd.md exists at two levels).
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
  if (/```(?:bash|sh|zsh|posix)\b/.test(text)) {
    console.error(`FAIL: ${file}: obsolete shell example fence`);
    failed = true;
  }
  for (const block of text.matchAll(/```nu\n([\s\S]*?)\n```/g)) {
    assert(block[1] !== undefined, "Missing fenced code capture");
    if (/\btorch [a-z]/.test(block[1]) && !block[1].includes("use torch")) {
      console.error(`FAIL: ${file}: native example is missing use torch`);
      failed = true;
    }
    for (const use of block[1].matchAll(
      /(?<![\w/.-])(?:torch|nutorch) ([a-z][a-z0-9_-]*|--version)(?![\w/.-])/g,
    )) {
      const verb = use[1];
      assert(verb !== undefined, "Missing command verb capture");
      if (!ops.has(verb) && !NON_OP_VERBS.has(verb)) {
        console.error(`FAIL: ${file}: unknown verb 'torch ${verb}'`);
        failed = true;
      }
    }
  }
}

// The landing page's demo code lives in TSX template literals, not
// markdown fences — scan only the backtick strings (prose like the logo
// alt text would otherwise false-positive).
const INDEX = new URL("../app/routes/home.tsx", import.meta.url).pathname;
const indexSource = readFileSync(INDEX, "utf8");
for (const command of [
  "brew tap astrohackerlabs/astrohacker",
  "brew trust astrohackerlabs/astrohacker",
  "brew install astrohackerlabs/astrohacker/nutorch",
]) {
  if (!indexSource.includes(command) || !gettingStarted.includes(command)) {
    console.error(`FAIL: homepage/onboarding missing ${command}`);
    failed = true;
  }
}
if (
  /unreleased|earlier tensor-tool|target\/release\/nutorch|Try the development shell/i.test(
    indexSource,
  )
) {
  console.error(
    "FAIL: homepage still requires an unreleased development build",
  );
  failed = true;
}
const nushellSource = readFileSync(`${DOCS}/nushell.md`, "utf8");
const requiredSetup = [
  "## Setup",
  "```nu\nnutorch\n```",
  "Run `use torch` inside NuTorch",
  "It is not imported by default.",
  "Do not import `nutorch.nu`",
  "[[7.0 10.0] [15.0 22.0]]",
  "Plain Nushell does not gain tensor commands",
];
for (const snippet of requiredSetup) {
  if (!nushellSource.includes(snippet)) {
    console.error(`FAIL: Nushell setup missing ${snippet}`);
    failed = true;
  }
}
for (const [name, source] of [
  [
    "landing",
    readFileSync(
      new URL("../build/client/index.html", import.meta.url),
      "utf8",
    ),
  ],
  ["getting-started", gettingStarted],
] as const) {
  if (!source.includes("/docs/nushell/#setup")) {
    console.error(`FAIL: ${name} lacks Nushell setup link`);
    failed = true;
  }
}
const activeSources = [
  indexSource,
  ...docsMdFiles(DOCS).map((file) => readFileSync(`${DOCS}/${file}`, "utf8")),
  ...["Header", "Footer"].map((name) =>
    readFileSync(
      new URL(`../app/components/${name}.tsx`, import.meta.url),
      "utf8",
    ),
  ),
];
for (const source of activeSources) {
  if (
    /nutorch -c|No tensor client import is needed|lang="(?:bash|sh|zsh)"|\[\w+, "bash"\]/.test(
      source,
    )
  ) {
    console.error("FAIL: obsolete example presentation");
    failed = true;
  }
  if (
    /github\.com\/nutorch\/(?:nutorch|homebrew-nutorch)|brew (?:tap|trust) nutorch\/nutorch|prebuilt\s+bottle|nothing to set up/.test(
      source,
    )
  ) {
    console.error("FAIL: obsolete installation claim or repository link");
    failed = true;
  }
}
for (const literal of indexSource.matchAll(/`([\s\S]*?)`/g)) {
  assert(literal[1] !== undefined, "Missing template literal capture");
  if (/\btorch [a-z]/.test(literal[1]) && !literal[1].includes("use torch")) {
    console.error("FAIL: homepage example missing use torch");
    failed = true;
  }
  for (const use of literal[1].matchAll(
    /(?<![\w/.-])(?:torch|nutorch) ([a-z][a-z0-9_-]*|--version)(?![\w/.-])/g,
  )) {
    const verb = use[1];
    assert(verb !== undefined, "Missing command verb capture");
    if (!ops.has(verb) && !NON_OP_VERBS.has(verb)) {
      console.error(`FAIL: home.tsx: unknown verb 'torch ${verb}'`);
      failed = true;
    }
  }
}

// 3. The brand gate (issue 0013 exp 6): in RENDERED prose, the name is
// NuTorch. Lowercase `nutorch` is code — it may appear only inside
// code/pre/script elements, attribute values, or URLs, all of which the
// strip below removes. Runs only when a build exists.
const DIST = new URL("../build/client/", import.meta.url).pathname;
function distHtmlFiles(dir: string): string[] {
  const out: string[] = [];
  let entries: string[] = [];
  try {
    entries = readdirSync(dir);
  } catch {
    return out;
  }
  for (const name of entries) {
    const path = `${dir}/${name}`;
    if (statSync(path).isDirectory()) out.push(...distHtmlFiles(path));
    else if (name.endsWith(".html")) out.push(path);
  }
  return out;
}
for (const file of distHtmlFiles(DIST)) {
  let html = readFileSync(file, "utf8");
  html = html.replace(/<(code|pre|script|style)[\s\S]*?<\/\1>/g, " ");
  html = html.replace(/<[^>]*>/g, " "); // tags incl. all attribute values
  // URL/path-shaped tokens (domains, repo paths) are identifiers, not
  // prose: nutorch.com, github.com/nutorch/nutorch, ~/.nutorch, …
  html = html.replace(/\S*nutorch[./]\S*/g, " ");
  html = html.replace(/\S*[./~]nutorch\S*/g, " ");
  for (const m of html.matchAll(/.{0,30}\bnutorch\b.{0,30}/g)) {
    console.error(
      `FAIL: ${file.replace(DIST, "")}: prose lowercase brand: …${m[0].trim()}…`,
    );
    failed = true;
  }
}

if (failed) process.exit(1);
console.log("content checks passed");
