export default function Footer(): React.JSX.Element {
  return (
    <footer className="border-t border-border bg-background/80 backdrop-blur-sm">
      <div className="mx-auto flex max-w-7xl flex-col gap-6 px-6 py-10 text-sm text-muted">
        <div className="flex flex-col items-center justify-between gap-3 sm:flex-row">
          <div className="flex items-center gap-2">
            <img
              src="/images/brand/nutorch-dark-64.webp"
              alt=""
              width="20"
              height="20"
            />
            <span>NuTorch · MIT</span>
          </div>
          <a
            href="https://github.com/astrohackerlabs/nutorch"
            className="transition-colors hover:text-foreground"
          >
            github.com/astrohackerlabs/nutorch
          </a>
        </div>
        <div className="flex flex-col items-center gap-2 text-xs">
          <a
            href="https://astrohacker.com"
            className="flex items-center gap-2 transition-colors hover:text-accent"
          >
            <img
              src="/images/brand/astrohacker-dark-64.webp"
              srcSet="/images/brand/astrohacker-dark-64.webp 1x, /images/brand/astrohacker-dark-128.webp 2x"
              alt="Astrohacker logo"
              className="h-5 w-5"
              width="20"
              height="20"
            />
            An Astrohacker Project
          </a>
          <p>&copy; {new Date().getFullYear()} Astrohacker · MIT License</p>
        </div>
      </div>
    </footer>
  );
}
