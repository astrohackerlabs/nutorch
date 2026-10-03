import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { MemoryRouter } from "react-router";
import { documents } from "../lib/docs.server";
import { renderMarkdown } from "../lib/markdown.server";
import Docs from "./docs";
import Home, { loader as loadHome } from "./home";
import type { Route as DocsRoute } from "./+types/docs";
import type { Route as HomeRoute } from "./+types/home";

const rootMatch = {
  id: "root" as const,
  params: {},
  pathname: "/",
  loaderData: undefined,
  handle: undefined,
};

function DocsFixture({
  loaderData,
}: Pick<DocsRoute.ComponentProps, "loaderData">): React.JSX.Element {
  const params = { "*": loaderData.entry.slug };
  return (
    <Docs
      loaderData={loaderData}
      params={params}
      matches={[
        rootMatch,
        {
          id: "routes/docs",
          params,
          pathname: `/docs/${loaderData.entry.slug}`,
          loaderData,
          handle: undefined,
        },
      ]}
    />
  );
}

function HomeFixture({
  loaderData,
}: Pick<HomeRoute.ComponentProps, "loaderData">): React.JSX.Element {
  return (
    <Home
      loaderData={loaderData}
      params={{}}
      matches={[
        rootMatch,
        {
          id: "routes/home",
          params: {},
          pathname: "/",
          loaderData,
          handle: undefined,
        },
      ]}
    />
  );
}

function sliceDiv(html: string, token: string): string {
  const at = html.indexOf(token);
  expect(at).toBeGreaterThanOrEqual(0);
  const start = html.lastIndexOf("<div", at);
  let depth = 0;
  const tags = /<div\b|<\/div>/g;
  tags.lastIndex = start;
  for (let match = tags.exec(html); match; match = tags.exec(html)) {
    depth += match[0] === "<div" ? 1 : -1;
    if (depth === 0) return html.slice(start, match.index + match[0].length);
  }
  throw new Error(`unclosed element for ${token}`);
}

test("docs article sits in the glass column and the nav stays outside", () => {
  const html = renderToStaticMarkup(
    <MemoryRouter>
      <DocsFixture
        loaderData={{
          entry: {
            slug: "tensors",
            title: "Tensors",
            description: "Tensor values",
            order: 2,
            section: "Core",
            content: undefined,
          },
          html: "<p>Tensor body</p>",
          items: [
            {
              slug: "getting-started",
              title: "Getting started",
              description: "Start",
              order: 1,
              section: "Start",
            },
            {
              slug: "tensors",
              title: "Tensors",
              description: "Tensor values",
              order: 2,
              section: "Core",
            },
          ],
          indexable: true,
        }}
      />
    </MemoryRouter>,
  );
  const columns = html.match(/data-reading-column="docs"/g) ?? [];
  expect(columns).toHaveLength(1);
  const column = sliceDiv(html, 'data-reading-column="docs"');
  expect(column).toContain(">Tensors<");
  expect(column).toContain('id="docs-search"');
  expect(column).toContain("bg-background/55");
  expect(column).toContain("backdrop-blur-md");
  expect(column).toContain("supports-[backdrop-filter]:bg-background/40");
  expect(column).toContain("min-h-dvh");
  expect(column).toContain("border-x");
  expect(column).toContain("max-w-3xl");
  expect(column).not.toContain("rounded");
  expect(html).toContain('aria-label="Documentation"');
  expect(column).not.toContain('aria-label="Documentation"');
});

test("shipped docs pages have one title", async () => {
  const items = documents.map(
    ({ slug, title, description, order, section }) => ({
      slug,
      title,
      description,
      order,
      section,
    }),
  );
  for (const slug of [
    "shell/book/README",
    "shell/commands/open",
    "getting-started",
  ]) {
    const doc = documents.find((item) => item.slug === slug);
    expect(doc).toBeTruthy();
    if (!doc) continue;
    const html = renderToStaticMarkup(
      <MemoryRouter>
        <DocsFixture
          loaderData={{
            entry: { ...doc, content: undefined },
            html: await renderMarkdown(doc.content),
            items,
            indexable: true,
          }}
        />
      </MemoryRouter>,
    );
    const titles = [...html.matchAll(/<h1\b[^>]*>([\s\S]*?)<\/h1>/g)].map(
      (match) => match[1]?.replace(/<[^>]+>/g, "").trim(),
    );
    expect(titles).toEqual([doc.title]);
  }
  expect(documents.find((doc) => doc.slug === "getting-started")?.title).toBe(
    "Getting started",
  );
});

test("homepage stays outside the reading column", async () => {
  const html = renderToStaticMarkup(
    <MemoryRouter>
      <HomeFixture loaderData={await loadHome()} />
    </MemoryRouter>,
  );
  expect(html).not.toContain("data-reading-column");
});
