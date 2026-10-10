import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

/** Catalog type to the one existing docs slug that shows its tips. */
export const tipPages = {
  "thinking-in-nu": "shell/book/thinking_in_nu",
  pipelines: "shell/book/pipelines",
  types: "shell/book/types_of_data",
  strings: "shell/book/working_with_strings",
  lists: "shell/book/working_with_lists",
  records: "shell/book/working_with_records",
  tables: "shell/book/working_with_tables",
  "navigating-data": "shell/book/navigating_structured_data",
  variables: "shell/book/variables",
  operators: "shell/book/operators",
  "control-flow": "shell/book/control_flow",
  "custom-commands": "shell/book/custom_commands",
  modules: "shell/book/modules",
  scripts: "shell/book/scripts",
  environment: "shell/book/environment",
  configuration: "shell/book/configuration",
  "moving-around": "shell/book/moving_around",
  "loading-data": "shell/book/loading_data",
  regex: "shell/book/regular_expressions",
  jobs: "shell/book/background_jobs",
  aliases: "shell/book/aliases",
  externs: "shell/book/externs",
  "line-editor": "shell/book/line_editor",
  parallelism: "shell/book/parallelism",
  metadata: "shell/book/metadata",
  filters: "shell/commands/categories/filters",
  math: "shell/commands/categories/math",
  path: "shell/commands/categories/path",
  date: "shell/commands/categories/date",
  conversions: "shell/commands/categories/conversions",
  random: "shell/commands/categories/random",
  bits: "shell/commands/categories/bits",
  bytes: "shell/commands/categories/bytes",
  system: "shell/commands/categories/system",
  history: "shell/commands/categories/history",
  hash: "shell/commands/categories/hash",
  generators: "shell/commands/categories/generators",
  network: "shell/commands/categories/network",
  tensors: "tensors",
  "neural-networks": "neural-networks",
  autograd: "reference/autograd",
  creation: "reference/creation",
  pointwise: "reference/pointwise",
  reduction: "reference/reduction",
  shape: "reference/shape",
  linalg: "reference/linalg",
  loss: "reference/loss",
  comparison: "reference/comparison",
  utility: "reference/utility",
} as const;

const levels = ["intro", "basic", "intermediate", "advanced"] as const;
const levelTitle = {
  intro: "Intro",
  basic: "Basic",
  intermediate: "Intermediate",
  advanced: "Advanced",
} as const;

interface Tip {
  kind: string;
  type: string;
  level: (typeof levels)[number];
  body: string;
}

const catalogPath = fileURLToPath(
  new URL("../../../../rs/shell/tips/catalog.json", import.meta.url),
);

function isLevel(value: string): value is Tip["level"] {
  return levels.some((level) => level === value);
}

function readCatalog(): Tip[] {
  const parsed: unknown = JSON.parse(readFileSync(catalogPath, "utf8"));
  if (!Array.isArray(parsed)) throw new Error("Tip catalog is not a list");
  return parsed.map((row: unknown) => {
    if (typeof row !== "object" || row === null) {
      throw new Error("Tip catalog row is not a record");
    }
    const tip = row as Record<string, unknown>;
    if (
      typeof tip.kind !== "string" ||
      typeof tip.type !== "string" ||
      typeof tip.level !== "string" ||
      typeof tip.body !== "string" ||
      !isLevel(tip.level)
    ) {
      throw new Error("Tip catalog row is missing kind, type, level, or body");
    }
    if (!(tip.type in tipPages)) {
      throw new Error(`Tip type has no page: ${tip.type}`);
    }
    return {
      kind: tip.kind,
      type: tip.type,
      level: tip.level,
      body: tip.body,
    };
  });
}

function groupCatalog(tips: Tip[]): Map<string, Tip[]> {
  const grouped = new Map<string, Tip[]>();
  for (const tip of tips) {
    const rows = grouped.get(tip.type) ?? [];
    rows.push(tip);
    grouped.set(tip.type, rows);
  }
  for (const type of Object.keys(tipPages)) {
    const rows = grouped.get(type);
    if (rows === undefined || rows.length === 0) {
      throw new Error(`Tip page has no tips: ${type}`);
    }
  }
  const slugs = Object.values(tipPages);
  if (new Set(slugs).size !== slugs.length) {
    throw new Error("Each tip type needs its own page");
  }
  return grouped;
}

const tipsByType = groupCatalog(readCatalog());
const typeBySlug = new Map<string, string>(
  Object.entries(tipPages).map(([type, slug]) => [slug, type]),
);

function tipBlock(body: string): string {
  const [first, ...rest] = body.split("\n");
  const caption = (first ?? "").replace(/^# /, "");
  // A raw HTML paragraph keeps the lesson characters. A markdown paragraph
  // would turn apostrophes into typographic quotes.
  const paragraph = `<p>${caption}</p>`;
  const code = rest.join("\n").replace(/\n+$/, "");
  if (code.length === 0) return paragraph;
  return `${paragraph}\n\n\`\`\`nu\n${code}\n\`\`\``;
}

function tipsMarkdown(tips: Tip[]): string {
  const parts = ["## Tips"];
  for (const level of levels) {
    const rows = tips.filter((tip) => tip.level === level);
    if (rows.length === 0) continue;
    parts.push(`### ${levelTitle[level]}`);
    for (const tip of rows) parts.push(tipBlock(tip.body));
  }
  return parts.join("\n\n");
}

/** Append the catalog section for a mapped slug. Other pages stay as written. */
export function appendTips(slug: string, content: string): string {
  const type = typeBySlug.get(slug);
  if (type === undefined) return content;
  const tips = tipsByType.get(type);
  if (tips === undefined) return content;
  return `${content.replace(/\s*$/, "")}\n\n${tipsMarkdown(tips)}\n`;
}
