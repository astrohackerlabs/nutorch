import type { Route } from "./+types/home";
import { Link, href } from "react-router";
import { metadata } from "../lib/metadata";
import { highlight } from "../lib/markdown.server";
import { Button } from "@astrohacker/ui/button";
import { Card } from "@astrohacker/ui/card";
const launchDemo = `./code/nutorch/rs/target/release/nutorch`;

const heroDemo = `use torch
let a = (torch tensor [1 2 3])
let b = (torch tensor [4 5 6])
torch add $a $b | torch value
# [5.0, 7.0, 9.0] — GPU computation, returned as a list`;

const nuDemo = `use torch
let t = ([[1 2] [3 4]] | torch tensor)
$t | torch mm $t | torch value
# [[7.0, 10.0], [15.0, 22.0]]`;

const trainDemo = `use torch
let w = torch tensor [1 2 3] --requires_grad
let loss = (torch mul $w $w | torch sum)
torch backward $loss
torch grad $w | torch value
# [2.0, 4.0, 6.0]`;

const features = [
  {
    title: "Apple-silicon GPU",
    body:
      "NuTorch runs tensors on Metal through LibTorch. There is no CPU mode " +
      "and no device flag.",
  },
  {
    title: "A full Nushell shell",
    body:
      "Use structured pipelines, records, lists, functions and scripts. " +
      "Import the native tensor commands with use torch.",
  },
  {
    title: "PyTorch-style API",
    body:
      "Operation names, arguments, defaults, broadcasting, autograd, modules, " +
      "and optimizers follow PyTorch wherever possible.",
  },
  {
    title: "Native tensor ownership",
    body:
      "The shell owns shared tensor, module and optimizer references. " +
      "LibTorch manages GPU storage and the computation graph in the same process.",
  },
];
export async function loader(): Promise<Record<string, string>> {
  const entries = await Promise.all(
    (
      [
        [launchDemo, "nu"],
        [heroDemo, "nu"],
        [nuDemo, "nu"],
        [trainDemo, "nu"],
      ] as const
    ).map(async ([code, lang]) => [code, await highlight(code, lang)]),
  );
  return Object.fromEntries(entries) as Record<string, string>;
}
export function meta(): ReturnType<typeof metadata> {
  return metadata(
    "NuTorch — A shell for GPU computing",
    "NuTorch is a Nushell-based shell with native GPU tensors, neural networks and optimizers on Apple-silicon Metal. The new shell is unreleased.",
    "/",
  );
}
function CodeBlock({
  blocks,
  code,
}: {
  blocks: Record<string, string>;
  code: string;
}): React.JSX.Element {
  return <div dangerouslySetInnerHTML={{ __html: blocks[code] ?? "" }} />;
}
export default function Home({
  loaderData,
}: Route.ComponentProps): React.JSX.Element {
  return (
    <>
      <section className="mx-auto w-full max-w-5xl px-6 pt-16 pb-12 sm:pt-24">
        <div className="flex flex-col items-center gap-10 sm:flex-row sm:gap-14">
          <div className="relative shrink-0">
            <div
              className="hero-glow absolute -inset-14"
              aria-hidden="true"
            ></div>
            <img
              src="/images/nutorch-hero.png"
              srcSet="/images/nutorch-hero.png 1x, /images/nutorch-hero@2x.png 2x"
              alt="The NuTorch logo: a green nautilus shell with a flame"
              width="300"
              height="300"
              className="relative w-52 sm:w-72"
            />
          </div>
          <div className="w-full min-w-0 flex-1 text-center sm:w-auto sm:text-left">
            <h1 className="font-heading text-4xl font-bold tracking-wider text-balance text-primary uppercase sm:text-5xl">
              A shell for GPU computing
            </h1>
            <p className="mt-4 text-lg text-muted">
              Work in a full Nushell-based shell, from everyday file commands
              and structured pipelines to native GPU tensors and neural
              networks. Powered by LibTorch on Metal for Apple-silicon macOS.
            </p>
            <div className="mt-6 text-left">
              <p className="mb-2 text-sm text-muted">Inside NuTorch</p>
              <CodeBlock blocks={loaderData} code={heroDemo} />
            </div>
            <p className="mt-3 text-sm text-muted">
              Unreleased development shell, powered by LibTorch on Metal. Use
              the{" "}
              <Link
                to={`${href("/docs/*", { "*": "nushell/" })}#setup`}
                className="text-primary underline"
              >
                local shell setup
              </Link>{" "}
              before trying these examples; they require the new local build.
            </p>
            <div className="mt-6 flex flex-wrap items-center justify-center gap-4 sm:justify-start">
              <Button asChild size="lg">
                <a href="#install">Try the development shell</a>
              </Button>
              <Button asChild size="lg" variant="secondary">
                <a href="https://github.com/astrohackerlabs/nutorch">GitHub</a>
              </Button>
            </div>
          </div>
        </div>
      </section>

      <section
        id="install"
        className="mx-auto max-w-5xl scroll-mt-8 px-6 py-12"
      >
        <h2 className="font-display text-2xl font-bold tracking-tight">
          Launch NuTorch, then import torch
        </h2>
        <p className="mt-2 text-muted">
          The native shell is unreleased. The current Homebrew package is the
          earlier tensor-tool release. Follow the{" "}
          <Link
            to={href("/docs/*", { "*": "install-from-source/" })}
            className="text-primary underline"
          >
            development build instructions
          </Link>
          , then launch it from the Astrohacker checkout in your current
          terminal:
        </p>
        <div className="relative mt-6">
          <CodeBlock blocks={loaderData} code={launchDemo} />
          <Button
            id="copy-install"
            type="button"
            aria-label="Copy launch command"
            data-copy={launchDemo}
            size="sm"
            variant="secondary"
            className="absolute top-3 right-3"
          >
            copy
          </Button>
        </div>
        <p className="mt-4 text-muted">
          At the new NuTorch prompt, type <code>use torch</code> to enable the
          native tensor commands shown above. Ordinary shell commands work
          immediately, without this import. Each standalone tensor example
          includes its own import so you can run it in a fresh session.
        </p>
      </section>

      <section className="mx-auto max-w-5xl px-6 py-12">
        <h2 className="font-display text-2xl font-bold tracking-tight">
          Shell-native tensors
        </h2>
        <p className="mt-2 text-muted">
          Each command calls the native tensor core in the shell process.
          Results are shared values, so tensor operations compose naturally in
          structured pipelines.
        </p>
        <p className="mt-2 text-muted">
          Follow the{" "}
          <Link
            to={`${href("/docs/*", { "*": "nushell/" })}#setup`}
            className="text-primary underline"
          >
            NuTorch setup instructions
          </Link>{" "}
          to launch the local shell, then enter <code>use torch</code>.
        </p>
        <div className="mt-6">
          <div className="min-w-0">
            <h3 className="mb-3 font-mono text-sm font-medium text-muted">
              Matrix multiplication inside NuTorch
            </h3>
            <CodeBlock blocks={loaderData} code={nuDemo} />
          </div>
        </div>
        <div className="mt-6">
          <h3 className="mb-3 font-mono text-sm font-medium text-muted">
            Autograd
          </h3>
          <CodeBlock blocks={loaderData} code={trainDemo} />
        </div>
      </section>

      <section className="mx-auto max-w-5xl px-6 py-12 pb-20">
        <div className="grid gap-5 sm:grid-cols-2">
          {features.map((f) => (
            <Card
              key={f.title}
              className="gap-0 border-border bg-background/70 p-6 backdrop-blur-sm"
            >
              <h3 className="font-display text-lg font-bold">{f.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                {f.body}
              </p>
            </Card>
          ))}
        </div>
      </section>
    </>
  );
}
