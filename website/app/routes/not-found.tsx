import { Link, href } from "react-router";
import { Button } from "@astrohacker/ui/button";
import { metadata } from "../lib/metadata";
export function meta(): ReturnType<typeof metadata> {
  return metadata(
    "404 — NuTorch",
    "No such tensor: the page you asked for is not in the registry.",
    "/404/",
  );
}
export default function NotFound(): React.JSX.Element {
  return (
    <section className="mx-auto max-w-5xl px-6 py-24 text-center">
      <img
        src="/images/brand/nutorch-dark-200.webp"
        alt=""
        width="96"
        height="96"
        className="mx-auto mb-8 opacity-60"
      />
      <h1 className="font-display text-3xl font-bold tracking-tight">
        unknown handle: <span className="font-mono text-accent">404://</span>
      </h1>
      <p className="mt-4 text-muted">
        No such tensor — the page you asked for is not in the registry.
      </p>
      <div className="mt-8 flex justify-center gap-4">
        <Button asChild>
          <Link to={href("/")}>Home</Link>
        </Button>
        <Button asChild variant="secondary">
          <Link to={href("/docs/*", { "*": "" })}>Docs</Link>
        </Button>
      </div>
    </section>
  );
}
