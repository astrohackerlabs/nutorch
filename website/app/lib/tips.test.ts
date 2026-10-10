import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, test } from "bun:test";
import { RouterContextProvider } from "react-router";
import { documents } from "./docs.server";
import { renderMarkdown } from "./markdown.server";
import { tipPages } from "./tips";
import { loader } from "../routes/docs";

const expectedPages = {
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

interface CatalogTip {
  kind: string;
  type: keyof typeof expectedPages;
  level: "intro" | "basic" | "intermediate" | "advanced";
  body: string;
}

const catalog = JSON.parse(
  readFileSync(
    fileURLToPath(
      new URL("../../../../rs/shell/tips/catalog.json", import.meta.url),
    ),
    "utf8",
  ),
) as CatalogTip[];

function page(slug: string): (typeof documents)[number] {
  const found = documents.find((doc) => doc.slug === slug);
  expect(found).toBeTruthy();
  if (!found) throw new Error(`missing ${slug}`);
  return found;
}

function caption(body: string): string {
  return (body.split("\n")[0] ?? "").replace(/^# /, "");
}

function block(body: string): string {
  const code = body.split("\n").slice(1).join("\n").replace(/\n+$/, "");
  const paragraph = `<p>${caption(body)}</p>`;
  if (code.length === 0) return paragraph;
  return `${paragraph}\n\n\`\`\`nu\n${code}\n\`\`\``;
}

async function rendered(slug: string): Promise<string> {
  const data = await loader({
    params: { "*": slug },
    request: new Request(`http://localhost/docs/${slug}`),
    url: new URL(`http://localhost/docs/${slug}`),
    pattern: "/docs/*",
    context: new RouterContextProvider(),
  });
  return data.html;
}

function tipsHtml(html: string): string {
  const at = html.indexOf('id="tips"');
  expect(at).toBeGreaterThan(-1);
  return html.slice(at);
}

function visible(html: string): string {
  return html.replace(/<[^>]+>/g, "");
}

test("every catalog tip is on its one topic page", () => {
  expect(tipPages).toEqual(expectedPages);
  expect(catalog).toHaveLength(310);
  expect(
    documents.filter((doc) => doc.slug.startsWith("shell/book/")),
  ).toHaveLength(65);
  expect(
    documents.filter((doc) => doc.slug.startsWith("shell/lang-guide/")),
  ).toHaveLength(53);
  expect(
    documents.filter((doc) => doc.slug.startsWith("shell/cookbook/")),
  ).toHaveLength(20);
  expect(
    documents.filter((doc) => doc.slug.startsWith("shell/commands/")),
  ).toHaveLength(751);

  const hosts = new Set(Object.values(expectedPages));
  for (const slug of hosts) {
    const markers = page(slug).content.split("\n## Tips\n").length - 1;
    expect(markers).toBe(1);
  }

  for (const tip of catalog) {
    const slug = expectedPages[tip.type];
    const marker = block(tip.body);
    const homes = documents.filter((doc) => doc.content.includes(marker));
    expect(homes.map((doc) => doc.slug)).toEqual([slug]);
  }

  for (const slug of [
    "shell/book/pipelines",
    "shell/commands/categories/filters",
    "tensors",
    "reference/autograd",
  ]) {
    const content = page(slug).content;
    const intro = content.indexOf("\n### Intro\n");
    const basic = content.indexOf("\n### Basic\n");
    const intermediate = content.indexOf("\n### Intermediate\n");
    const advanced = content.indexOf("\n### Advanced\n");
    expect(intro).toBeGreaterThan(content.indexOf("\n## Tips\n"));
    expect(basic).toBeGreaterThan(intro);
    expect(intermediate).toBeGreaterThan(basic);
    expect(advanced).toBeGreaterThan(intermediate);
  }
});

test("rendered topic pages show the caption and the nu example", async () => {
  const pipelines = await rendered("shell/book/pipelines");
  const pipelineTips = visible(tipsHtml(pipelines));
  expect(pipelines).toContain(
    "A pipe sends one command's output into the next.",
  );
  expect(pipelineTips).toContain("[1 2 3] | math sum");
  expect(pipelines).not.toMatch(/<h1[\s>]/);
  expect(pipelines).toContain('id="intro"');
  expect(pipelines).toContain('id="basic"');
  expect(pipelines).toContain('id="intermediate"');
  expect(pipelines).toContain('id="advanced"');
  expect(pipelines).toContain("astro-code");

  const filters = await rendered("shell/commands/categories/filters");
  expect(visible(tipsHtml(filters))).toContain("first keeps the opening item.");
  expect(filters).not.toMatch(/<h1[\s>]/);

  const tensors = await rendered("tensors");
  const tensorTips = visible(tipsHtml(tensors));
  expect(tensorTips).toContain("A tensor is a native array of numbers.");
  expect(tensorTips).toContain("use torch");
  expect(tensors).toContain("astro-code");
  expect(tensors).not.toMatch(/<h1[\s>]/);

  const autograd = await rendered("reference/autograd");
  expect(visible(tipsHtml(autograd))).toContain(
    "requires-grad marks a tensor for differentiation.",
  );
  expect(autograd).not.toMatch(/<h1[\s>]/);

  const pipelinesMarkdown = await renderMarkdown(
    page("shell/book/pipelines").content,
  );
  expect(visible(pipelinesMarkdown)).toContain("[1 2 3] | math sum");
  expect(pipelinesMarkdown).not.toMatch(/<h1[\s>]/);
});

test("the shell page points at topic tips and copies stay off the other chapters", () => {
  const shell = page("nushell");
  expect(shell.content).toContain(
    "Tips for each topic are at the bottom of that page. In the shell, `nutorch tip` prints one tip.",
  );
  expect(shell.content).not.toContain(
    "A pipe sends one command's output into the next.",
  );

  const guide = page("shell/lang-guide/chapters/pipelines");
  expect(guide.content).not.toContain(
    "A pipe sends one command's output into the next.",
  );
  expect(guide.content).not.toContain("\n## Tips\n");

  const stringCommands = page("shell/commands/categories/strings");
  const stringTip = catalog.find((tip) => tip.type === "strings");
  expect(stringTip).toBeTruthy();
  if (!stringTip) return;
  expect(stringCommands.content).not.toContain(block(stringTip.body));
  expect(page("shell/book/working_with_strings").content).toContain(
    block(stringTip.body),
  );

  const concept = page("autograd");
  expect(concept.content).toContain(
    "Tips for these commands are at the bottom of [Autograd ops](/docs/reference/autograd/#tips).",
  );
  expect(concept.content).not.toContain("\n## Tips\n");
});
