import { useEffect, useRef } from "react";
import { useLocation } from "react-router";
import { createPageviews } from "../lib/umami";

export function UmamiPageviews(): null {
  const { pathname, search } = useLocation();
  const pageviews = useRef<ReturnType<typeof createPageviews> | null>(null);

  useEffect(() => {
    const views = (pageviews.current ??= createPageviews());
    const script = document.querySelector<HTMLScriptElement>(
      "script[data-umami-tracker]",
    );
    if (!script) {
      views.fail();
      return;
    }
    // Also handles a failed script whose error event preceded hydration.
    const timeout = window.setTimeout(views.fail, 30_000);
    const ready = (): void => {
      if (typeof window.umami?.track === "function") {
        window.clearTimeout(timeout);
        views.ready(window.umami);
      }
    };
    const fail = (): void => {
      window.clearTimeout(timeout);
      views.fail();
    };
    script.addEventListener("load", ready);
    script.addEventListener("error", fail);
    ready();
    return (): void => {
      window.clearTimeout(timeout);
      script.removeEventListener("load", ready);
      script.removeEventListener("error", fail);
    };
  }, []);

  useEffect(() => {
    pageviews.current?.visit({
      url: `${window.location.origin}${pathname}${search}`,
      title: document.title,
      referrer: document.referrer,
    });
  }, [pathname, search]);

  return null;
}
