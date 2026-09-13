import type { RainFamily } from "./config";
import type { TraitVector } from "./model";
import { wrapAngle, type Vec3 } from "./planet-rotation";

export type VolumetricFamily = Exclude<RainFamily, "planet">;

export interface VolumetricMotion {
  axisInclination: number;
  axisPosition: number;
  bulkAngle: number;
  bulkRate: number;
  internalAngle: number;
  internalRate: number;
  evolutionAngle: number;
  evolutionRate: number;
}

export const MIN_VOLUME_ROTATION_PERIOD_SECONDS = 24;
export const MAX_VOLUME_ROTATION_PERIOD_SECONDS = 96;
export const MIN_VOLUME_SAMPLES = 6;
export const MAX_VOLUME_SAMPLES = 20;

function dot(first: Vec3, second: Vec3): number {
  return first[0] * second[0] + first[1] * second[1] + first[2] * second[2];
}

function cross(first: Vec3, second: Vec3): Vec3 {
  return [
    first[1] * second[2] - first[2] * second[1],
    first[2] * second[0] - first[0] * second[2],
    first[0] * second[1] - first[1] * second[0],
  ];
}

function normalize(vector: Vec3): Vec3 {
  const length = Math.hypot(...vector);
  if (!Number.isFinite(length) || length < 1e-9) {
    throw new Error("Cannot normalize a degenerate volume vector");
  }
  return [vector[0] / length, vector[1] / length, vector[2] / length];
}

function rotateLocalY(vector: Vec3, angle: number): Vec3 {
  const cosine = Math.cos(angle);
  const sine = Math.sin(angle);
  return [
    cosine * vector[0] + sine * vector[2],
    vector[1],
    -sine * vector[0] + cosine * vector[2],
  ];
}

export function volumeRotationPeriod(rate: number): number {
  return (Math.PI * 2) / Math.abs(rate);
}

export function volumeAxis(motion: VolumetricMotion): Vec3 {
  const projected = Math.sin(motion.axisInclination);
  return normalize([
    projected * Math.cos(motion.axisPosition),
    projected * Math.sin(motion.axisPosition),
    Math.cos(motion.axisInclination),
  ]);
}

export function volumeFrame(motion: VolumetricMotion): {
  axis: Vec3;
  east: Vec3;
  forward: Vec3;
} {
  const axis = volumeAxis(motion);
  const reference: Vec3 = Math.abs(axis[2]) < 0.92 ? [0, 0, 1] : [0, 1, 0];
  const east = normalize(cross(reference, axis));
  const forward = normalize(cross(axis, east));
  return { axis, east, forward };
}

export function volumeLocalToView(
  point: Vec3,
  motion: VolumetricMotion,
  angle = motion.bulkAngle,
): Vec3 {
  const { axis, east, forward } = volumeFrame(motion);
  const rotated = rotateLocalY(point, angle);
  return [
    east[0] * rotated[0] + axis[0] * rotated[1] + forward[0] * rotated[2],
    east[1] * rotated[0] + axis[1] * rotated[1] + forward[1] * rotated[2],
    east[2] * rotated[0] + axis[2] * rotated[1] + forward[2] * rotated[2],
  ];
}

export function volumeViewToLocal(
  point: Vec3,
  motion: VolumetricMotion,
  angle = motion.bulkAngle,
): Vec3 {
  const { axis, east, forward } = volumeFrame(motion);
  return rotateLocalY(
    [dot(point, east), dot(point, axis), dot(point, forward)],
    -angle,
  );
}

export function advanceVolumetricMotion(
  motion: VolumetricMotion,
  elapsedSeconds: number,
): void {
  motion.bulkAngle = wrapAngle(
    motion.bulkAngle + motion.bulkRate * elapsedSeconds,
  );
  motion.internalAngle = wrapAngle(
    motion.internalAngle + motion.internalRate * elapsedSeconds,
  );
  motion.evolutionAngle = wrapAngle(
    motion.evolutionAngle + motion.evolutionRate * elapsedSeconds,
  );
}

export function validVolumetricMotion(motion: VolumetricMotion): boolean {
  const period = volumeRotationPeriod(motion.bulkRate);
  return (
    Object.values(motion).every(Number.isFinite) &&
    motion.axisInclination > 0 &&
    motion.axisInclination < Math.PI / 2 &&
    motion.bulkRate !== 0 &&
    motion.internalRate !== 0 &&
    motion.evolutionRate !== 0 &&
    period >= MIN_VOLUME_ROTATION_PERIOD_SECONDS - 1e-9 &&
    period <= MAX_VOLUME_ROTATION_PERIOD_SECONDS + 1e-9
  );
}

export function volumeSampleCount(depth: number, detail: number): number {
  const amount = Math.min(1, Math.max(0, depth * 0.58 + detail * 0.42));
  return Math.round(
    MIN_VOLUME_SAMPLES + (MAX_VOLUME_SAMPLES - MIN_VOLUME_SAMPLES) * amount,
  );
}

export function rayEllipsoidIntersection(
  origin: Vec3,
  direction: Vec3,
  radii: Vec3,
): readonly [number, number] | null {
  if (radii.some((radius) => !Number.isFinite(radius) || radius <= 0)) {
    return null;
  }
  const scaledOrigin: Vec3 = [
    origin[0] / radii[0],
    origin[1] / radii[1],
    origin[2] / radii[2],
  ];
  const scaledDirection: Vec3 = [
    direction[0] / radii[0],
    direction[1] / radii[1],
    direction[2] / radii[2],
  ];
  const a = dot(scaledDirection, scaledDirection);
  const b = 2 * dot(scaledOrigin, scaledDirection);
  const c = dot(scaledOrigin, scaledOrigin) - 1;
  const discriminant = b * b - 4 * a * c;
  if (!Number.isFinite(discriminant) || discriminant < 0 || a <= 1e-12) {
    return null;
  }
  const root = Math.sqrt(discriminant);
  const first = (-b - root) / (2 * a);
  const second = (-b + root) / (2 * a);
  return first <= second ? [first, second] : [second, first];
}

export function ellipsoidEnvelope(point: Vec3, radii: Vec3): number {
  if (radii.some((radius) => radius <= 0)) return 0;
  const radius = Math.hypot(
    point[0] / radii[0],
    point[1] / radii[1],
    point[2] / radii[2],
  );
  return Math.max(0, 1 - radius);
}

export function supernovaAxisScales(
  traitsA: TraitVector,
  traitsB: TraitVector,
): Vec3 {
  return [
    0.82 + traitsA[0] * 0.18,
    0.38 + traitsA[1] * 0.14,
    0.58 + traitsB[0] * 0.24,
  ];
}

export function supernovaShellRadius(
  direction: Vec3,
  traitsA: TraitVector,
  traitsB: TraitVector,
): number {
  const unit = normalize(direction);
  const azimuth = Math.atan2(unit[2], unit[0]);
  const polarLobe = unit[1] * unit[1] - 1 / 3;
  return (
    1 +
    (0.12 + traitsA[2] * 0.1) * Math.sin(2 * azimuth + traitsB[1] * 6) +
    (0.15 + traitsA[3] * 0.13) * polarLobe
  );
}
