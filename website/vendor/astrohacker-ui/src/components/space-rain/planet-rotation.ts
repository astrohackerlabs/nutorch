export type Vec3 = readonly [number, number, number];

export interface PlanetRotation {
  axisInclination: number;
  axisPosition: number;
  surfaceAngle: number;
  surfaceRate: number;
  cloudAngle: number;
  cloudRate: number;
  ringAngle: number;
  ringRate: number;
}

export const MIN_PLANET_ROTATION_PERIOD_SECONDS = 24;
export const MAX_PLANET_ROTATION_PERIOD_SECONDS = 80;
export const MIN_PLANET_ROTATION_RATE =
  (Math.PI * 2) / MAX_PLANET_ROTATION_PERIOD_SECONDS;
export const MAX_PLANET_ROTATION_RATE =
  (Math.PI * 2) / MIN_PLANET_ROTATION_PERIOD_SECONDS;

export function wrapAngle(angle: number): number {
  const turn = Math.PI * 2;
  return ((angle % turn) + turn) % turn;
}

export function rotationPeriod(rate: number): number {
  return (Math.PI * 2) / Math.abs(rate);
}

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
    throw new Error("Cannot normalize a degenerate planet vector");
  }
  return [vector[0] / length, vector[1] / length, vector[2] / length];
}

export function axisFromAngles(inclination: number, position: number): Vec3 {
  const projected = Math.sin(inclination);
  return normalize([
    projected * Math.cos(position),
    projected * Math.sin(position),
    Math.cos(inclination),
  ]);
}

export function planetFrame(rotation: PlanetRotation): {
  axis: Vec3;
  east: Vec3;
  forward: Vec3;
} {
  const axis = axisFromAngles(rotation.axisInclination, rotation.axisPosition);
  const reference: Vec3 = Math.abs(axis[2]) < 0.92 ? [0, 0, 1] : [0, 1, 0];
  const east = normalize(cross(reference, axis));
  const forward = normalize(cross(axis, east));
  return { axis, east, forward };
}

function rotateBodyY(direction: Vec3, angle: number): Vec3 {
  const cosine = Math.cos(angle);
  const sine = Math.sin(angle);
  return [
    cosine * direction[0] + sine * direction[2],
    direction[1],
    -sine * direction[0] + cosine * direction[2],
  ];
}

export function bodyToViewDirection(
  direction: Vec3,
  rotation: PlanetRotation,
  angle = rotation.surfaceAngle,
): Vec3 {
  const { axis, east, forward } = planetFrame(rotation);
  const turned = rotateBodyY(direction, angle);
  return normalize([
    east[0] * turned[0] + axis[0] * turned[1] + forward[0] * turned[2],
    east[1] * turned[0] + axis[1] * turned[1] + forward[1] * turned[2],
    east[2] * turned[0] + axis[2] * turned[1] + forward[2] * turned[2],
  ]);
}

export function viewToBodyDirection(
  direction: Vec3,
  rotation: PlanetRotation,
  angle = rotation.surfaceAngle,
): Vec3 {
  const { axis, east, forward } = planetFrame(rotation);
  const base: Vec3 = [
    dot(direction, east),
    dot(direction, axis),
    dot(direction, forward),
  ];
  return normalize(rotateBodyY(base, -angle));
}

export function advancePlanetRotation(
  rotation: PlanetRotation,
  elapsedSeconds: number,
): void {
  rotation.surfaceAngle = wrapAngle(
    rotation.surfaceAngle + rotation.surfaceRate * elapsedSeconds,
  );
  rotation.cloudAngle = wrapAngle(
    rotation.cloudAngle + rotation.cloudRate * elapsedSeconds,
  );
  rotation.ringAngle = wrapAngle(
    rotation.ringAngle + rotation.ringRate * elapsedSeconds,
  );
}

export function validPlanetRotation(rotation: PlanetRotation): boolean {
  const values = Object.values(rotation);
  const period = rotationPeriod(rotation.surfaceRate);
  return (
    values.every(Number.isFinite) &&
    rotation.axisInclination > 0 &&
    rotation.axisInclination < Math.PI / 2 &&
    period >= MIN_PLANET_ROTATION_PERIOD_SECONDS - 1e-9 &&
    period <= MAX_PLANET_ROTATION_PERIOD_SECONDS + 1e-9 &&
    rotation.surfaceRate !== 0 &&
    rotation.cloudRate !== rotation.surfaceRate &&
    rotation.ringRate !== 0
  );
}
