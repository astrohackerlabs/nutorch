import {
  useEffect,
  useRef,
  type ComponentProps,
  type ReactElement,
} from "react";

import { mountSpaceRain } from "./space-rain/mount";

export type SpaceRainProps = Omit<
  ComponentProps<"canvas">,
  "aria-hidden" | "children" | "ref"
>;

/** Shared procedural Austin Night background. */
export function SpaceRain({
  className = "pointer-events-none fixed inset-0 z-1 block h-dvh w-dvw",
  ...props
}: SpaceRainProps = {}): ReactElement {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const owner = new AbortController();
    void mountSpaceRain(canvas, owner.signal);
    return (): void => {
      owner.abort();
    };
  }, []);

  return (
    <canvas
      {...props}
      ref={canvasRef}
      data-space-rain-canvas
      aria-hidden="true"
      className={className}
    />
  );
}
