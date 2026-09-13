import { useEffect, useRef } from "react";
import { useLocation } from "react-router";

declare global {
  interface Window {
    PagefindUI?: new (options: {
      element: HTMLElement;
      showSubResults: boolean;
      showImages: boolean;
    }) => { destroy(): void };
  }
}
let loading: Promise<void> | undefined;
function load(): Promise<void> {
  if (window.PagefindUI) return Promise.resolve();
  return (loading ??= new Promise<void>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = "/pagefind/pagefind-ui.js";
    script.onload = (): void => {
      resolve();
    };
    script.onerror = (): void => {
      script.remove();
      loading = undefined;
      reject(
        new Error(
          "Search is available after building and previewing the site.",
        ),
      );
    };
    document.head.append(script);
  }));
}
export default function Search(): React.JSX.Element {
  const element = useRef<HTMLDivElement>(null);
  const location = useLocation();
  useEffect(() => {
    let cancelled = false;
    let ui: { destroy(): void } | undefined;
    const target = element.current;
    if (target)
      void load()
        .then(() => {
          if (!cancelled && window.PagefindUI)
            ui = new window.PagefindUI({
              element: target,
              showSubResults: true,
              showImages: false,
            });
        })
        .catch(() => {
          if (!cancelled)
            target.textContent = "Search is available in the built preview.";
        });
    return (): void => {
      cancelled = true;
      ui?.destroy();
      if (target) target.replaceChildren();
    };
  }, [location.pathname]);
  return (
    <div ref={element} id="docs-search" className="mb-8" data-pagefind-ignore />
  );
}
