import { Link, href } from "react-router";
import { useState } from "react";
import { Button } from "@astrohacker/ui/button";
import {
  Sheet,
  SheetTrigger,
  SheetContent,
  SheetTitle,
} from "@astrohacker/ui/sheet";
import NotFound from "./not-found";
import { metadata } from "../lib/metadata";
import type { Route } from "./+types/docs";
import Search from "../components/Search";
import { documents } from "../lib/docs.server";
import { renderMarkdown } from "../lib/markdown.server";

interface DocsData {
  entry: Omit<(typeof documents)[number], "content"> & { content: undefined };
  html: string;
  items: Omit<(typeof documents)[number], "content">[];
  indexable: boolean;
}

export async function loader({ params }: Route.LoaderArgs): Promise<DocsData> {
  const slug = params["*"].replace(/\/$/, "") || "getting-started";
  const entry = documents.find((d) => d.slug === slug);
  if (!entry) throw new Response("Not found", { status: 404 });
  return {
    entry: { ...entry, content: undefined },
    html: await renderMarkdown(entry.content),
    items: documents.map(({ slug, title, description, order, section }) => ({
      slug,
      title,
      description,
      order,
      section,
    })),
    indexable: !!params["*"],
  };
}
export function meta({
  loaderData: data,
}: Route.MetaArgs): ReturnType<typeof metadata> {
  if (!data) return [{ title: "404 — NuTorch" }];
  return metadata(
    `${data.entry.title} — NuTorch docs`,
    data.entry.description,
    `/docs/${data.entry.slug}/`,
    true,
  );
}
export function links(): ReturnType<Route.LinksFunction> {
  return [{ rel: "stylesheet", href: "/pagefind/pagefind-ui.css" }];
}
export function ErrorBoundary(): React.JSX.Element {
  return <NotFound />;
}
export default function Docs({
  loaderData: data,
}: Route.ComponentProps): React.JSX.Element {
  const [menuOpen, setMenuOpen] = useState(false);
  const groups = [...new Set(data.items.map((d) => d.section))];
  const idx = data.items.findIndex((d) => d.slug === data.entry.slug);
  const previous = idx > 0 ? data.items[idx - 1] : undefined;
  const next = data.items[idx + 1];
  const address = (slug: string): string =>
    href("/docs/*", { "*": `${slug}/` });
  const nav = (
    <div className="space-y-6">
      {groups.map((section) => (
        <div key={section}>
          {section && (
            <p className="mb-2 text-xs font-semibold tracking-wider text-muted uppercase">
              {section}
            </p>
          )}
          <ul className="space-y-1.5">
            {data.items
              .filter((d) => d.section === section)
              .map((d) => (
                <li key={d.slug}>
                  <Link
                    onClick={() => {
                      setMenuOpen(false);
                    }}
                    to={address(d.slug)}
                    aria-current={
                      d.slug === data.entry.slug ? "page" : undefined
                    }
                    className={`block text-sm transition-colors ${d.slug === data.entry.slug ? "font-semibold text-primary" : "text-muted hover:text-foreground"}`}
                  >
                    {d.title}
                  </Link>
                </li>
              ))}
          </ul>
        </div>
      ))}
    </div>
  );
  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-10 md:flex-row">
      <nav className="w-full shrink-0 md:w-48" aria-label="Documentation">
        <div className="md:hidden">
          <Sheet open={menuOpen} onOpenChange={setMenuOpen}>
            <SheetTrigger asChild>
              <Button>Documentation</Button>
            </SheetTrigger>
            <SheetContent
              side="left"
              className="max-w-[90vw] overflow-y-auto"
              aria-describedby={undefined}
            >
              <div className="mb-6 flex items-center justify-between gap-2">
                <SheetTitle>Documentation</SheetTitle>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => {
                    setMenuOpen(false);
                  }}
                >
                  Close
                </Button>
              </div>
              {nav}
            </SheetContent>
          </Sheet>
        </div>
        <div className="sticky top-24 hidden max-h-[calc(100dvh-7rem)] overflow-y-auto px-1 py-1 md:block">
          {nav}
        </div>
      </nav>
      <article
        className="prose-nutorch min-w-0 flex-1"
        data-pagefind-body={data.indexable ? true : undefined}
      >
        <Search />
        <h1>{data.entry.title}</h1>
        <div dangerouslySetInnerHTML={{ __html: data.html }} />
        <div className="mt-12 flex justify-between gap-4 border-t border-border pt-6 text-sm">
          {previous ? (
            <Link
              to={address(previous.slug)}
              className="text-muted hover:text-foreground"
            >
              ← {previous.title}
            </Link>
          ) : (
            <span />
          )}
          {next && (
            <Link
              to={address(next.slug)}
              className="text-right font-medium text-primary hover:text-primary-strong"
            >
              {next.title} →
            </Link>
          )}
        </div>
      </article>
    </div>
  );
}
