// Content honesty checks (issue 0012 exp 2):
// 1. The getting-started install block byte-matches src/lib/install.ts.
// 2. Every `torch <op>` used in docs fences is a real table op or a known
//    client/registry verb, per `torch ops --json` from the real binary.
import { execSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { INSTALL } from "../src/lib/install";

const DOCS = new URL("../src/content/docs/", import.meta.url).pathname;
let failed = false;

// 1. Install block.
const gettingStarted = readFileSync(`${DOCS}/getting-started.md`, "utf8");
const fence = gettingStarted.match(/```bash\n([\s\S]*?)\n```/);
if (!fence || fence[1] !== INSTALL) {
  console.error("FAIL: getting-started install block drifted from install.ts");
  failed = true;
}

// 2. Op-name membership. Non-op verbs are the client/registry surface,
// verified live in the experiment's verification.
const NON_OP_VERBS = new Set([
  "tensor",
  "value",
  "free",
  "tensors",
  "forward",
  "step",
  "daemon",
  "nn",
  "ops",
  "nu-module",
  "--version",
]);
const ops = new Set(
  (
    JSON.parse(
      execSync("torch ops --json", {
        env: { ...process.env, TMPDIR: execSync("mktemp -d").toString().trim() },
      }).toString(),
    ) as { name: string }[]
  ).map((o) => o.name),
);
execSync("torch daemon stop || true", { stdio: "ignore", shell: "/bin/zsh" });

for (const file of readdirSync(DOCS).filter((f) => f.endsWith(".md"))) {
  const text = readFileSync(`${DOCS}/${file}`, "utf8");
  for (const block of text.matchAll(/```(?:bash|nu)\n([\s\S]*?)\n```/g)) {
    for (const use of block[1].matchAll(/(?:torch|nutorch) ([a-z_-]+|--version)/g)) {
      const verb = use[1];
      if (!ops.has(verb) && !NON_OP_VERBS.has(verb)) {
        console.error(`FAIL: ${file}: unknown verb 'torch ${verb}'`);
        failed = true;
      }
    }
  }
}

if (failed) process.exit(1);
console.log("content checks passed");
