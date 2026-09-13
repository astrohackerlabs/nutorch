import {
  objectIdentity,
  randomFor,
  type PageSeed,
  type SeededRandom,
} from "./seed";

export type BlackHoleVec2 = readonly [number, number];
export type BlackHoleIdentity = `black-hole:${string}`;

export interface BlackHoleAppearance {
  mass: number;
  shadowRadius: number;
  influenceRadius: number;
  deflectionStrength: number;
  criticalRadius: number;
  criticalWidth: number;
  magnification: number;
  frameDragTwist: number;
  handedness: -1 | 1;
  phase: number;
}

export interface BlackHoleTrajectory {
  start: BlackHoleVec2;
  end: BlackHoleVec2;
  drift: number;
  fallSpeed: number;
  duration: number;
}

export interface BlackHoleEncounter {
  identity: BlackHoleIdentity;
  ordinal: bigint;
  visualSignature: string;
  appearance: BlackHoleAppearance;
  trajectory: BlackHoleTrajectory;
  depth: number;
  size: number;
}

export interface BlackHoleState {
  kind: "active" | "complete";
  position: BlackHoleVec2;
  velocity: BlackHoleVec2;
  progress: number;
  localTime: number;
}

export interface BlackHoleLensSample {
  sourceUv: BlackHoleVec2;
  radius: number;
  insideShadow: boolean;
  gain: number;
}

export interface BlackHoleScheduler {
  pageSeed: PageSeed;
  reviewMode: boolean;
  reducedMotion: boolean;
  nextOrdinal: bigint;
  active: BlackHoleEncounter | null;
  activeElapsed: number;
  waitRemaining: number;
  completedCount: number;
}

export const BLACK_HOLE_MASS_RANGE = [0.72, 1.38] as const;
export const BLACK_HOLE_SHADOW_RADIUS_RANGE = [0.9, 1.12] as const;
export const BLACK_HOLE_INFLUENCE_RADIUS_RANGE = [3.4, 5.2] as const;
export const BLACK_HOLE_DEFLECTION_RANGE = [0.72, 1.18] as const;
export const BLACK_HOLE_CRITICAL_RADIUS_RANGE = [1.45, 2.05] as const;
export const BLACK_HOLE_CRITICAL_WIDTH_RANGE = [0.09, 0.22] as const;
export const BLACK_HOLE_MAGNIFICATION_RANGE = [0.42, 1.08] as const;
export const BLACK_HOLE_TWIST_RANGE = [0.08, 0.34] as const;
export const BLACK_HOLE_DEPTH_RANGE = [0.08, 1] as const;
export const BLACK_HOLE_SIZE_RANGE = [24, 62] as const;
export const BLACK_HOLE_DRIFT_RANGE = [-0.2, 0.2] as const;
export const BLACK_HOLE_NORMAL_DURATION_RANGE = [12, 24] as const;
export const BLACK_HOLE_REVIEW_DURATION_RANGE = [5, 8] as const;
export const BLACK_HOLE_NORMAL_FIRST_DELAY_RANGE = [8, 16] as const;
export const BLACK_HOLE_NORMAL_GAP_RANGE = [18, 40] as const;
export const BLACK_HOLE_REVIEW_FIRST_DELAY_RANGE = [0.1, 0.4] as const;
export const BLACK_HOLE_REVIEW_GAP_RANGE = [0.5, 1.5] as const;

const START_Y = 1.28;
const END_Y = -1.28;
const PI2 = Math.PI * 2;

function mix(first: number, second: number, amount: number): number {
  return first + (second - first) * amount;
}

function sampleRange(
  random: SeededRandom,
  range: readonly [number, number],
): number {
  return mix(range[0], range[1], random.next());
}

function smoothstep(edge0: number, edge1: number, value: number): number {
  const amount = Math.min(1, Math.max(0, (value - edge0) / (edge1 - edge0)));
  return amount * amount * (3 - 2 * amount);
}

export function blackHoleLensSample(options: {
  uv: BlackHoleVec2;
  resolution: BlackHoleVec2;
  position: BlackHoleVec2;
  shadowPixels: number;
  appearance: BlackHoleAppearance;
  sourceRadiance?: number;
}): BlackHoleLensSample {
  const { uv, resolution, position, appearance } = options;
  const center: BlackHoleVec2 = [
    position[0] * 0.5 + 0.5,
    position[1] * 0.5 + 0.5,
  ];
  const deltaPixels: BlackHoleVec2 = [
    (uv[0] - center[0]) * resolution[0],
    (uv[1] - center[1]) * resolution[1],
  ];
  const distancePixels = Math.hypot(...deltaPixels);
  const shadowPixels = Math.max(options.shadowPixels, 1);
  const radius = distancePixels / shadowPixels;
  if (radius >= appearance.influenceRadius) {
    return { sourceUv: uv, radius, insideShadow: false, gain: 1 };
  }
  const antialiasWidth = 1.25 / shadowPixels;
  if (radius <= 1 - antialiasWidth) {
    return { sourceUv: center, radius, insideShadow: true, gain: 1 };
  }
  const safeRadius = Math.max(radius, 0.35);
  const edgeStart = Math.max(
    appearance.criticalRadius + appearance.criticalWidth * 2,
    appearance.influenceRadius * 0.72,
  );
  const edgeWeight =
    1 - smoothstep(edgeStart, appearance.influenceRadius, radius);
  const inverseImpact =
    (appearance.criticalRadius ** 2 *
      appearance.deflectionStrength *
      appearance.mass) /
    safeRadius;
  const sourceRadius = radius + (radius - inverseImpact - radius) * edgeWeight;
  const baseDirection: BlackHoleVec2 =
    distancePixels > 0.0001
      ? [deltaPixels[0] / distancePixels, deltaPixels[1] / distancePixels]
      : [1, 0];
  const twist =
    (appearance.frameDragTwist *
      appearance.handedness *
      edgeWeight *
      (0.9 + 0.1 * Math.sin(appearance.phase))) /
    Math.max(radius * radius, 0.6);
  const sine = Math.sin(twist);
  const cosine = Math.cos(twist);
  const direction: BlackHoleVec2 = [
    cosine * baseDirection[0] - sine * baseDirection[1],
    sine * baseDirection[0] + cosine * baseDirection[1],
  ];
  const sourceUv: BlackHoleVec2 = [
    Math.min(
      1,
      Math.max(
        0,
        center[0] +
          (direction[0] * sourceRadius * shadowPixels) / resolution[0],
      ),
    ),
    Math.min(
      1,
      Math.max(
        0,
        center[1] +
          (direction[1] * sourceRadius * shadowPixels) / resolution[1],
      ),
    ),
  ];
  const criticalOffset =
    (radius - appearance.criticalRadius) /
    Math.max(appearance.criticalWidth, 0.02);
  const criticalBand = Math.exp(-Math.pow(criticalOffset, 2));
  const sourceRadiance = Math.min(1, Math.max(0, options.sourceRadiance ?? 0));
  const gain =
    1 +
    criticalBand *
      appearance.magnification *
      smoothstep(0.015, 0.24, sourceRadiance);
  return { sourceUv, radius, insideShadow: false, gain };
}

function appearanceSignature(appearance: BlackHoleAppearance): string {
  return Object.values(appearance)
    .map((value) => Number(value).toFixed(9))
    .join(":");
}

export function createBlackHoleEncounter(
  pageSeed: PageSeed,
  ordinal: bigint,
  reviewMode = false,
): BlackHoleEncounter {
  const lens = randomFor(pageSeed, ordinal, "black-hole-lens");
  const placement = randomFor(pageSeed, ordinal, "black-hole-placement");
  const motion = randomFor(pageSeed, ordinal, "black-hole-motion");
  const depth = sampleRange(placement, BLACK_HOLE_DEPTH_RANGE);
  const size = mix(BLACK_HOLE_SIZE_RANGE[0], BLACK_HOLE_SIZE_RANGE[1], depth);
  const durationRange = reviewMode
    ? BLACK_HOLE_REVIEW_DURATION_RANGE
    : BLACK_HOLE_NORMAL_DURATION_RANGE;
  const duration = mix(durationRange[1], durationRange[0], depth);
  const entranceX = sampleRange(placement, [-0.82, 0.82]);
  const drift = sampleRange(motion, BLACK_HOLE_DRIFT_RANGE);
  const appearance: BlackHoleAppearance = {
    mass: sampleRange(lens, BLACK_HOLE_MASS_RANGE),
    shadowRadius: sampleRange(lens, BLACK_HOLE_SHADOW_RADIUS_RANGE),
    influenceRadius: sampleRange(lens, BLACK_HOLE_INFLUENCE_RADIUS_RANGE),
    deflectionStrength: sampleRange(lens, BLACK_HOLE_DEFLECTION_RANGE),
    criticalRadius: sampleRange(lens, BLACK_HOLE_CRITICAL_RADIUS_RANGE),
    criticalWidth: sampleRange(lens, BLACK_HOLE_CRITICAL_WIDTH_RANGE),
    magnification: sampleRange(lens, BLACK_HOLE_MAGNIFICATION_RANGE),
    frameDragTwist: sampleRange(lens, BLACK_HOLE_TWIST_RANGE),
    handedness: lens.next() < 0.5 ? -1 : 1,
    phase: motion.next() * PI2,
  };
  const trajectory: BlackHoleTrajectory = {
    start: [entranceX, START_Y],
    end: [entranceX + drift, END_Y],
    drift,
    fallSpeed: (START_Y - END_Y) / duration,
    duration,
  };
  const identity = `black-hole:${objectIdentity(pageSeed, ordinal)}` as const;
  return {
    identity,
    ordinal,
    visualSignature: [
      appearanceSignature(appearance),
      depth.toFixed(9),
      size.toFixed(9),
      ...trajectory.start.map((value) => value.toFixed(9)),
      ...trajectory.end.map((value) => value.toFixed(9)),
      duration.toFixed(9),
    ].join(":"),
    appearance,
    trajectory,
    depth,
    size,
  };
}

export function evaluateBlackHoleEncounter(
  encounter: BlackHoleEncounter,
  elapsedSeconds: number,
): BlackHoleState {
  const localTime = Math.min(
    Math.max(0, elapsedSeconds),
    encounter.trajectory.duration,
  );
  const progress = localTime / encounter.trajectory.duration;
  const { start, end, duration } = encounter.trajectory;
  return {
    kind: progress >= 1 ? "complete" : "active",
    position: [
      mix(start[0], end[0], progress),
      mix(start[1], end[1], progress),
    ],
    velocity:
      progress >= 1
        ? [0, 0]
        : [(end[0] - start[0]) / duration, (end[1] - start[1]) / duration],
    progress,
    localTime,
  };
}

function gapFor(
  scheduler: Pick<BlackHoleScheduler, "pageSeed" | "reviewMode">,
  ordinal: bigint,
  initial: boolean,
): number {
  const random = randomFor(
    scheduler.pageSeed,
    ordinal,
    initial ? "black-hole-first-delay" : "black-hole-gap",
  );
  const range = initial
    ? scheduler.reviewMode
      ? BLACK_HOLE_REVIEW_FIRST_DELAY_RANGE
      : BLACK_HOLE_NORMAL_FIRST_DELAY_RANGE
    : scheduler.reviewMode
      ? BLACK_HOLE_REVIEW_GAP_RANGE
      : BLACK_HOLE_NORMAL_GAP_RANGE;
  return sampleRange(random, range);
}

export function createBlackHoleScheduler(options: {
  pageSeed: PageSeed;
  reviewMode: boolean;
  reducedMotion: boolean;
}): BlackHoleScheduler {
  const scheduler: BlackHoleScheduler = {
    pageSeed: options.pageSeed,
    reviewMode: options.reviewMode,
    reducedMotion: options.reducedMotion,
    nextOrdinal: 0n,
    active: null,
    activeElapsed: 0,
    waitRemaining: 0,
    completedCount: 0,
  };
  scheduler.waitRemaining = gapFor(scheduler, 0n, true);
  return scheduler;
}

export function setBlackHoleReducedMotion(
  scheduler: BlackHoleScheduler,
  reducedMotion: boolean,
): void {
  scheduler.reducedMotion = reducedMotion;
  if (!reducedMotion) return;
  scheduler.active = null;
  scheduler.activeElapsed = 0;
  scheduler.waitRemaining = gapFor(scheduler, scheduler.nextOrdinal, true);
}

export function advanceBlackHoleScheduler(
  scheduler: BlackHoleScheduler,
  elapsedSeconds: number,
): void {
  if (
    scheduler.reducedMotion ||
    !Number.isFinite(elapsedSeconds) ||
    elapsedSeconds <= 0
  ) {
    return;
  }
  let remaining = elapsedSeconds;
  for (
    let transition = 0;
    transition < 64 && remaining > 1e-9;
    transition += 1
  ) {
    if (!scheduler.active) {
      if (remaining < scheduler.waitRemaining) {
        scheduler.waitRemaining -= remaining;
        return;
      }
      remaining -= scheduler.waitRemaining;
      scheduler.active = createBlackHoleEncounter(
        scheduler.pageSeed,
        scheduler.nextOrdinal,
        scheduler.reviewMode,
      );
      scheduler.nextOrdinal += 1n;
      scheduler.activeElapsed = 0;
      scheduler.waitRemaining = 0;
      continue;
    }
    const activeRemaining =
      scheduler.active.trajectory.duration - scheduler.activeElapsed;
    if (remaining < activeRemaining) {
      scheduler.activeElapsed += remaining;
      return;
    }
    remaining -= activeRemaining;
    scheduler.completedCount += 1;
    scheduler.active = null;
    scheduler.activeElapsed = 0;
    scheduler.waitRemaining = gapFor(
      scheduler,
      scheduler.nextOrdinal - 1n,
      false,
    );
  }
}

export function currentBlackHoleState(
  scheduler: BlackHoleScheduler,
): BlackHoleState | null {
  return scheduler.active
    ? evaluateBlackHoleEncounter(scheduler.active, scheduler.activeElapsed)
    : null;
}
