import type { ReactNode } from "react";

const glass =
  "border-border/40 bg-background/55 shadow-[0_0_40px_rgba(0,0,0,0.25)] backdrop-blur-md supports-[backdrop-filter]:bg-background/40";

/** Full-height glass strip for documentation prose. Not a short card. */
export function ReadingColumn({
  children,
}: {
  children: ReactNode;
}): React.JSX.Element {
  return (
    <div
      data-reading-column="docs"
      className={`${glass} relative z-10 flex min-h-dvh w-full max-w-3xl min-w-0 flex-1 flex-col border-x px-6 pt-10 pb-8`}
    >
      {children}
    </div>
  );
}
