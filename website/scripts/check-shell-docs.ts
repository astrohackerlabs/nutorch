// Catalog proof for the English Nushell docs port.
import assert from "node:assert/strict";
import { documents } from "../app/lib/docs.server.ts";
import { renderMarkdown } from "../app/lib/markdown.server.ts";

function bySlug(slug: string): (typeof documents)[number] {
  const found = documents.find((doc) => doc.slug === slug);
  assert(found, `missing shipped doc ${slug}`);
  return found;
}

function count(prefix: string): number {
  return documents.filter((doc) => doc.slug.startsWith(prefix)).length;
}

assert.equal(count("shell/book/"), 65);
assert.equal(count("shell/lang-guide/"), 53);
assert.equal(count("shell/cookbook/"), 20);
assert.equal(count("shell/commands/"), 751);
for (const slug of ["getting-started", "tensors", "reference/creation"]) {
  bySlug(slug);
}
for (const doc of documents) {
  if (!doc.slug.startsWith("shell/")) continue;
  assert(
    doc.content.includes("Copyright (c) 2021 Nushell Project"),
    `${doc.slug} is missing the MIT notice`,
  );
}

function outsideCode(html: string): string {
  return html
    .replace(/<pre[\s\S]*?<\/pre>/g, "")
    .replace(/<code[\s\S]*?<\/code>/g, "");
}

function assertNoPageTitle(html: string, slug: string): void {
  assert(!/<h1[\s>]/.test(html), `${slug} still renders a markdown h1`);
}

const introduction = await renderMarkdown(bySlug("shell/book/README").content);
assertNoPageTitle(introduction, "shell/book/README");
const keypresses = await renderMarkdown(
  bySlug("shell/cookbook/input_listen_keys").content,
);
assertNoPageTitle(keypresses, "shell/cookbook/input_listen_keys");
assert.equal(
  bySlug("shell/cookbook/ssh_agent").title,
  "Manage SSH passphrases",
);
assertNoPageTitle(
  await renderMarkdown(bySlug("shell/cookbook/ssh_agent").content),
  "shell/cookbook/ssh_agent",
);
assert.equal(
  bySlug("shell/cookbook/foreign_shell_scripts").title,
  "Working With Foreign Shell Scripts",
);
assertNoPageTitle(
  await renderMarkdown(bySlug("shell/cookbook/foreign_shell_scripts").content),
  "shell/cookbook/foreign_shell_scripts",
);

const configuration = await renderMarkdown(
  bySlug("shell/book/configuration").content,
);
assertNoPageTitle(configuration, "shell/book/configuration");
const configurationText = outsideCode(configuration);
assert(configuration.includes("NuTorch"));
assert(configuration.includes("env.nu"));
assert(configuration.includes("config.nu"));
assert(configuration.includes("XDG_CONFIG_HOME"));
assert(!/::: (?:tip|warning|note|important)/.test(configurationText));
assert(!configurationText.includes("<p>:::</p>"));

const tour = await renderMarkdown(bySlug("shell/book/quick_tour").content);
assert(tour.length > 500);
assert(tour.includes("Finding Data Using the"));

const openDoc = bySlug("shell/commands/open");
assert.equal(openDoc.title, "open");
const open = await renderMarkdown(openDoc.content);
assertNoPageTitle(open, "shell/commands/open");
assert(open.includes("/docs/shell/commands/categories/filesystem/"));
assert(open.includes("Load a file into a cell"));
const configurationSource = bySlug("shell/book/configuration").content;
assert(configurationSource.includes("> ```nu\n> config nu --doc"));
assert(!configurationSource.includes("config nutorch"));
assert(configurationSource.includes("`nutorch` (no flags)"));
assert(configurationSource.includes("`nutorch --commands"));
assert(configurationSource.includes("`nutorch <script_file>`"));
assert(configurationSource.includes("`nutorch --help`"));

const filesystem = await renderMarkdown(
  bySlug("shell/commands/categories/filesystem").content,
);
assert(filesystem.includes("/docs/shell/commands/open/"));
assert(!filesystem.includes("v-for"));
assert(!filesystem.includes("<script"));

const language = await renderMarkdown(
  bySlug("shell/lang-guide/chapters/types/00_types_overview").content,
);
assert(language.includes("Nu is strongly typed and gradually typed."));

const cookbook = await renderMarkdown(bySlug("shell/cookbook/tables").content);
assert(cookbook.includes("merge tables of different sizes"));

const creatingModules = await renderMarkdown(
  bySlug("shell/book/modules/creating_modules").content,
);
assert(creatingModules.includes('id="module-files"'));
assert(!outsideCode(creatingModules).includes("::: note"));
assert(outsideCode(creatingModules).includes("Additionally, "));

const customCommands = await renderMarkdown(
  bySlug("shell/book/custom_commands").content,
);
const customPres = [...customCommands.matchAll(/<pre[\s\S]*?<\/pre>/g)].map(
  (match) => match[0].replace(/<[^>]+>/g, ""),
);
const randomFile = customPres.find((pre) => pre.includes("random file"));
assert(randomFile, "missing the match-expression sample");
assert(!randomFile.includes("exponents-of-three"));
assert(customPres.some((pre) => pre.includes("exponents-of-three")));

console.log("shell docs catalog passed");
