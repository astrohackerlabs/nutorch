import { Link, href } from "react-router";
export default function Header(): React.JSX.Element {
  return (
    <header className="sticky top-0 z-50 border-b border-border bg-background/85 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
        <Link to={href("/")} className="flex items-center gap-2.5">
          <img
            src="/images/brand/nutorch-dark-64.webp"
            srcSet="/images/brand/nutorch-dark-64.webp 1x, /images/brand/nutorch-dark-128.webp 2x"
            alt=""
            width="32"
            height="32"
          />
          <span className="font-display text-xl font-bold tracking-tight">
            NuTorch
          </span>
        </Link>
        <nav className="flex items-center gap-5">
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
        </nav>
      </div>
    </header>
  );
}
