import { Link, href } from "react-router";
import { MotionModeSelector, useMotionMode } from "@astrohacker/ui/motion-mode";
export default function Header(): React.JSX.Element {
  const { mode, setMode } = useMotionMode();
  return (
    <header className="sticky top-0 z-50 border-b border-border bg-background/85 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-3 sm:px-6 lg:px-8">
        <Link to={href("/")} className="flex items-center gap-2 sm:gap-2.5">
          <img
            className="size-7 sm:size-8"
            src="/images/brand/nutorch-dark-64.webp"
            srcSet="/images/brand/nutorch-dark-64.webp 1x, /images/brand/nutorch-dark-128.webp 2x"
            alt=""
            width="32"
            height="32"
          />
          <span className="font-display text-lg font-bold tracking-tight sm:text-xl">
            NuTorch
          </span>
        </Link>
        <nav className="flex items-center gap-2 sm:gap-5">
          <Link
            to={href("/docs/*", { "*": "" })}
            className="text-sm font-medium text-muted transition-colors hover:text-foreground"
          >
            Docs
          </Link>
          <a
            href="https://github.com/astrohackerlabs/nutorch"
            className="text-sm font-medium text-muted transition-colors hover:text-foreground"
          >
            GitHub
          </a>
          <MotionModeSelector value={mode} onValueChange={setMode} />
        </nav>
      </div>
    </header>
  );
}
