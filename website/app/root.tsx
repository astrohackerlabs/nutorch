import { useEffect } from "react";
import { Links, Meta, Outlet, Scripts, ScrollRestoration } from "react-router";
import Header from "./components/Header";
import Footer from "./components/Footer";
import { SpaceRain } from "@astrohacker/ui/space-rain";
import "@fontsource/space-grotesk/400.css";
import "@fontsource/space-grotesk/500.css";
import "@fontsource/space-grotesk/600.css";
import "@fontsource/space-grotesk/700.css";
import "@fontsource/jetbrains-mono/400.css";
import "./styles/global.css";

export function Layout({
  children,
}: {
  children: React.ReactNode;
}): React.JSX.Element {
  return (
    <html lang="en" data-theme="dark" suppressHydrationWarning>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <link
          rel="icon"
          href="/favicon-dark.png"
          type="image/png"
          sizes="32x32"
        />
        <link
          rel="icon"
          href="/favicon-dark.png"
          type="image/png"
          sizes="32x32"
          media="(prefers-color-scheme: dark)"
        />
        <link
          rel="icon"
          href="/favicon-light.png"
          type="image/png"
          sizes="32x32"
          media="(prefers-color-scheme: light)"
        />
        <Links />
      </head>
      <body className="flex min-h-screen flex-col bg-background text-foreground antialiased">
        <div
          data-site-background
          className="relative isolate min-h-dvh bg-background text-foreground"
        >
          <SpaceRain data-testid="ntcom-space-rain" />
          <div
            data-site-scrim
            aria-hidden="true"
            className="pointer-events-none fixed inset-0 z-2 bg-background/55"
          />
          <div
            data-site-content
            className="relative z-10 flex min-h-dvh flex-col"
          >
            <Header />
            <main className="flex-1">{children}</main>
            <Footer />
          </div>
        </div>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}
export default function App(): React.JSX.Element {
  useEffect(() => {
    const click = async (event: MouseEvent): Promise<void> => {
      const target = event.target instanceof Element ? event.target : null;
      const copy = target?.closest<HTMLElement>("[data-copy]");
      if (copy?.dataset.copy) {
        try {
          await navigator.clipboard.writeText(copy.dataset.copy);
          copy.textContent = "copied";
          setTimeout(() => {
            if (copy.isConnected) copy.textContent = "copy";
          }, 1500);
        } catch {
          /* Storage or optional fixture read may be unavailable. */
        }
      }
    };
    const onClick = (event: MouseEvent): void => {
      void click(event);
    };
    document.addEventListener("click", onClick);
    return (): void => {
      document.removeEventListener("click", onClick);
    };
  }, []);
  return <Outlet />;
}
