import {
  useEffect,
  useRef,
  useState,
  type ComponentProps,
  type ReactElement,
} from "react";

import { mountSpaceRain } from "./space-rain/mount";
import { createMotionModeSource, type MotionMode } from "./motion-mode/state";
export type { SpaceRainDiagnostics } from "./space-rain/runtime";

export type SpaceRainProps = Omit<
  ComponentProps<"canvas">,
  "aria-hidden" | "children" | "ref"
> & { motionMode?: MotionMode };

/** Shared procedural Austin Night background. */
export function SpaceRain({
  className = "pointer-events-none fixed inset-0 z-1 block h-dvh w-dvw",
  motionMode = "system",
  ...props
}: SpaceRainProps = {}): ReactElement | null {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [motion] = useState(() => createMotionModeSource(motionMode));
  const shown = motionMode !== "no-graphics";

  useEffect(() => {
    motion.setMode(motionMode);
  }, [motion, motionMode]);

  useEffect(() => {
    if (!shown) return;
    const canvas = canvasRef.current;
    if (!canvas) return;

    const owner = new AbortController();
    void mountSpaceRain(canvas, owner.signal, undefined, motion);
    return (): void => {
      owner.abort();
    };
  }, [motion, shown]);

  if (!shown) return null;

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
