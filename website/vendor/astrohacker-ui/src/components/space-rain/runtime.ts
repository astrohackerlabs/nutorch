import {
  DEFAULT_SPACE_RAIN_CONFIG,
  reviewModeFromSearch,
  type ReviewMode,
} from "./config";
import {
  advanceScene,
  createScene,
  resizeScene,
  seedClassHistogram,
  setReducedMotion,
  type SpaceRainScene,
} from "./model";
import { SpaceRainRenderer, type ResourceCounts } from "./renderer";
import { validPlanetRotation } from "./planet-rotation";
import { validVolumetricMotion } from "./volumetric-motion";
import { currentBlackHoleState } from "./black-hole";
import {
  currentUfoState,
  ufoTraversalClassFromSearch,
  type UfoTraversalClass,
} from "./ufo";
import { freshPageSeed, pageSeedFromHex, type PageSeed } from "./seed";
import {
  resolveReducedMotion,
  type MotionModeSource,
} from "../motion-mode/state";

type RainRenderer = Pick<
  SpaceRainRenderer,
  "resources" | "shaderStatus" | "render" | "resize" | "dispose"
>;
type RendererFactory = (canvas: HTMLCanvasElement) => RainRenderer;
const createRenderer: RendererFactory = (canvas) =>
  new SpaceRainRenderer(canvas);

export interface SpaceRainDiagnostics {
  status: "idle" | "running" | "paused" | "fallback" | "disposed";
  shaderStatus: readonly string[];
  frameIntervals: readonly number[];
  frameCount: number;
  drawCalls: number;
  instanceCount: number;
  recycleCount: number;
  pageSeedIdentifier: string | null;
  highestSpawnOrdinal: string | null;
  activeIdentityCount: number;
  activeVisualSignatureCount: number;
  seedClassCounts: Readonly<Record<string, number>>;
  planetRotationCount: number;
  invalidPlanetRotationCount: number;
  planetSpinDirections: Readonly<{ negative: number; positive: number }>;
  planetRotationIdentityChecksum: string;
  planetRotationChecksum: string;
  volumetricMotionCount: number;
  invalidVolumetricMotionCount: number;
  volumetricSpinDirections: Readonly<{ negative: number; positive: number }>;
  volumetricMotionIdentityChecksum: string;
  volumetricMotionChecksum: string;
  ufoSchedulerState: "suppressed" | "waiting" | "active";
  ufoEncounterOrdinal: string | null;
  ufoIdentity: string | null;
  ufoVisualSignature: string | null;
  ufoSegmentKind: "travel" | "pause" | "complete" | null;
  ufoSegmentIndex: number | null;
  ufoTurnCount: number | null;
  ufoPauseCount: number | null;
  ufoTraversalClass: UfoTraversalClass | null;
  ufoCameraPosition: readonly [number, number, number] | null;
  ufoCameraVelocity: readonly [number, number, number] | null;
  ufoProjectedPosition: readonly [number, number] | null;
  ufoProjectedVelocity: readonly [number, number] | null;
  ufoProjectedRadius: number | null;
  ufoOrientation: readonly [number, number, number] | null;
  ufoCompletedCount: number;
  blackHoleSchedulerState: "suppressed" | "waiting" | "active";
  blackHoleEncounterOrdinal: string | null;
  blackHoleIdentity: string | null;
  blackHoleVisualSignature: string | null;
  blackHolePosition: readonly [number, number] | null;
  blackHoleVelocity: readonly [number, number] | null;
  blackHoleShadowRadius: number | null;
  blackHoleInfluenceRadius: number | null;
  blackHoleMass: number | null;
  blackHoleStrength: number | null;
  blackHoleSpin: number | null;
  blackHoleHandedness: -1 | 1 | null;
  blackHoleCompletedCount: number;
  offscreenActive: boolean;
  offscreenBytes: number;
  resources: Readonly<ResourceCounts>;
  reviewMode: string | null;
  reducedMotion: boolean;
}

declare global {
  interface Window {
    __spaceRainDiagnostics?: SpaceRainDiagnostics;
  }
}

const EMPTY_RESOURCES: ResourceCounts = {
  contexts: 0,
  programs: 0,
  buffers: 0,
  vertexArrays: 0,
  textures: 0,
  framebuffers: 0,
};

function installCanvas(
  canvas: HTMLCanvasElement,
  pageSeed: PageSeed,
  reviewMode: ReviewMode | null,
  ufoTraversalClass: UfoTraversalClass | null,
  motion?: MotionModeSource,
  rendererFactory: RendererFactory = createRenderer,
): AbortController {
  const abortController = new AbortController();
  const { signal } = abortController;
  const reducedQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  const resolveMotion = (): boolean =>
    resolveReducedMotion(motion?.getMode() ?? "system", reducedQuery.matches);
  let reducedMotion = resolveMotion();
  let renderer: RainRenderer | null = null;
  let scene: SpaceRainScene | null = null;
  let frameRequest = 0;
  let lastTime = 0;
  let visualTime = 0;
  let status: SpaceRainDiagnostics["status"] = "idle";
  let drawCalls = 0;
  let instanceCount = 0;
  let frameCount = 0;
  const frameIntervals: number[] = [];
  let shaderStatus: readonly string[] = [];
  let offscreenActive = false;
  let offscreenBytes = 0;
  canvas.dataset.spaceRainStatus = "idle";

  const diagnostics = (): SpaceRainDiagnostics => {
    const instances = scene?.instances ?? [];
    const planetRotations = instances.flatMap((instance) =>
      instance.planetRotation ? [instance.planetRotation] : [],
    );
    const volumetricMotions = instances.flatMap((instance) =>
      instance.volumetricMotion ? [instance.volumetricMotion] : [],
    );
    const scheduler = scene?.ufoScheduler ?? null;
    const encounter = scheduler?.active ?? null;
    const ufoState = scheduler ? currentUfoState(scheduler) : null;
    const blackHoleScheduler = scene?.blackHoleScheduler ?? null;
    const blackHoleEncounter = blackHoleScheduler?.active ?? null;
    const blackHoleState = blackHoleScheduler
      ? currentBlackHoleState(blackHoleScheduler)
      : null;
    const highestOrdinal = scene ? scene.nextSpawnOrdinal - 1n : null;
    return Object.freeze({
      status,
      shaderStatus: Object.freeze([...shaderStatus]),
      frameIntervals: Object.freeze([...frameIntervals]),
      frameCount,
      drawCalls,
      instanceCount,
      recycleCount: scene?.recycleCount ?? 0,
      pageSeedIdentifier: scene?.pageSeedId ?? null,
      highestSpawnOrdinal:
        highestOrdinal === null || highestOrdinal < 0n
          ? null
          : highestOrdinal.toString(),
      activeIdentityCount: new Set(
        instances.map((instance) => instance.identity),
      ).size,
      activeVisualSignatureCount: new Set(
        instances.map((instance) => instance.visualSignature),
      ).size,
      seedClassCounts: Object.freeze(seedClassHistogram(instances)),
      planetRotationCount: planetRotations.length,
      invalidPlanetRotationCount: planetRotations.filter(
        (rotation) => !validPlanetRotation(rotation),
      ).length,
      planetSpinDirections: Object.freeze({
        negative: planetRotations.filter((rotation) => rotation.surfaceRate < 0)
          .length,
        positive: planetRotations.filter((rotation) => rotation.surfaceRate > 0)
          .length,
      }),
      planetRotationIdentityChecksum: instances
        .flatMap((instance) =>
          instance.planetRotation ? [instance.identity] : [],
        )
        .join("|"),
      planetRotationChecksum: instances
        .flatMap((instance) =>
          instance.planetRotation
            ? [
                [
                  instance.identity,
                  instance.planetRotation.surfaceAngle.toFixed(8),
                  instance.planetRotation.cloudAngle.toFixed(8),
                  instance.planetRotation.ringAngle.toFixed(8),
                ].join(":"),
              ]
            : [],
        )
        .join("|"),
      volumetricMotionCount: volumetricMotions.length,
      invalidVolumetricMotionCount: volumetricMotions.filter(
        (motion) => !validVolumetricMotion(motion),
      ).length,
      volumetricSpinDirections: Object.freeze({
        negative: volumetricMotions.filter((motion) => motion.bulkRate < 0)
          .length,
        positive: volumetricMotions.filter((motion) => motion.bulkRate > 0)
          .length,
      }),
      volumetricMotionIdentityChecksum: instances
        .flatMap((instance) =>
          instance.volumetricMotion ? [instance.identity] : [],
        )
        .join("|"),
      volumetricMotionChecksum: instances
        .flatMap((instance) =>
          instance.volumetricMotion
            ? [
                [
                  instance.identity,
                  instance.volumetricMotion.bulkAngle.toFixed(8),
                  instance.volumetricMotion.internalAngle.toFixed(8),
                  instance.volumetricMotion.evolutionAngle.toFixed(8),
                ].join(":"),
              ]
            : [],
        )
        .join("|"),
      ufoSchedulerState: scheduler?.reducedMotion
        ? "suppressed"
        : encounter
          ? "active"
          : "waiting",
      ufoEncounterOrdinal: encounter?.ordinal.toString() ?? null,
      ufoIdentity: encounter?.identity ?? null,
      ufoVisualSignature: encounter?.visualSignature ?? null,
      ufoSegmentKind: ufoState?.kind ?? null,
      ufoSegmentIndex: ufoState?.segmentIndex ?? null,
      ufoTurnCount: encounter?.trajectory.turnCount ?? null,
      ufoPauseCount: encounter?.trajectory.pauseCount ?? null,
      ufoTraversalClass: encounter?.trajectory.traversalClass ?? null,
      ufoCameraPosition: ufoState?.cameraPosition ?? null,
      ufoCameraVelocity: ufoState?.cameraVelocity ?? null,
      ufoProjectedPosition: ufoState?.projectedPosition ?? null,
      ufoProjectedVelocity: ufoState?.projectedVelocity ?? null,
      ufoProjectedRadius: ufoState?.projectedRadius ?? null,
      ufoOrientation: ufoState
        ? ([ufoState.yaw, ufoState.pitch, ufoState.roll] as const)
        : null,
      ufoCompletedCount: scheduler?.completedCount ?? 0,
      blackHoleSchedulerState: blackHoleScheduler?.reducedMotion
        ? "suppressed"
        : blackHoleEncounter
          ? "active"
          : "waiting",
      blackHoleEncounterOrdinal: blackHoleEncounter?.ordinal.toString() ?? null,
      blackHoleIdentity: blackHoleEncounter?.identity ?? null,
      blackHoleVisualSignature: blackHoleEncounter?.visualSignature ?? null,
      blackHolePosition: blackHoleState?.position ?? null,
      blackHoleVelocity: blackHoleState?.velocity ?? null,
      blackHoleShadowRadius: blackHoleEncounter
        ? blackHoleEncounter.size * blackHoleEncounter.appearance.shadowRadius
        : null,
      blackHoleInfluenceRadius:
        blackHoleEncounter?.appearance.influenceRadius ?? null,
      blackHoleMass: blackHoleEncounter?.appearance.mass ?? null,
      blackHoleStrength:
        blackHoleEncounter?.appearance.deflectionStrength ?? null,
      blackHoleSpin: blackHoleEncounter?.appearance.frameDragTwist ?? null,
      blackHoleHandedness: blackHoleEncounter?.appearance.handedness ?? null,
      blackHoleCompletedCount: blackHoleScheduler?.completedCount ?? 0,
      offscreenActive,
      offscreenBytes,
      resources: Object.freeze({ ...(renderer?.resources ?? EMPTY_RESOURCES) }),
      reviewMode,
      reducedMotion,
    });
  };

  Object.defineProperty(window, "__spaceRainDiagnostics", {
    configurable: true,
    get: diagnostics,
  });

  const showFallback = (): void => {
    status = "fallback";
    canvas.dataset.spaceRainStatus = "fallback";
  };

  const draw = (): void => {
    if (!renderer || !scene) return;
    const report = renderer.render(scene, visualTime);
    drawCalls = report.drawCalls;
    instanceCount = report.instanceCount;
    offscreenActive = report.offscreenActive;
    offscreenBytes = report.offscreenBytes;
  };

  const resize = (redraw = true): void => {
    if (!renderer || !scene) return;
    const bounds = canvas.getBoundingClientRect();
    const width = Math.max(1, bounds.width);
    const height = Math.max(1, bounds.height);
    const dpr = Math.min(
      window.devicePixelRatio || 1,
      DEFAULT_SPACE_RAIN_CONFIG.maxDpr,
    );
    renderer.resize(width, height, dpr);
    resizeScene(scene, width, height);
    if (redraw && (reducedMotion || document.hidden)) draw();
  };

  const frame = (time: number): void => {
    frameRequest = 0;
    if (
      !renderer ||
      !scene ||
      signal.aborted ||
      document.hidden ||
      reducedMotion
    )
      return;
    const elapsed =
      lastTime === 0 ? 0 : Math.min(0.05, (time - lastTime) / 1_000);
    if (lastTime > 0) {
      frameIntervals.push(time - lastTime);
      if (frameIntervals.length > 3_000) frameIntervals.shift();
    }
    lastTime = time;
    visualTime += elapsed;
    advanceScene(scene, elapsed);
    draw();
    frameCount += 1;
    status = "running";
    frameRequest = requestAnimationFrame(frame);
  };

  const start = (): void => {
    if (signal.aborted || renderer) return;
    try {
      renderer = rendererFactory(canvas);
      shaderStatus = renderer.shaderStatus;
      const bounds = canvas.getBoundingClientRect();
      scene ??= createScene({
        pageSeed,
        width: Math.max(1, bounds.width),
        height: Math.max(1, bounds.height),
        reducedMotion,
        reviewMode,
        ufoTraversalClass,
      });
      resize(false);
      draw();
      status = reducedMotion || document.hidden ? "paused" : "running";
      canvas.dataset.spaceRainStatus = status;
      if (!reducedMotion && !document.hidden) {
        frameRequest = requestAnimationFrame(frame);
      }
    } catch (error) {
      console.warn("SpaceRain unavailable; retaining page background", error);
      renderer?.dispose(false);
      renderer = null;
      showFallback();
    }
  };

  const stopFrame = (): void => {
    cancelAnimationFrame(frameRequest);
    frameRequest = 0;
    lastTime = 0;
  };

  const onVisibility = (): void => {
    if (document.hidden) {
      stopFrame();
      if (renderer) {
        status = "paused";
        canvas.dataset.spaceRainStatus = status;
      }
    } else if (renderer && scene && !reducedMotion && frameRequest === 0) {
      status = "running";
      canvas.dataset.spaceRainStatus = status;
      frameRequest = requestAnimationFrame(frame);
    }
  };

  const onReducedMotion = (): void => {
    const next = resolveMotion();
    if (next === reducedMotion) return;
    reducedMotion = next;
    stopFrame();
    if (!scene) return;
    setReducedMotion(scene, reducedMotion);
    if (!renderer) return;
    resize(false);
    draw();
    status = reducedMotion || document.hidden ? "paused" : "running";
    canvas.dataset.spaceRainStatus = status;
    if (!reducedMotion && !document.hidden) {
      frameRequest = requestAnimationFrame(frame);
    }
  };

  const onContextLost = (event: Event): void => {
    event.preventDefault();
    stopFrame();
    renderer?.dispose(false);
    renderer = null;
    offscreenActive = false;
    offscreenBytes = 0;
    showFallback();
  };

  const onContextRestored = (): void => {
    start();
  };
  const resizeObserver = new ResizeObserver(() => {
    resize();
  });
  resizeObserver.observe(canvas);
  document.addEventListener("visibilitychange", onVisibility, { signal });
  reducedQuery.addEventListener("change", onReducedMotion, { signal });
  const unsubscribeMotion = motion?.subscribe(onReducedMotion);
  canvas.addEventListener("webglcontextlost", onContextLost, { signal });
  canvas.addEventListener("webglcontextrestored", onContextRestored, {
    signal,
  });
  signal.addEventListener(
    "abort",
    () => {
      stopFrame();
      unsubscribeMotion?.();
      resizeObserver.disconnect();
      renderer?.dispose();
      renderer = null;
      scene = null;
      offscreenActive = false;
      offscreenBytes = 0;
      status = "disposed";
      canvas.dataset.spaceRainStatus = status;
    },
    { once: true },
  );
  start();
  return abortController;
}

export function installSpaceRainCanvas(
  canvas: HTMLCanvasElement,
  explicitPageSeed?: PageSeed,
  motion?: MotionModeSource,
  rendererFactory: RendererFactory = createRenderer,
): AbortController {
  const search = window.location.search;
  const reviewMode = reviewModeFromSearch(search);
  const ufoTraversalClass = ufoTraversalClassFromSearch(search, reviewMode);
  const pageSeed = resolvePageSeed(search, explicitPageSeed);
  return installCanvas(
    canvas,
    pageSeed,
    reviewMode,
    ufoTraversalClass,
    motion,
    rendererFactory,
  );
}

export function resolvePageSeed(
  search: string,
  explicitPageSeed?: PageSeed,
  fresh: () => PageSeed = freshPageSeed,
): PageSeed {
  if (explicitPageSeed) return explicitPageSeed;
  const reviewMode = reviewModeFromSearch(search);
  const reviewSeed = reviewMode
    ? pageSeedFromHex(new URLSearchParams(search).get("space-rain-seed"))
    : null;
  return reviewSeed ?? fresh();
}
