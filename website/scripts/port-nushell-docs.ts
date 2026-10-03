// Port the English Nushell user docs into content/docs/shell.
// Reads the uncommitted clone at vendor/nushell.github.io. Does not commit it.
import {
  cpSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import GithubSlugger from "../../../../../node_modules/.bun/github-slugger@2.0.0/node_modules/github-slugger/index.js";

function splitFrontmatter(raw: string): {
  data: Record<string, string>;
  body: string;
} {
  if (!raw.startsWith("---\n")) return { data: {}, body: raw };
  const end = raw.indexOf("\n---\n", 4);
  if (end === -1) return { data: {}, body: raw };
  const data: Record<string, string> = {};
  let key = "";
  let block = false;
  for (const line of raw.slice(4, end).split("\n")) {
    if (block) {
      if (!/^\S/.test(line)) {
        data[key] = `${data[key] ?? ""}\n${line.trim()}`;
        continue;
      }
      block = false;
    }
    const match = /^([A-Za-z0-9_]+):\s*(.*)$/.exec(line);
    if (!match?.[1]) continue;
    key = match[1];
    const value = match[2] ?? "";
    if (value === "|" || value === ">") {
      block = true;
      data[key] = "";
    } else data[key] = value.replace(/^["']|["']$/g, "");
  }
  return { data, body: raw.slice(end + 5) };
}

function dumpPage(
  body: string,
  data: { title: string; description: string; order: number; section: string },
): string {
  return `---\ntitle: ${JSON.stringify(data.title)}\ndescription: ${JSON.stringify(data.description)}\norder: ${String(data.order)}\nsection: ${JSON.stringify(data.section)}\n---\n\n${body.startsWith("\n") ? body.slice(1) : body}`;
}

const HERE = dirname(fileURLToPath(import.meta.url));
const SITE = resolve(HERE, "..");
const REPO = resolve(SITE, "../../../..");
const VENDOR = join(REPO, "vendor/nushell.github.io");
const OUT = join(SITE, "content/docs/shell");
const IMAGES = join(SITE, "public/images/nushell");
const NOTICE =
  "\n\nPorted from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.\n";

const BOOK_CHAPTERS: [string, string[]][] = [
  ["Book · Introduction", ["README.md", "table_of_contents.md"]],
  ["Book · Installation", ["installation.md", "default_shell.md"]],
  [
    "Book · Getting Started",
    [
      "getting_started.md",
      "quick_tour.md",
      "moving_around.md",
      "thinking_in_nu.md",
      "cheat_sheet.md",
    ],
  ],
  [
    "Book · Nu Fundamentals",
    [
      "nu_fundamentals.md",
      "types_of_data.md",
      "loading_data.md",
      "pipelines.md",
      "working_with_strings.md",
      "working_with_lists.md",
      "working_with_records.md",
      "working_with_tables.md",
      "navigating_structured_data.md",
      "special_variables.md",
    ],
  ],
  [
    "Book · Programming in Nu",
    [
      "programming_in_nu.md",
      "custom_commands.md",
      "aliases.md",
      "operators.md",
      "variables.md",
      "control_flow.md",
      "scripts.md",
      "modules.md",
      "modules/using_modules.md",
      "modules/creating_modules.md",
      "overlays.md",
      "sorting.md",
      "testing.md",
      "style_guide.md",
      "regular_expressions.md",
    ],
  ],
  [
    "Book · Nu as a Shell",
    [
      "nu_as_a_shell.md",
      "configuration.md",
      "environment.md",
      "stdout_stderr_exit_codes.md",
      "running_externals.md",
      "3rdpartyprompts.md",
      "directory_stack.md",
      "line_editor.md",
      "custom_completions.md",
      "externs.md",
      "coloring_and_theming.md",
      "hooks.md",
      "background_jobs.md",
    ],
  ],
  [
    "Book · Coming to Nu",
    [
      "coming_to_nu.md",
      "coming_from_bash.md",
      "coming_from_cmd.md",
      "coming_from_powershell.md",
      "nushell_map.md",
      "nushell_map_imperative.md",
      "nushell_map_functional.md",
      "nushell_operator_map.md",
    ],
  ],
  ["Book · Design Notes", ["design_notes.md", "how_nushell_code_gets_run.md"]],
  [
    "Book · (Not So) Advanced",
    [
      "advanced.md",
      "standard_library.md",
      "dataframes.md",
      "metadata.md",
      "creating_errors.md",
      "parallelism.md",
      "plugins.md",
      "explore.md",
    ],
  ],
];

interface Page {
  rel: string;
  slug: string;
  section: string;
  title: string;
  description: string;
  body: string;
  category: string;
}

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) out.push(...walk(path));
    else if (name.endsWith(".md")) out.push(path);
  }
  return out;
}

function relPosix(path: string): string {
  return relative(VENDOR, path).split("\\").join("/");
}

function slugFor(rel: string): string {
  const bare = rel
    .replace(/^\//, "")
    .replace(/\.html$/, "")
    .replace(/\.md$/, "");
  const trimmed = bare.endsWith("/") ? bare.slice(0, -1) : bare;
  if (trimmed === "book" || trimmed === "book/README")
    return "shell/book/README";
  if (trimmed.startsWith("book/")) return `shell/${trimmed}`;
  if (trimmed === "lang-guide" || trimmed === "lang-guide/README")
    return "shell/lang-guide/README";
  if (trimmed.startsWith("lang-guide/")) return `shell/${trimmed}`;
  if (trimmed === "cookbook" || trimmed === "cookbook/README")
    return "shell/cookbook/README";
  if (trimmed.startsWith("cookbook/")) return `shell/${trimmed}`;
  if (trimmed === "commands" || trimmed === "commands/README")
    return "shell/commands/readme";
  if (trimmed.startsWith("commands/docs/"))
    return `shell/commands/${trimmed.slice("commands/docs/".length)}`;
  if (trimmed.startsWith("commands/categories/"))
    return `shell/commands/categories/${trimmed.slice("commands/categories/".length)}`;
  return "";
}

function bookSection(rel: string): string {
  const local = rel.slice("book/".length);
  for (const [section, files] of BOOK_CHAPTERS) {
    if (files.includes(local)) return section;
  }
  return "Book · Introduction";
}

function categoriesOf(value: unknown): string[] {
  if (typeof value !== "string") return [];
  return value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

function plainText(value: string): string {
  return value
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/[`*_]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function plainTitle(body: string, fallback: string): string {
  const heading = /^#\s+(.+)$/m.exec(body);
  const text = plainText(heading?.[1] ?? fallback);
  return text.length > 0 ? text : fallback;
}

const commandTitle = /^# `([^`]+)` for (\[[^\]]+\]\([^)]+\))$/;

function dropPageTitle(
  body: string,
  title: string,
): { body: string; title: string } {
  const lines = body.split("\n");
  let open = 0;
  let quote = false;
  for (let index = 0; index < lines.length; index += 1) {
    const raw = lines[index] ?? "";
    if (quote && /^\s*:::\s*$/.test(raw)) {
      quote = false;
      open = 0;
      continue;
    }
    const next = fenceUpdate(raw, open);
    const fenceLine = next.open !== open || /^`{3,}/.test(raw);
    open = next.open;
    if (open > 0 || fenceLine) continue;
    if (!quote && /^\s*:::\s*\S/.test(raw)) {
      quote = true;
      continue;
    }
    const heading = /^#\s+(.+)$/.exec(raw);
    if (!heading?.[1]) continue;
    const command = commandTitle.exec(raw);
    if (command?.[2]) {
      lines[index] = `For ${command[2]}.`;
      return { body: lines.join("\n"), title };
    }
    if (plainText(heading[1]) !== plainText(title)) {
      title = heading[1].trim();
    }
    lines.splice(index, 1);
    return { body: lines.join("\n"), title };
  }
  return { body, title };
}

function oneLine(value: string, fallback: string): string {
  const line = value.replace(/\s+/g, " ").trim();
  const cut = line.length > 180 ? `${line.slice(0, 177)}...` : line;
  return cut.length > 0 ? cut : fallback;
}

function fenceLang(file: string): string {
  const ext = extname(file).slice(1);
  if (ext === "nu") return "nu";
  if (ext === "sh" || ext === "bash") return "sh";
  return "text";
}

function inlineSnippets(body: string): string {
  return body.replace(
    /@\[code\]\(@snippets\/([^)]+)\)/g,
    (_match, spec: string) => {
      const file = join(VENDOR, "snippets", spec);
      const source = readFileSync(file, "utf8").replace(/\n$/, "");
      return `\`\`\`${fenceLang(spec)}\n${source}\n\`\`\``;
    },
  );
}

function fenceUpdate(
  line: string,
  open: number,
): { line: string; open: number } {
  if (open === 0) {
    const start = /^(`{3,})([^`]*)$/.exec(line);
    if (start?.[1]) {
      const info = (start[2] ?? "").replace(/:(?:no-)?line-numbers\b/g, "");
      return { line: `${start[1]}${info}`, open: start[1].length };
    }
    return { line, open };
  }
  if (/^`{3,}$/.test(line) && line.length >= open) return { line, open: 0 };
  return { line, open };
}

function containers(body: string): string {
  const out: string[] = [];
  let open = 0;
  let quote = false;
  for (const raw of body.split("\n")) {
    // VuePress finds a container closer before it tokenizes inner fences,
    // so a bare ::: ends the container and any fence still open inside it.
    if (quote && /^\s*:::\s*$/.test(raw)) {
      if (open > 0) {
        out.push(`> ${"`".repeat(open)}`);
        open = 0;
      }
      quote = false;
      out.push("");
      continue;
    }
    const next = fenceUpdate(raw, open);
    const fenceLine = next.open !== open || /^`{3,}/.test(raw);
    open = next.open;
    if (open > 0 || fenceLine) {
      out.push(quote ? `> ${next.line}` : next.line);
      continue;
    }
    if (!quote && /^\s*:::\s*\S/.test(raw)) {
      out.push(`> **${raw.replace(/^\s*:::\s*/, "").trim()}**`);
      quote = true;
      continue;
    }
    if (raw.trim() === "[[toc]]") continue;
    out.push(quote ? `> ${raw}` : raw);
  }
  return out.join("\n");
}

function headingSlugs(body: string): Map<string, string> {
  const slugs = new GithubSlugger();
  const map = new Map<string, string>();
  let open = 0;
  let quote = false;
  let skippedTitle = false;
  for (const line of body.split("\n")) {
    if (quote && /^\s*:::\s*$/.test(line)) {
      quote = false;
      open = 0;
      continue;
    }
    const next = fenceUpdate(line, open);
    if (next.open !== open || next.line !== line) {
      open = next.open;
      continue;
    }
    open = next.open;
    if (open > 0) continue;
    if (!quote && /^\s*:::\s*\S/.test(line)) {
      quote = true;
      continue;
    }
    const heading = /^(#{1,6})\s+(.+?)\s*$/.exec(line);
    if (!heading?.[1] || !heading[2]) continue;
    if (heading[1] === "#" && !skippedTitle) {
      skippedTitle = true;
      continue;
    }
    const text = heading[2]
      .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
      .replace(/`([^`]+)`/g, "$1")
      .replace(/(^|\s)_([^_\n]+)_(?=\s|$)/g, "$1$2")
      .replace(/(^|\s)\*([^*\n]+)\*(?=\s|$)/g, "$1$2")
      .replace(/<[^>]+>/g, "");
    const id = slugs.slug(text);
    const remember = (key: string): void => {
      if (!map.has(key)) map.set(key, id);
    };
    remember(id);
    remember(id.replace(/[-_]/g, ""));
    remember(new GithubSlugger().slug(heading[2]).replace(/[-_]/g, ""));
  }
  return map;
}

const headingCache = new Map<string, Map<string, string>>();

function headingsFor(rel: string): Map<string, string> {
  const cached = headingCache.get(rel);
  if (cached) return cached;
  const file = join(VENDOR, rel);
  let body =
    rel.endsWith(".md") && statExists(file) ? readFileSync(file, "utf8") : "";
  if (body.startsWith("---\n")) body = splitFrontmatter(body).body;
  const map = headingSlugs(body);
  headingCache.set(rel, map);
  return map;
}

function statExists(path: string): boolean {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}

function dirExists(path: string): boolean {
  try {
    return statSync(path).isDirectory();
  } catch {
    return false;
  }
}

function rewriteHash(fragment: string, targetRel: string): string {
  const map = headingsFor(targetRel);
  const plain = new GithubSlugger().slug(decodeURIComponent(fragment));
  const compact = plain.replace(/[-_]/g, "");
  return map.get(fragment) ?? map.get(plain) ?? map.get(compact) ?? "";
}

function resolveRel(fromRel: string, urlPath: string): string {
  if (urlPath.startsWith("/")) return urlPath.replace(/^\//, "");
  const base = dirname(fromRel);
  const parts = join(base, urlPath).split(/[/\\]/);
  const stack: string[] = [];
  for (const part of parts) {
    if (part === "" || part === ".") continue;
    if (part === "..") stack.pop();
    else stack.push(part);
  }
  return stack.join("/");
}

function copyImage(fromRel: string, urlPath: string): string | undefined {
  const resolved = resolveRel(fromRel, urlPath.split("#")[0] ?? urlPath);
  const source = join(VENDOR, resolved);
  if (!statExists(source)) return undefined;
  const ext = extname(source);
  if (!/\.(png|gif|jpg|jpeg|svg|webp)$/i.test(ext)) return undefined;
  mkdirSync(IMAGES, { recursive: true });
  const name = resolved.replace(/[\\/]/g, "-");
  cpSync(source, join(IMAGES, name));
  return `/images/nushell/${name}`;
}

function sourceExists(rel: string): boolean {
  const bare = rel
    .replace(/\/+$/, "")
    .replace(/\.html$/, "")
    .replace(/\.md$/, "");
  if (
    bare === "commands" ||
    bare === "book" ||
    bare === "cookbook" ||
    bare === "lang-guide"
  ) {
    return statExists(join(VENDOR, `${bare}/README.md`));
  }
  return (
    statExists(join(VENDOR, `${bare}.md`)) || statExists(join(VENDOR, rel))
  );
}

function rewriteTarget(fromRel: string, raw: string): string {
  let trimmed = raw.trim().replace(/^<|>$/g, "");
  if (trimmed.startsWith("[") && trimmed.endsWith("]"))
    trimmed = trimmed.slice(1, -1);
  if (/^(https?:|mailto:)/.test(trimmed)) return trimmed;
  if (trimmed.startsWith("#")) {
    const id = rewriteHash(trimmed.slice(1), fromRel);
    const slug = slugFor(fromRel);
    return id ? `#${id}` : slug ? `/docs/${slug}/` : trimmed;
  }
  const hashAt = trimmed.indexOf("#");
  const pathPart = hashAt === -1 ? trimmed : trimmed.slice(0, hashAt);
  const fragment = hashAt === -1 ? "" : trimmed.slice(hashAt + 1);
  if (pathPart === "") {
    return fragment
      ? `#${rewriteHash(decodeURIComponent(fragment), fromRel)}`
      : trimmed;
  }
  const resolved = resolveRel(fromRel, pathPart);
  const blogAt = resolved.indexOf("blog/");
  if (blogAt !== -1 || resolved.includes("contributor-book")) {
    const sitePath = blogAt !== -1 ? resolved.slice(blogAt) : resolved;
    const hash = fragment ? `#${fragment}` : "";
    return `https://www.nushell.sh/${sitePath}${hash}`;
  }
  const image = copyImage(fromRel, pathPart);
  if (image) return fragment ? `${image}#${fragment}` : image;
  const slug = slugFor(resolved);
  if (!slug || !sourceExists(resolved)) {
    const sitePath = resolved.startsWith("/") ? resolved : `/${resolved}`;
    const hash = fragment ? `#${fragment}` : "";
    return `https://www.nushell.sh${sitePath}${hash}`;
  }
  const targetRel = resolved.endsWith(".md")
    ? resolved
    : resolved.endsWith(".html")
      ? `${resolved.slice(0, -5)}.md`
      : `${resolved}.md`;
  const id = fragment ? rewriteHash(fragment, targetRel) : "";
  return `/docs/${slug}/${id ? `#${id}` : ""}`;
}

function rewriteLinks(body: string, fromRel: string): string {
  const linked = body.replace(
    /(!?\[[^\]]*\]\()([^)\s]+)(\))/g,
    (_all, open: string, url: string, close: string) =>
      `${open}${rewriteTarget(fromRel, url)}${close}`,
  );
  return linked.replace(
    /href="([^"]+)"/g,
    (_all, url: string) => `href="${rewriteTarget(fromRel, url)}"`,
  );
}

function staticCommandList(
  commands: {
    title: string;
    slug: string;
    usage: string;
    categories: string[];
  }[],
  category?: string,
): string {
  const rows = commands
    .filter((command) =>
      category ? command.categories.includes(category) : true,
    )
    .sort((a, b) => a.title.localeCompare(b.title));
  if (rows.length === 0) return "";
  return rows
    .map(
      (command) =>
        `- [\`${command.title}\`](/docs/${command.slug}/) — ${oneLine(command.usage, command.title)}`,
    )
    .join("\n");
}

function replaceVueTable(body: string, list: string): string {
  const withoutScript = body.replace(/<script[\s\S]*?<\/script>/g, "");
  return withoutScript.replace(/<table[\s\S]*?<\/table>/g, list);
}

function shellInvocations(body: string): string {
  return body
    .replaceAll("`nu`/`nu.exe`", "`nutorch`/`nutorch.exe`")
    .replaceAll("`nu.exe`", "`nutorch.exe`")
    .replaceAll("`nu`", "`nutorch`")
    .replace(
      /(?<!config )\bnu(?=\s+(?:--|-c\b|-l\b|-n\b|<[A-Za-z]|[A-Za-z0-9_.-]+\.nu\b))/g,
      "nutorch",
    );
}

if (!dirExists(VENDOR)) {
  console.error(`Missing Nushell docs clone: ${VENDOR}`);
  process.exit(1);
}

const commandFiles = walk(join(VENDOR, "commands/docs"));
const commands = commandFiles.map((file) => {
  const parsed = splitFrontmatter(readFileSync(file, "utf8"));
  const rel = relPosix(file);
  const title = parsed.data.title ?? "command";
  const usage = parsed.data.usage ?? title;
  return {
    title,
    slug: slugFor(rel),
    usage,
    categories: categoriesOf(parsed.data.categories),
    rel,
  };
});

const trees = ["book", "lang-guide", "cookbook", "commands"];
const sources = trees.flatMap((tree) => walk(join(VENDOR, tree)));
rmSync(OUT, { recursive: true, force: true });
rmSync(IMAGES, { recursive: true, force: true });

const pages: Page[] = [];
for (const file of sources) {
  const rel = relPosix(file);
  const parsed = splitFrontmatter(readFileSync(file, "utf8"));
  let body = inlineSnippets(parsed.body);
  body = containers(body);
  body = rewriteLinks(body, rel);
  const slug = slugFor(rel);
  if (!slug) throw new Error(`No slug for ${rel}`);
  let section = "Book · Introduction";
  let category = "";
  if (rel.startsWith("book/")) section = bookSection(rel);
  else if (rel.startsWith("lang-guide/")) section = "Language";
  else if (rel.startsWith("cookbook/")) section = "Cookbook";
  else if (rel === "commands/README.md") section = "Commands";
  else if (rel.startsWith("commands/categories/")) {
    category = rel.slice("commands/categories/".length).replace(/\.md$/, "");
    section = `Commands · ${category}`;
    body = replaceVueTable(body, staticCommandList(commands, category));
  } else if (rel.startsWith("commands/docs/")) {
    const found = commands.find((command) => command.rel === rel);
    category = found?.categories[0] ?? "misc";
    section = `Commands · ${category}`;
  }
  if (rel === "commands/README.md") {
    const groups = [
      ...new Set(commands.flatMap((command) => command.categories)),
    ].sort();
    const index = groups
      .map((name) => {
        const list = staticCommandList(commands, name);
        return list ? `## ${name}\n\n${list}` : "";
      })
      .filter((part) => part.length > 0)
      .join("\n\n");
    body = replaceVueTable(body, index);
  }
  if (rel === "book/configuration.md") {
    body = body.replace(
      /# Configuration\n/,
      "# Configuration\n\nNuTorch is the shell that reads these startup files. The binary is `nutorch`. It uses the same `env.nu`, `config.nu`, and XDG paths described on this page.\n",
    );
    body = shellInvocations(body);
  }
  if (rel === "book/README.md") {
    body = body.replace(
      /^(# Introduction\n)/,
      "$1\nNuTorch is this shell. Launch it with `nutorch`. It reads the same `env.nu`, `config.nu`, and XDG startup paths described in the configuration chapter.\n",
    );
  }
  const titled = parsed.data.title?.trim();
  const named =
    titled !== undefined && titled.length > 0 ? titled : plainTitle(body, slug);
  const dropped = dropPageTitle(body, named);
  body = dropped.body;
  const title = dropped.title;
  const described = parsed.data.usage ?? parsed.data.description ?? title;
  pages.push({
    rel,
    slug,
    section,
    title,
    description: oneLine(described, title),
    body: `${body.trimEnd()}${NOTICE}`,
    category,
  });
}

const sectionRank = new Map<string, number>();
let rank = 0;
for (const [section] of BOOK_CHAPTERS) sectionRank.set(section, rank++);
sectionRank.set("Language", rank++);
sectionRank.set("Cookbook", rank++);
sectionRank.set("Commands", rank++);
for (const page of pages) {
  if (
    page.section.startsWith("Commands · ") &&
    !sectionRank.has(page.section)
  ) {
    sectionRank.set(page.section, rank++);
  }
}
pages.sort((a, b) => {
  const left = sectionRank.get(a.section) ?? 1000;
  const right = sectionRank.get(b.section) ?? 1000;
  if (left !== right) return left - right;
  return a.slug.localeCompare(b.slug);
});

pages.forEach((page, index) => {
  const content = dumpPage(page.body, {
    title: page.title,
    description: page.description,
    order: 1000 + index,
    section: page.section,
  });
  const dest = join(OUT, `${page.slug.slice("shell/".length)}.md`);
  mkdirSync(dirname(dest), { recursive: true });
  writeFileSync(dest, content);
});

const counts = {
  book: pages.filter((page) => page.slug.startsWith("shell/book/")).length,
  lang: pages.filter((page) => page.slug.startsWith("shell/lang-guide/"))
    .length,
  cookbook: pages.filter((page) => page.slug.startsWith("shell/cookbook/"))
    .length,
  commands: pages.filter((page) => page.slug.startsWith("shell/commands/"))
    .length,
};
console.log(
  `ported ${String(pages.length)} pages (book ${String(counts.book)}, lang ${String(counts.lang)}, cookbook ${String(counts.cookbook)}, commands ${String(counts.commands)})`,
);
if (
  counts.book !== 65 ||
  counts.lang !== 53 ||
  counts.cookbook !== 20 ||
  counts.commands !== 751
) {
  console.error(`FAIL: unexpected counts ${JSON.stringify(counts)}`);
  process.exit(1);
}
