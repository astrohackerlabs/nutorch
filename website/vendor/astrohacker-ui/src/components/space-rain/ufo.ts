import {
  objectIdentity,
  randomFor,
  type PageSeed,
  type SeededRandom,
} from "./seed";

export type UfoVec2 = readonly [number, number];
export type UfoVec3 = readonly [number, number, number];
export type UfoColor = readonly [number, number, number];
export type UfoIdentity = `ufo:${string}`;
export type UfoPathEdge = "left" | "right" | "top" | "bottom";
export const UFO_TRAVERSAL_CLASSES = [
  "lateral",
  "camera-entry",
  "camera-exit",
  "distance-entry",
  "distance-exit",
] as const;
export type UfoTraversalClass = (typeof UFO_TRAVERSAL_CLASSES)[number];

export interface UfoAppearance {
  hullThickness: number;
  rimBevel: number;
  domeRadius: number;
  domeHeight: number;
  domeTint: number;
  undersideDepth: number;
  metallicity: number;
  roughness: number;
  panelFrequency: number;
  bodyColor: UfoColor;
  trimColor: UfoColor;
  lightCount: number;
  lightOffset: number;
  lightSpacingJitter: number;
  lightColorA: UfoColor;
  lightColorB: UfoColor;
  lightPulsePhase: number;
  lightPulseFrequency: number;
}

export interface UfoLeg {
  kind: "travel";
  startTime: number;
  endTime: number;
  from: UfoVec3;
  to: UfoVec3;
  velocity: UfoVec3;
  heading: number;
  elevation: number;
}

export type UfoTravelLeg = UfoLeg;

export interface UfoPause {
  kind: "pause";
  startTime: number;
  endTime: number;
  position: UfoVec3;
  spinTurns: number;
  spinDirection: -1 | 1;
}

export type UfoSegment = UfoTravelLeg | UfoPause;

export interface UfoTrajectory {
  traversalClass: UfoTraversalClass;
  points: readonly UfoVec3[];
  segments: readonly UfoSegment[];
  turnCount: number;
  pauseCount: number;
  duration: number;
}

export interface UfoEncounter {
  identity: UfoIdentity;
  ordinal: bigint;
  visualSignature: string;
  appearance: UfoAppearance;
  trajectory: UfoTrajectory;
  referenceRadius: number;
  inclination: number;
  initialYaw: number;
  initialRoll: number;
}

export interface UfoState {
  kind: UfoSegment["kind"] | "complete";
  segmentIndex: number;
  cameraPosition: UfoVec3;
  cameraVelocity: UfoVec3;
  projectedPosition: UfoVec2;
  projectedVelocity: UfoVec2;
  projectedRadius: number;
  yaw: number;
  pitch: number;
  roll: number;
  localTime: number;
}

export interface UfoScheduler {
  pageSeed: PageSeed;
  reviewMode: boolean;
  forcedTraversalClass: UfoTraversalClass | null;
  reducedMotion: boolean;
  nextOrdinal: bigint;
  active: UfoEncounter | null;
  activeElapsed: number;
  waitRemaining: number;
  completedCount: number;
}

export const UFO_LIGHT_COUNT_MIN = 6;
export const UFO_LIGHT_COUNT_MAX = 16;
export const UFO_TURN_COUNT_MIN = 1;
export const UFO_TURN_COUNT_MAX = 4;
export const UFO_PAUSE_COUNT_MIN = 0;
export const UFO_PAUSE_COUNT_MAX = 2;
export const UFO_SIZE_MIN = 12;
export const UFO_SIZE_MAX = 64;
export const UFO_TURN_ANGLE_MIN = Math.PI / 6;
export const UFO_TURN_ANGLE_MAX = (Math.PI * 11) / 12;
export const UFO_NORMAL_GAP_RANGE = [2, 5] as const;
export const UFO_REVIEW_GAP_RANGE = [0.25, 1] as const;
export const UFO_FIRST_DELAY_RANGE = [0.12, 0.85] as const;
export const UFO_REVIEW_FIRST_DELAY_RANGE = [0.04, 0.2] as const;
export const UFO_REFERENCE_DEPTH = 1;
export const UFO_NEAR_BOUND_RANGE = [128, 280] as const;
export const UFO_NEAR_DEPTH_RANGE = [
  (UFO_SIZE_MIN * 1.24) / UFO_NEAR_BOUND_RANGE[1],
  (UFO_SIZE_MAX * 1.24) / UFO_NEAR_BOUND_RANGE[0],
] as const;
export const UFO_FAR_BOUND_RANGE = [0.28, 0.55] as const;
export const UFO_PROJECTED_TURN_ANGLE_MIN = Math.PI / 9;
export const UFO_PROJECTED_TURN_ANGLE_MAX = (Math.PI * 17) / 18;
export const UFO_SAFE_VIEWPORT_MIN = 240;
export const UFO_QUAD_EXTENT = 1.24;

const PI2 = Math.PI * 2;
const OUTSIDE = 1.3;
const PATH_INTERIOR_X = 0.9;
const PATH_INTERIOR_Y = 0.78;
const PATH_MIN_LEG = 0.28;
const PATH_MIN_SPAN = 0.82;
const PATH_CANDIDATE_LIMIT = 256;
const PATH_EDGES: readonly UfoPathEdge[] = ["left", "right", "top", "bottom"];

function mix(first: number, second: number, amount: number): number {
  return first + (second - first) * amount;
}

function sampleRange(
  random: SeededRandom,
  range: readonly [number, number],
): number {
  return mix(range[0], range[1], random.next());
}

function fract(value: number): number {
  return value - Math.floor(value);
}

function hueToRgb(p: number, q: number, t: number): number {
  const wrapped = fract(t);
  if (wrapped < 1 / 6) return p + (q - p) * 6 * wrapped;
  if (wrapped < 1 / 2) return q;
  if (wrapped < 2 / 3) return p + (q - p) * (2 / 3 - wrapped) * 6;
  return p;
}

function hsl(hue: number, saturation: number, lightness: number): UfoColor {
  const normalizedHue = fract(hue / 360);
  const q =
    lightness < 0.5
      ? lightness * (1 + saturation)
      : lightness + saturation - lightness * saturation;
  const p = 2 * lightness - q;
  return [
    hueToRgb(p, q, normalizedHue + 1 / 3),
    hueToRgb(p, q, normalizedHue),
    hueToRgb(p, q, normalizedHue - 1 / 3),
  ];
}

function distance(first: UfoVec2, second: UfoVec2): number {
  return Math.hypot(second[0] - first[0], second[1] - first[1]);
}

function distance3(first: UfoVec3, second: UfoVec3): number {
  return Math.hypot(
    second[0] - first[0],
    second[1] - first[1],
    second[2] - first[2],
  );
}

export function projectUfoPoint(point: UfoVec3): UfoVec2 {
  if (!Number.isFinite(point[2]) || point[2] <= 0) {
    throw new Error("UFO camera depth must be finite and positive");
  }
  return [point[0] / point[2], point[1] / point[2]];
}

export function projectUfoRadius(
  referenceRadius: number,
  cameraDepth: number,
): number {
  if (
    !Number.isFinite(referenceRadius) ||
    referenceRadius <= 0 ||
    !Number.isFinite(cameraDepth) ||
    cameraDepth <= 0
  ) {
    throw new Error("UFO projection inputs must be finite and positive");
  }
  return referenceRadius / cameraDepth;
}

function cameraPoint(projected: UfoVec2, depth: number): UfoVec3 {
  return [projected[0] * depth, projected[1] * depth, depth];
}

export function ufoTurnAngle3(
  previous: UfoVec3,
  current: UfoVec3,
  next: UfoVec3,
): number {
  const incoming: UfoVec3 = [
    current[0] - previous[0],
    current[1] - previous[1],
    current[2] - previous[2],
  ];
  const outgoing: UfoVec3 = [
    next[0] - current[0],
    next[1] - current[1],
    next[2] - current[2],
  ];
  const incomingLength = Math.hypot(...incoming);
  const outgoingLength = Math.hypot(...outgoing);
  if (incomingLength * outgoingLength < 1e-9) return 0;
  const cosine =
    (incoming[0] * outgoing[0] +
      incoming[1] * outgoing[1] +
      incoming[2] * outgoing[2]) /
    (incomingLength * outgoingLength);
  return Math.acos(Math.min(1, Math.max(-1, cosine)));
}

export function ufoSizeForVariate(variate: number): number {
  const bounded = Math.min(1, Math.max(0, variate));
  return UFO_SIZE_MIN + (UFO_SIZE_MAX - UFO_SIZE_MIN) * bounded ** 3;
}

export function ufoPathEdge(point: UfoVec2): UfoPathEdge | null {
  if (point[0] <= -OUTSIDE) return "left";
  if (point[0] >= OUTSIDE) return "right";
  if (point[1] >= OUTSIDE) return "top";
  if (point[1] <= -OUTSIDE) return "bottom";
  return null;
}

export function ufoSignedTurnAngle(
  previous: UfoVec2,
  current: UfoVec2,
  next: UfoVec2,
): number {
  const incoming: UfoVec2 = [
    current[0] - previous[0],
    current[1] - previous[1],
  ];
  const outgoing: UfoVec2 = [next[0] - current[0], next[1] - current[1]];
  const cross = incoming[0] * outgoing[1] - incoming[1] * outgoing[0];
  const dot = incoming[0] * outgoing[0] + incoming[1] * outgoing[1];
  if (Math.hypot(...incoming) * Math.hypot(...outgoing) < 1e-9) return 0;
  return Math.atan2(cross, dot);
}

export function ufoTurnAngle(
  previous: UfoVec2,
  current: UfoVec2,
  next: UfoVec2,
): number {
  return Math.abs(ufoSignedTurnAngle(previous, current, next));
}

function appearanceFor(pageSeed: PageSeed, ordinal: bigint): UfoAppearance {
  const shape = randomFor(pageSeed, ordinal, "ufo-shape");
  const material = randomFor(pageSeed, ordinal, "ufo-material");
  const lights = randomFor(pageSeed, ordinal, "ufo-lights");
  const bodyHue = sampleRange(material, [178, 286]);
  const lightHueA = sampleRange(lights, [0, 360]);
  const lightHueB = (lightHueA + sampleRange(lights, [92, 268])) % 360;
  return {
    hullThickness: sampleRange(shape, [0.16, 0.25]),
    rimBevel: sampleRange(shape, [0.045, 0.105]),
    domeRadius: sampleRange(shape, [0.34, 0.55]),
    domeHeight: sampleRange(shape, [0.24, 0.46]),
    domeTint: sampleRange(material, [0.22, 0.78]),
    undersideDepth: sampleRange(shape, [0.16, 0.31]),
    metallicity: sampleRange(material, [0.62, 0.96]),
    roughness: sampleRange(material, [0.12, 0.48]),
    panelFrequency: sampleRange(material, [5, 17]),
    bodyColor: hsl(bodyHue, sampleRange(material, [0.08, 0.28]), 0.38),
    trimColor: hsl(
      (bodyHue + sampleRange(material, [18, 72])) % 360,
      sampleRange(material, [0.2, 0.52]),
      0.58,
    ),
    lightCount:
      UFO_LIGHT_COUNT_MIN +
      Math.floor(
        lights.next() * (UFO_LIGHT_COUNT_MAX - UFO_LIGHT_COUNT_MIN + 1),
      ),
    lightOffset: lights.next() * PI2,
    lightSpacingJitter: sampleRange(lights, [0.08, 0.32]),
    lightColorA: hsl(lightHueA, sampleRange(lights, [0.72, 0.98]), 0.62),
    lightColorB: hsl(lightHueB, sampleRange(lights, [0.72, 0.98]), 0.62),
    lightPulsePhase: lights.next() * PI2,
    lightPulseFrequency: sampleRange(lights, [2.2, 7.5]),
  };
}

function pathFor(
  pageSeed: PageSeed,
  ordinal: bigint,
  turnCount: number,
  domain = "ufo-path",
): readonly UfoVec2[] {
  const random = randomFor(pageSeed, ordinal, domain);
  const edgePoint = (edge: UfoPathEdge): UfoVec2 => {
    const horizontal = sampleRange(random, [-PATH_INTERIOR_X, PATH_INTERIOR_X]);
    const vertical = sampleRange(random, [-PATH_INTERIOR_Y, PATH_INTERIOR_Y]);
    switch (edge) {
      case "left":
        return [-OUTSIDE, vertical];
      case "right":
        return [OUTSIDE, vertical];
      case "top":
        return [horizontal, OUTSIDE];
      case "bottom":
        return [horizontal, -OUTSIDE];
    }
  };
  const valid = (points: readonly UfoVec2[]): boolean => {
    for (let index = 0; index < points.length - 1; index += 1) {
      const from = points[index];
      const to = points[index + 1];
      if (!from || !to || distance(from, to) < PATH_MIN_LEG) return false;
    }
    for (let index = 1; index < points.length - 1; index += 1) {
      const previous = points[index - 1];
      const current = points[index];
      const next = points[index + 1];
      if (!previous || !current || !next) return false;
      const angle = ufoTurnAngle(previous, current, next);
      if (angle < UFO_TURN_ANGLE_MIN || angle > UFO_TURN_ANGLE_MAX) {
        return false;
      }
    }
    const xs = points.map(([x]) => x);
    const ys = points.map(([, y]) => y);
    const horizontalSpan = Math.max(...xs) - Math.min(...xs);
    const verticalSpan = Math.max(...ys) - Math.min(...ys);
    if (Math.max(horizontalSpan, verticalSpan) < PATH_MIN_SPAN) return false;
    const entryEdge = ufoPathEdge(points[0] ?? [0, 0]);
    const exitEdge = ufoPathEdge(points.at(-1) ?? [0, 0]);
    if (!entryEdge || !exitEdge || entryEdge !== exitEdge) return true;
    switch (entryEdge) {
      case "left":
        return Math.max(...xs) + OUTSIDE >= PATH_MIN_SPAN;
      case "right":
        return OUTSIDE - Math.min(...xs) >= PATH_MIN_SPAN;
      case "top":
        return OUTSIDE - Math.min(...ys) >= PATH_MIN_SPAN;
      case "bottom":
        return Math.max(...ys) + OUTSIDE >= PATH_MIN_SPAN;
    }
  };

  for (let attempt = 0; attempt < PATH_CANDIDATE_LIMIT; attempt += 1) {
    const entryEdge = PATH_EDGES[Math.floor(random.next() * PATH_EDGES.length)];
    const exitEdge = PATH_EDGES[Math.floor(random.next() * PATH_EDGES.length)];
    if (!entryEdge || !exitEdge) throw new Error("Invalid UFO path edge");
    const points: UfoVec2[] = [edgePoint(entryEdge)];
    for (let index = 0; index < turnCount; index += 1) {
      points.push([
        sampleRange(random, [-PATH_INTERIOR_X, PATH_INTERIOR_X]),
        sampleRange(random, [-PATH_INTERIOR_Y, PATH_INTERIOR_Y]),
      ]);
    }
    points.push(edgePoint(exitEdge));
    if (valid(points)) return points;
  }
  throw new Error("Unable to generate a valid procedural UFO path");
}

function appearanceSignature(appearance: UfoAppearance): string {
  return Object.values({ ...appearance })
    .flatMap((value) => (typeof value === "number" ? [value] : value))
    .map((value) => value.toFixed(7))
    .join(":");
}

export function ufoTraversalClassFor(
  pageSeed: PageSeed,
  ordinal: bigint,
  forcedTraversalClass: UfoTraversalClass | null = null,
): UfoTraversalClass {
  if (forcedTraversalClass) return forcedTraversalClass;
  const variate = randomFor(pageSeed, ordinal, "ufo-traversal-class").next();
  if (variate < 0.5) return "lateral";
  const specialIndex = Math.min(3, Math.floor((variate - 0.5) * 8));
  return UFO_TRAVERSAL_CLASSES[specialIndex + 1] ?? "distance-exit";
}

export function ufoTraversalClassFromSearch(
  search: string,
  reviewMode: string | null,
): UfoTraversalClass | null {
  if (reviewMode !== "ufo") return null;
  const value = new URLSearchParams(search).get("space-rain-ufo-depth");
  return UFO_TRAVERSAL_CLASSES.find((candidate) => candidate === value) ?? null;
}

function nearProjectedEndpoint(
  edge: UfoPathEdge,
  random: SeededRandom,
  referenceRadius: number,
  depth: number,
): UfoVec2 {
  const projectedBound =
    (projectUfoRadius(referenceRadius, depth) * UFO_QUAD_EXTENT * 2) /
    UFO_SAFE_VIEWPORT_MIN;
  const outside = 1 + projectedBound + 0.12;
  const horizontal = sampleRange(random, [-PATH_INTERIOR_X, PATH_INTERIOR_X]);
  const vertical = sampleRange(random, [-PATH_INTERIOR_Y, PATH_INTERIOR_Y]);
  switch (edge) {
    case "left":
      return [-outside, vertical];
    case "right":
      return [outside, vertical];
    case "top":
      return [horizontal, outside];
    case "bottom":
      return [horizontal, -outside];
  }
}

function validCameraPath(points: readonly UfoVec3[]): boolean {
  const projected = points.map(projectUfoPoint);
  for (let index = 0; index < points.length - 1; index += 1) {
    const from = points[index];
    const to = points[index + 1];
    const projectedFrom = projected[index];
    const projectedTo = projected[index + 1];
    if (
      !from ||
      !to ||
      !projectedFrom ||
      !projectedTo ||
      distance3(from, to) < 1e-6 ||
      distance(projectedFrom, projectedTo) < 0.18
    ) {
      return false;
    }
  }
  for (let index = 1; index < points.length - 1; index += 1) {
    const previous = points[index - 1];
    const current = points[index];
    const next = points[index + 1];
    const previousProjected = projected[index - 1];
    const currentProjected = projected[index];
    const nextProjected = projected[index + 1];
    if (
      !previous ||
      !current ||
      !next ||
      !previousProjected ||
      !currentProjected ||
      !nextProjected
    ) {
      return false;
    }
    const cameraAngle = ufoTurnAngle3(previous, current, next);
    const projectedAngle = ufoTurnAngle(
      previousProjected,
      currentProjected,
      nextProjected,
    );
    if (
      cameraAngle < UFO_TURN_ANGLE_MIN ||
      cameraAngle > UFO_TURN_ANGLE_MAX ||
      projectedAngle < UFO_PROJECTED_TURN_ANGLE_MIN ||
      projectedAngle > UFO_PROJECTED_TURN_ANGLE_MAX
    ) {
      return false;
    }
  }
  return true;
}

function cameraPathFor(options: {
  pageSeed: PageSeed;
  ordinal: bigint;
  projectedPoints: readonly UfoVec2[];
  turnCount: number;
  traversalClass: UfoTraversalClass;
  referenceRadius: number;
}): readonly UfoVec3[] {
  const {
    pageSeed,
    ordinal,
    projectedPoints: baseline,
    turnCount,
    traversalClass,
    referenceRadius,
  } = options;
  if (traversalClass === "lateral") {
    return baseline.map(([x, y]) => [x, y, UFO_REFERENCE_DEPTH] as const);
  }

  const random = randomFor(pageSeed, ordinal, "ufo-depth-path");
  const specialAtStart =
    traversalClass === "camera-entry" || traversalClass === "distance-entry";
  for (let attempt = 0; attempt < PATH_CANDIDATE_LIMIT; attempt += 1) {
    const projectedBaseline =
      attempt === 0
        ? baseline
        : pathFor(
            pageSeed,
            ordinal,
            turnCount,
            `ufo-depth-projected-path:${String(attempt)}`,
          );
    const specialIndex = specialAtStart ? 0 : projectedBaseline.length - 1;
    const projected = projectedBaseline.map(
      (point) => [...point] as [number, number],
    );
    const depths = projectedBaseline.map(() => UFO_REFERENCE_DEPTH);
    for (let index = 1; index < depths.length - 1; index += 1) {
      depths[index] = sampleRange(random, [0.72, 1.28]);
    }

    if (traversalClass === "camera-entry" || traversalClass === "camera-exit") {
      const baselineEndpoint = projectedBaseline[specialIndex];
      const edge = baselineEndpoint ? ufoPathEdge(baselineEndpoint) : null;
      if (!edge) throw new Error("Near-camera UFO endpoint has no edge");
      const targetBound = sampleRange(random, UFO_NEAR_BOUND_RANGE);
      const nearDepth = (referenceRadius * UFO_QUAD_EXTENT) / targetBound;
      projected[specialIndex] = [
        ...nearProjectedEndpoint(edge, random, referenceRadius, nearDepth),
      ];
      depths[specialIndex] = nearDepth;
    } else {
      const targetBound = sampleRange(random, UFO_FAR_BOUND_RANGE);
      const farDepth = (referenceRadius * UFO_QUAD_EXTENT) / targetBound;
      projected[specialIndex] = [
        sampleRange(random, [-0.88, 0.88]),
        sampleRange(random, [-0.76, 0.76]),
      ];
      depths[specialIndex] = farDepth;
    }

    const points = projected.map((point, index) =>
      cameraPoint(point, depths[index] ?? UFO_REFERENCE_DEPTH),
    );
    if (validCameraPath(points)) return points;
  }
  throw new Error("Unable to generate a valid camera-space UFO path");
}

function trajectoryFor(
  pageSeed: PageSeed,
  ordinal: bigint,
  referenceRadius: number,
  forcedTraversalClass: UfoTraversalClass | null,
): UfoTrajectory {
  const random = randomFor(pageSeed, ordinal, "ufo-trajectory");
  const turnCount =
    UFO_TURN_COUNT_MIN +
    Math.floor(random.next() * (UFO_TURN_COUNT_MAX - UFO_TURN_COUNT_MIN + 1));
  const projectedPoints = pathFor(pageSeed, ordinal, turnCount);
  const traversalClass = ufoTraversalClassFor(
    pageSeed,
    ordinal,
    forcedTraversalClass,
  );
  const points = cameraPathFor({
    pageSeed,
    ordinal,
    projectedPoints,
    turnCount,
    traversalClass,
    referenceRadius,
  });
  const pauseCount = Math.min(
    turnCount,
    UFO_PAUSE_COUNT_MIN +
      Math.floor(
        random.next() * (UFO_PAUSE_COUNT_MAX - UFO_PAUSE_COUNT_MIN + 1),
      ),
  );
  const pauseCandidates = Array.from(
    { length: turnCount },
    (_, index) => index + 1,
  );
  for (let index = pauseCandidates.length - 1; index > 0; index -= 1) {
    const selected = Math.floor(random.next() * (index + 1));
    [pauseCandidates[index], pauseCandidates[selected]] = [
      pauseCandidates[selected] ?? 0,
      pauseCandidates[index] ?? 0,
    ];
  }
  const pauseWaypoints = new Set(pauseCandidates.slice(0, pauseCount));
  const segments: UfoSegment[] = [];
  let elapsed = 0;
  for (let index = 0; index < points.length - 1; index += 1) {
    const from = points[index];
    const to = points[index + 1];
    if (!from || !to) throw new Error("Invalid UFO path");
    const speed =
      index === points.length - 2
        ? sampleRange(random, [1.65, 2.55])
        : sampleRange(random, [1.12, 1.88]);
    const projectedFrom = projectUfoPoint(from);
    const projectedTo = projectUfoPoint(to);
    const duration = distance(projectedFrom, projectedTo) / speed;
    const velocity: UfoVec3 = [
      (to[0] - from[0]) / duration,
      (to[1] - from[1]) / duration,
      (to[2] - from[2]) / duration,
    ];
    const heading = Math.atan2(
      projectedTo[1] - projectedFrom[1],
      projectedTo[0] - projectedFrom[0],
    );
    segments.push({
      kind: "travel",
      startTime: elapsed,
      endTime: elapsed + duration,
      from,
      to,
      velocity,
      heading,
      elevation: Math.atan2(velocity[2], Math.hypot(velocity[0], velocity[1])),
    });
    elapsed += duration;
    const waypointIndex = index + 1;
    if (pauseWaypoints.has(waypointIndex)) {
      const pauseDuration = sampleRange(random, [0.35, 1.25]);
      segments.push({
        kind: "pause",
        startTime: elapsed,
        endTime: elapsed + pauseDuration,
        position: to,
        spinTurns: sampleRange(random, [0.75, 2.5]),
        spinDirection: random.next() < 0.5 ? -1 : 1,
      });
      elapsed += pauseDuration;
    }
  }
  return {
    traversalClass,
    points,
    segments,
    turnCount,
    pauseCount,
    duration: elapsed,
  };
}

export function createUfoEncounter(
  pageSeed: PageSeed,
  ordinal: bigint,
  forcedTraversalClass: UfoTraversalClass | null = null,
): UfoEncounter {
  const placement = randomFor(pageSeed, ordinal, "ufo-placement");
  const appearance = appearanceFor(pageSeed, ordinal);
  placement.next();
  const referenceRadius = ufoSizeForVariate(placement.next());
  const trajectory = trajectoryFor(
    pageSeed,
    ordinal,
    referenceRadius,
    forcedTraversalClass,
  );
  return {
    identity: `ufo:${objectIdentity(pageSeed, ordinal)}`,
    ordinal,
    visualSignature: appearanceSignature(appearance),
    appearance,
    trajectory,
    referenceRadius,
    inclination: sampleRange(placement, [0.28, 0.92]),
    initialYaw: placement.next() * PI2,
    initialRoll: sampleRange(placement, [-0.28, 0.28]),
  };
}

function projectedVelocityFor(
  cameraPosition: UfoVec3,
  cameraVelocity: UfoVec3,
): UfoVec2 {
  const [x, y, z] = cameraPosition;
  const [velocityX, velocityY, velocityZ] = cameraVelocity;
  const squaredDepth = z * z;
  return [
    (velocityX * z - x * velocityZ) / squaredDepth,
    (velocityY * z - y * velocityZ) / squaredDepth,
  ];
}

function stateForPosition(options: {
  encounter: UfoEncounter;
  kind: UfoState["kind"];
  segmentIndex: number;
  cameraPosition: UfoVec3;
  cameraVelocity: UfoVec3;
  yaw: number;
  pitch: number;
  roll: number;
  localTime: number;
}): UfoState {
  const {
    encounter,
    kind,
    segmentIndex,
    cameraPosition,
    cameraVelocity,
    yaw,
    pitch,
    roll,
    localTime,
  } = options;
  return {
    kind,
    segmentIndex,
    cameraPosition,
    cameraVelocity,
    projectedPosition: projectUfoPoint(cameraPosition),
    projectedVelocity: projectedVelocityFor(cameraPosition, cameraVelocity),
    projectedRadius: projectUfoRadius(
      encounter.referenceRadius,
      cameraPosition[2],
    ),
    yaw,
    pitch,
    roll,
    localTime,
  };
}

function stateForTravel(
  encounter: UfoEncounter,
  segment: UfoTravelLeg,
  segmentIndex: number,
  elapsed: number,
): UfoState {
  const duration = segment.endTime - segment.startTime;
  const progress = Math.min(
    1,
    Math.max(0, (elapsed - segment.startTime) / duration),
  );
  const turnLean = Math.sin(segment.heading * 1.7) * 0.1;
  const cameraPosition: UfoVec3 = [
    mix(segment.from[0], segment.to[0], progress),
    mix(segment.from[1], segment.to[1], progress),
    mix(segment.from[2], segment.to[2], progress),
  ];
  return stateForPosition({
    encounter,
    kind: "travel",
    segmentIndex,
    cameraPosition,
    cameraVelocity: segment.velocity,
    yaw: encounter.initialYaw + segment.heading,
    pitch: encounter.inclination + turnLean + segment.elevation * 0.42,
    roll: encounter.initialRoll + Math.sin(segment.heading) * 0.16,
    localTime: elapsed,
  });
}

function stateForPause(
  encounter: UfoEncounter,
  segment: UfoPause,
  segmentIndex: number,
  elapsed: number,
): UfoState {
  const duration = segment.endTime - segment.startTime;
  const progress = Math.min(
    1,
    Math.max(0, (elapsed - segment.startTime) / duration),
  );
  const spin = progress * segment.spinTurns * PI2 * segment.spinDirection;
  return stateForPosition({
    encounter,
    kind: "pause",
    segmentIndex,
    cameraPosition: segment.position,
    cameraVelocity: [0, 0, 0],
    yaw: encounter.initialYaw + spin,
    pitch: encounter.inclination + Math.sin(spin * 0.5) * 0.34,
    roll: encounter.initialRoll + Math.sin(spin) * 0.24,
    localTime: elapsed,
  });
}

export function evaluateUfoEncounter(
  encounter: UfoEncounter,
  elapsed: number,
): UfoState {
  const time = Math.max(0, elapsed);
  for (
    let index = 0;
    index < encounter.trajectory.segments.length;
    index += 1
  ) {
    const segment = encounter.trajectory.segments[index];
    if (!segment || time >= segment.endTime) continue;
    return segment.kind === "travel"
      ? stateForTravel(encounter, segment, index, time)
      : stateForPause(encounter, segment, index, time);
  }
  const last =
    encounter.trajectory.points.at(-1) ??
    ([OUTSIDE, 0, UFO_REFERENCE_DEPTH] as const);
  return stateForPosition({
    encounter,
    kind: "complete",
    segmentIndex: encounter.trajectory.segments.length,
    cameraPosition: last,
    cameraVelocity: [0, 0, 0],
    yaw: encounter.initialYaw,
    pitch: encounter.inclination,
    roll: encounter.initialRoll,
    localTime: Math.min(time, encounter.trajectory.duration),
  });
}

function gapFor(
  scheduler: Pick<UfoScheduler, "pageSeed" | "reviewMode">,
  ordinal: bigint,
  initial: boolean,
): number {
  const random = randomFor(
    scheduler.pageSeed,
    ordinal,
    initial ? "ufo-first-delay" : "ufo-gap",
  );
  const range = initial
    ? scheduler.reviewMode
      ? UFO_REVIEW_FIRST_DELAY_RANGE
      : UFO_FIRST_DELAY_RANGE
    : scheduler.reviewMode
      ? UFO_REVIEW_GAP_RANGE
      : UFO_NORMAL_GAP_RANGE;
  return sampleRange(random, range);
}

export function createUfoScheduler(options: {
  pageSeed: PageSeed;
  reviewMode: boolean;
  reducedMotion: boolean;
  forcedTraversalClass?: UfoTraversalClass | null;
}): UfoScheduler {
  const scheduler: UfoScheduler = {
    pageSeed: options.pageSeed,
    reviewMode: options.reviewMode,
    forcedTraversalClass: options.forcedTraversalClass ?? null,
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

export function setUfoReducedMotion(
  scheduler: UfoScheduler,
  reducedMotion: boolean,
): void {
  scheduler.reducedMotion = reducedMotion;
  if (!reducedMotion) return;
  scheduler.active = null;
  scheduler.activeElapsed = 0;
  scheduler.waitRemaining = gapFor(scheduler, scheduler.nextOrdinal, true);
}

export function advanceUfoScheduler(
  scheduler: UfoScheduler,
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
      scheduler.active = createUfoEncounter(
        scheduler.pageSeed,
        scheduler.nextOrdinal,
        scheduler.forcedTraversalClass,
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
    scheduler.activeElapsed = scheduler.active.trajectory.duration;
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

export function currentUfoState(scheduler: UfoScheduler): UfoState | null {
  return scheduler.active
    ? evaluateUfoEncounter(scheduler.active, scheduler.activeElapsed)
    : null;
}
