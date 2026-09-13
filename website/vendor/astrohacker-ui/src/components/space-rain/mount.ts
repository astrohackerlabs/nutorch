interface SpaceRainRuntime {
  installSpaceRainCanvas(canvas: HTMLCanvasElement): AbortController;
}

type SpaceRainRuntimeLoader = () => Promise<SpaceRainRuntime>;

const loadSpaceRainRuntime: SpaceRainRuntimeLoader = () => import("./runtime");

/**
 * Starts one browser-only renderer unless its owner was already disposed.
 * Exported for lifecycle tests; product code should render the SpaceRain component.
 */
export async function mountSpaceRain(
  canvas: HTMLCanvasElement,
  signal: AbortSignal,
  loadRuntime: SpaceRainRuntimeLoader = loadSpaceRainRuntime,
): Promise<AbortController | undefined> {
  try {
    const runtime = await loadRuntime();
    if (signal.aborted) return undefined;

    const renderer = runtime.installSpaceRainCanvas(canvas);
    if (isAborted(signal)) {
      renderer.abort();
      return undefined;
    }
    signal.addEventListener(
      "abort",
      () => {
        renderer.abort();
      },
      { once: true },
    );
    return renderer;
  } catch (error) {
    if (!signal.aborted) {
      canvas.dataset.spaceRainStatus = "fallback";
      console.warn(
        "SpaceRain runtime unavailable; retaining page background",
        error,
      );
    }
    return undefined;
  }
}

// Runtime installation can synchronously abort the owner; read its live state.
function isAborted(signal: AbortSignal): boolean {
  return signal.aborted;
}
