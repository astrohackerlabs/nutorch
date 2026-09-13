import {
  advanceBlackHoleScheduler,
  createBlackHoleScheduler,
  setBlackHoleReducedMotion,
  type BlackHoleScheduler,
} from "./black-hole";
import {
  DEFAULT_SPACE_RAIN_CONFIG,
  RAIN_FAMILIES,
  SEED_CLASSES,
  type RainFamily,
  type ReviewMode,
  type SeedClassSpec,
  type SpaceRainConfig,
  validateConfig,
} from "./config";
import {
  objectIdentity,
  pageSeedIdentifier,
  randomFor,
  type PageSeed,
  type SeededRandom,
} from "./seed";
import {
  advancePlanetRotation,
  MAX_PLANET_ROTATION_PERIOD_SECONDS,
  MIN_PLANET_ROTATION_PERIOD_SECONDS,
  type PlanetRotation,
} from "./planet-rotation";
import {
  advanceVolumetricMotion,
  MAX_VOLUME_ROTATION_PERIOD_SECONDS,
  MIN_VOLUME_ROTATION_PERIOD_SECONDS,
  type VolumetricMotion,
} from "./volumetric-motion";
import {
  advanceUfoScheduler,
  createUfoScheduler,
  setUfoReducedMotion,
  type UfoScheduler,
  type UfoTraversalClass,
} from "./ufo";

export type Color = readonly [number, number, number];
export type TraitVector = readonly [number, number, number, number];

export interface RainInstance {
  identity: string;
  spawnOrdinal: bigint;
  family: RainFamily;
  seedClass: number;
  seedClassName: string;
  visualSignature: string;
  visualVector: readonly number[];
  x: number;
  y: number;
  depth: number;
  size: number;
  intensity: number;
  detail: number;
  speed: number;
  drift: number;
  rotation: number;
  spin: number;
  planetRotation: PlanetRotation | null;
  volumetricMotion: VolumetricMotion | null;
  phase: number;
  age: number;
  traitsA: TraitVector;
  traitsB: TraitVector;
  colorLow: Color;
  colorMid: Color;
  colorHigh: Color;
}

export interface SpaceRainScene {
  pageSeed: PageSeed;
  pageSeedId: string;
  nextSpawnOrdinal: bigint;
  config: SpaceRainConfig;
  width: number;
  height: number;
  reducedMotion: boolean;
  reviewMode: ReviewMode | null;
  instances: RainInstance[];
  targetCount: number;
  spawnCredit: number;
  nextSpawnThreshold: number;
  recycleCount: number;
  ufoScheduler: UfoScheduler;
  blackHoleScheduler: BlackHoleScheduler;
}

function mix(a: number, b: number, amount: number): number {
  return a + (b - a) * amount;
}

function fract(value: number): number {
  return value - Math.floor(value);
}

function pickWeighted(random: SeededRandom, values: readonly number[]): number {
  const target = random.next();
  let sum = 0;
  for (let index = 0; index < values.length; index += 1) {
    sum += values[index] ?? 0;
    if (target <= sum) return index;
  }
  return values.length - 1;
}

export function sizeForDepth(
  depth: number,
  config = DEFAULT_SPACE_RAIN_CONFIG,
): number {
  return mix(config.sizeRange[0], config.sizeRange[1], depth * depth);
}

export function intensityForDepth(
  depth: number,
  config = DEFAULT_SPACE_RAIN_CONFIG,
): number {
  return mix(config.intensityRange[0], config.intensityRange[1], depth);
}

export function detailForDepth(
  depth: number,
  config = DEFAULT_SPACE_RAIN_CONFIG,
): number {
  return mix(config.detailRange[0], config.detailRange[1], depth);
}

export function speedForDepth(
  depth: number,
  config = DEFAULT_SPACE_RAIN_CONFIG,
): number {
  return mix(config.speedRange[0], config.speedRange[1], depth * depth);
}

function familyFor(random: SeededRandom, config: SpaceRainConfig): RainFamily {
  const index = pickWeighted(
    random,
    RAIN_FAMILIES.map((family) => config.familyWeights[family]),
  );
  return RAIN_FAMILIES[index] ?? "planet";
}

function sampleRange(
  random: SeededRandom,
  range: readonly [number, number],
): number {
  return mix(range[0], range[1], random.next());
}

function samplePlanetRotation(
  pageSeed: PageSeed,
  ordinal: bigint,
  attempt: number,
): PlanetRotation {
  const random = randomFor(
    pageSeed,
    ordinal,
    `planet-rotation:${String(attempt)}`,
  );
  const direction = random.next() < 0.5 ? -1 : 1;
  const period = sampleRange(random, [
    MIN_PLANET_ROTATION_PERIOD_SECONDS,
    MAX_PLANET_ROTATION_PERIOD_SECONDS,
  ]);
  const surfaceRate = (direction * Math.PI * 2) / period;
  const cloudMultiplier = sampleRange(random, [1.06, 1.22]);
  const ringMultiplier = sampleRange(random, [0.58, 1.38]);
  return {
    axisInclination: sampleRange(random, [0.16, 1.45]),
    axisPosition: random.next() * Math.PI * 2,
    surfaceAngle: random.next() * Math.PI * 2,
    surfaceRate,
    cloudAngle: random.next() * Math.PI * 2,
    cloudRate: surfaceRate * cloudMultiplier,
    ringAngle: random.next() * Math.PI * 2,
    ringRate: surfaceRate * ringMultiplier,
  };
}

function planetRotationVector(rotation: PlanetRotation): readonly number[] {
  const direction = rotation.surfaceRate < 0 ? 0 : 1;
  return [
    rotation.axisInclination / (Math.PI / 2),
    rotation.axisPosition / (Math.PI * 2),
    rotation.surfaceAngle / (Math.PI * 2),
    direction,
    (Math.abs(rotation.surfaceRate) - (Math.PI * 2) / 80) /
      ((Math.PI * 2) / 24 - (Math.PI * 2) / 80),
    rotation.cloudAngle / (Math.PI * 2),
    Math.abs(rotation.cloudRate / rotation.surfaceRate - 1) / 0.22,
    rotation.ringAngle / (Math.PI * 2),
    Math.abs(rotation.ringRate / rotation.surfaceRate - 0.58) / 0.8,
  ];
}

function sampleVolumetricMotion(
  pageSeed: PageSeed,
  ordinal: bigint,
  attempt: number,
  family: Exclude<RainFamily, "planet">,
): VolumetricMotion {
  const random = randomFor(
    pageSeed,
    ordinal,
    `volume-motion:${String(attempt)}`,
  );
  const direction = random.next() < 0.5 ? -1 : 1;
  const periodRange: readonly [number, number] =
    family === "galaxy"
      ? [MIN_VOLUME_ROTATION_PERIOD_SECONDS, 72]
      : [36, MAX_VOLUME_ROTATION_PERIOD_SECONDS];
  const bulkRate = (direction * Math.PI * 2) / sampleRange(random, periodRange);
  const internalMultiplier =
    family === "galaxy"
      ? sampleRange(random, [1.18, 2.35])
      : sampleRange(random, [0.34, 0.82]);
  const evolutionPeriod =
    family === "galaxy"
      ? sampleRange(random, [8, 22])
      : sampleRange(random, [9, 28]);
  return {
    axisInclination: sampleRange(random, [0.12, 1.48]),
    axisPosition: random.next() * Math.PI * 2,
    bulkAngle: random.next() * Math.PI * 2,
    bulkRate,
    internalAngle: random.next() * Math.PI * 2,
    internalRate: bulkRate * internalMultiplier,
    evolutionAngle: random.next() * Math.PI * 2,
    evolutionRate: (Math.PI * 2) / evolutionPeriod,
  };
}

function volumetricMotionVector(motion: VolumetricMotion): readonly number[] {
  return [
    motion.axisInclination / (Math.PI / 2),
    motion.axisPosition / (Math.PI * 2),
    motion.bulkAngle / (Math.PI * 2),
    motion.bulkRate < 0 ? 0 : 1,
    (Math.abs(motion.bulkRate) - (Math.PI * 2) / 96) /
      ((Math.PI * 2) / 24 - (Math.PI * 2) / 96),
    motion.internalAngle / (Math.PI * 2),
    Math.abs(motion.internalRate / motion.bulkRate),
    motion.evolutionAngle / (Math.PI * 2),
    motion.evolutionRate / (Math.PI * 2),
  ];
}

function sampleHue(
  random: SeededRandom,
  range: readonly [number, number],
): number {
  const [start, end] = range;
  const span = start <= end ? end - start : 360 - start + end;
  return (start + random.next() * span) % 360;
}

function hueToRgb(p: number, q: number, t: number): number {
  const wrapped = fract(t);
  if (wrapped < 1 / 6) return p + (q - p) * 6 * wrapped;
  if (wrapped < 1 / 2) return q;
  if (wrapped < 2 / 3) return p + (q - p) * (2 / 3 - wrapped) * 6;
  return p;
}

export function hslToRgb(
  hue: number,
  saturation: number,
  lightness: number,
): Color {
  const h = fract(hue / 360);
  if (saturation === 0) return [lightness, lightness, lightness];
  const q =
    lightness < 0.5
      ? lightness * (1 + saturation)
      : lightness + saturation - lightness * saturation;
  const p = 2 * lightness - q;
  return [
    hueToRgb(p, q, h + 1 / 3),
    hueToRgb(p, q, h),
    hueToRgb(p, q, h - 1 / 3),
  ];
}

function sampleColors(
  random: SeededRandom,
  seedClass: SeedClassSpec,
): {
  low: Color;
  mid: Color;
  high: Color;
  normalized: readonly [number, number, number];
} {
  const hue = sampleHue(random, seedClass.hueRange);
  const saturation = sampleRange(random, seedClass.saturationRange);
  const lightness = sampleRange(random, seedClass.lightnessRange);
  const relationship = random.next();
  const lowHue = (hue - mix(8, 38, relationship) + 360) % 360;
  const highHue = (hue + mix(12, 54, 1 - relationship)) % 360;
  return {
    low: hslToRgb(
      lowHue,
      Math.max(0.16, saturation * mix(0.62, 0.88, relationship)),
      Math.max(0.055, lightness * mix(0.28, 0.48, relationship)),
    ),
    mid: hslToRgb(hue, saturation, lightness),
    high: hslToRgb(
      highHue,
      Math.min(0.98, saturation * mix(0.74, 1.08, relationship)),
      Math.min(0.96, lightness + mix(0.16, 0.34, relationship)),
    ),
    normalized: [hue / 360, saturation, lightness],
  };
}

function visualSignature(
  family: RainFamily,
  seedClass: number,
  vector: readonly number[],
): string {
  return `${family}:${String(seedClass)}:${vector
    .map((value) => value.toFixed(7))
    .join(":")}`;
}

export function visualDistance(
  first: readonly number[],
  second: readonly number[],
): number {
  if (first.length !== second.length || first.length === 0) return 0;
  const squared = first.reduce((sum, value, index) => {
    const difference = value - (second[index] ?? value);
    return sum + difference * difference;
  }, 0);
  return Math.sqrt(squared / first.length);
}

function buildCandidate(options: {
  pageSeed: PageSeed;
  ordinal: bigint;
  config: SpaceRainConfig;
  family: RainFamily;
  seedClass: number;
  attempt: number;
  width: number;
  height: number;
}): RainInstance {
  const {
    pageSeed,
    ordinal,
    config,
    family,
    seedClass,
    attempt,
    width,
    height,
  } = options;
  const seedClassSpec = SEED_CLASSES[family][seedClass];
  if (!seedClassSpec) throw new Error(`Missing ${family} seed class`);
  const morphology = randomFor(
    pageSeed,
    ordinal,
    `morphology:${String(attempt)}`,
  );
  const material = randomFor(pageSeed, ordinal, `material:${String(attempt)}`);
  const colorRandom = randomFor(pageSeed, ordinal, `color:${String(attempt)}`);
  const placement = randomFor(pageSeed, ordinal, "placement");
  const motion = randomFor(pageSeed, ordinal, "motion");
  const traitsA: TraitVector = [
    morphology.next(),
    morphology.next(),
    morphology.next(),
    morphology.next(),
  ];
  const traitsB: TraitVector = [
    material.next(),
    material.next(),
    material.next(),
    material.next(),
  ];
  const colors = sampleColors(colorRandom, seedClassSpec);
  const planetRotation =
    family === "planet"
      ? samplePlanetRotation(pageSeed, ordinal, attempt)
      : null;
  const volumetricMotion =
    family === "planet"
      ? null
      : sampleVolumetricMotion(pageSeed, ordinal, attempt, family);
  const vector = [
    ...traitsA,
    ...traitsB,
    ...colors.normalized,
    ...(planetRotation ? planetRotationVector(planetRotation) : []),
    ...(volumetricMotion ? volumetricMotionVector(volumetricMotion) : []),
  ];
  const depth = 0.08 + placement.next() * 0.92;
  const drift = sampleRange(motion, config.driftRange) * (0.3 + depth);
  return {
    identity: objectIdentity(pageSeed, ordinal),
    spawnOrdinal: ordinal,
    family,
    seedClass,
    seedClassName: seedClassSpec.name,
    visualSignature: visualSignature(family, seedClass, vector),
    visualVector: vector,
    x: width * (0.02 + placement.next() * 0.96),
    y: height * (1.1 - placement.next() * 1.275),
    depth,
    size: sizeForDepth(depth, config) * (0.78 + placement.next() * 0.44),
    intensity: intensityForDepth(depth, config),
    detail: detailForDepth(depth, config),
    speed: speedForDepth(depth, config),
    drift,
    rotation: 0,
    spin: 0,
    planetRotation,
    volumetricMotion,
    phase: motion.next() * 20,
    age: motion.next() * 90,
    traitsA,
    traitsB,
    colorLow: colors.low,
    colorMid: colors.mid,
    colorHigh: colors.high,
  };
}

function minimumDistance(
  candidate: RainInstance,
  active: readonly RainInstance[],
): number {
  const comparable = active.filter(
    (instance) =>
      instance.family === candidate.family &&
      instance.seedClass === candidate.seedClass,
  );
  if (comparable.length === 0) return Number.POSITIVE_INFINITY;
  return Math.min(
    ...comparable.map((instance) =>
      visualDistance(candidate.visualVector, instance.visualVector),
    ),
  );
}

export function spawnRainInstance(options: {
  pageSeed: PageSeed;
  ordinal: bigint;
  config?: SpaceRainConfig;
  active?: readonly RainInstance[];
  family?: RainFamily;
  seedClass?: number;
  width?: number;
  height?: number;
}): RainInstance {
  const config = options.config ?? DEFAULT_SPACE_RAIN_CONFIG;
  const width = options.width ?? config.referenceWidth;
  const height = options.height ?? config.referenceHeight;
  const classRandom = randomFor(options.pageSeed, options.ordinal, "class");
  const family = options.family ?? familyFor(classRandom, config);
  const seedClass =
    options.seedClass ?? pickWeighted(classRandom, config.classWeights[family]);
  const active = options.active ?? [];
  let best: RainInstance | null = null;
  let bestDistance = -1;
  for (let attempt = 0; attempt < 32; attempt += 1) {
    const candidate = buildCandidate({
      pageSeed: options.pageSeed,
      ordinal: options.ordinal,
      config,
      family,
      seedClass,
      attempt,
      width,
      height,
    });
    const distance = minimumDistance(candidate, active);
    if (distance >= config.visualSeparation) return candidate;
    if (distance > bestDistance) {
      best = candidate;
      bestDistance = distance;
    }
  }
  if (!best) throw new Error("Unable to create a SpaceRain object");
  const offset = Number(options.ordinal % 997n) / 997;
  const traitsA: TraitVector = [
    fract(best.traitsA[0] + offset),
    best.traitsA[1],
    best.traitsA[2],
    best.traitsA[3],
  ];
  const vector = [traitsA[0], ...best.visualVector.slice(1)];
  return {
    ...best,
    traitsA,
    visualVector: vector,
    visualSignature: visualSignature(family, seedClass, vector),
  };
}

export function sceneCount(
  width: number,
  height: number,
  reducedMotion: boolean,
  config = DEFAULT_SPACE_RAIN_CONFIG,
): number {
  const referenceCount = reducedMotion
    ? config.referenceReducedMotionCount
    : config.referenceCount;
  const count = Math.round(
    (Math.max(0, width) * Math.max(0, height) * referenceCount) /
      (config.referenceWidth * config.referenceHeight),
  );
  return Math.min(count, config.maxActiveCount);
}

function forcedClass(
  reviewMode: ReviewMode | null,
  ordinal: bigint,
): { family?: RainFamily; seedClass?: number } {
  if (!reviewMode || reviewMode === "rain") return {};
  if (reviewMode === "ufo" || reviewMode === "black-hole") return {};
  if (reviewMode === "spiral") return { family: "galaxy", seedClass: 0 };
  if (reviewMode === "barred-spiral") {
    return { family: "galaxy", seedClass: 1 };
  }
  return {
    family: reviewMode,
    seedClass: Number(ordinal % BigInt(SEED_CLASSES[reviewMode].length)),
  };
}

function isInspectionMode(reviewMode: ReviewMode | null): boolean {
  return Boolean(
    reviewMode &&
    reviewMode !== "rain" &&
    reviewMode !== "ufo" &&
    reviewMode !== "black-hole",
  );
}

function desiredSceneCount(
  width: number,
  height: number,
  reducedMotion: boolean,
  reviewMode: ReviewMode | null,
  config: SpaceRainConfig,
): number {
  if (reviewMode === "ufo") return 0;
  if (isInspectionMode(reviewMode)) {
    return reducedMotion
      ? config.referenceReducedMotionCount
      : config.referenceCount;
  }
  return sceneCount(width, height, reducedMotion, config);
}

function placeAboveTop(
  instance: RainInstance,
  scene: SpaceRainScene,
  ordinal: bigint,
): void {
  const placement = randomFor(scene.pageSeed, ordinal, "recycle-placement");
  instance.y = -sampleRange(placement, scene.config.spawnOffsetRange);
}

function placeReviewInstance(
  instance: RainInstance,
  scene: SpaceRainScene,
  index: number,
): void {
  if (
    !scene.reviewMode ||
    scene.reviewMode === "rain" ||
    scene.reviewMode === "spiral" ||
    scene.reviewMode === "barred-spiral" ||
    scene.reviewMode === "ufo" ||
    scene.reviewMode === "black-hole"
  ) {
    return;
  }
  const columns = SEED_CLASSES[scene.reviewMode].length;
  const column = index % columns;
  const row = Math.floor(index / columns);
  instance.x =
    scene.width * (0.09 + (column / Math.max(1, columns - 1)) * 0.82);
  instance.y = scene.height * (-0.025 + (row % 5) * 0.24);
}

function appendInstance(
  scene: SpaceRainScene,
  placement: "warm" | "top" = "warm",
): RainInstance {
  const ordinal = scene.nextSpawnOrdinal;
  const forced = forcedClass(scene.reviewMode, ordinal);
  const instance = spawnRainInstance({
    pageSeed: scene.pageSeed,
    ordinal,
    config: scene.config,
    active: scene.instances,
    width: scene.width,
    height: scene.height,
    ...forced,
  });
  scene.nextSpawnOrdinal += 1n;
  if (placement === "top") placeAboveTop(instance, scene, ordinal);
  return instance;
}

function nextSpawnThreshold(scene: SpaceRainScene): number {
  return sampleRange(
    randomFor(scene.pageSeed, scene.nextSpawnOrdinal, "spawn-cadence"),
    scene.config.spawnIntervalJitterRange,
  );
}

function replaceAtTop(scene: SpaceRainScene, index: number): void {
  const ordinal = scene.nextSpawnOrdinal;
  const forced = forcedClass(scene.reviewMode, ordinal);
  const active = scene.instances.filter(
    (_candidate, candidateIndex) => candidateIndex !== index,
  );
  const replacement = spawnRainInstance({
    pageSeed: scene.pageSeed,
    ordinal,
    config: scene.config,
    active,
    width: scene.width,
    height: scene.height,
    ...forced,
  });
  scene.nextSpawnOrdinal += 1n;
  placeAboveTop(replacement, scene, ordinal);
  scene.instances[index] = replacement;
  scene.nextSpawnThreshold = nextSpawnThreshold(scene);
  scene.recycleCount += 1;
}

function reconcileStaticScene(scene: SpaceRainScene): void {
  if (scene.instances.length > scene.targetCount) {
    scene.instances.length = scene.targetCount;
  }
  while (scene.instances.length < scene.targetCount) {
    const instance = appendInstance(scene);
    placeReviewInstance(instance, scene, scene.instances.length);
    scene.instances.push(instance);
  }
  scene.nextSpawnThreshold = nextSpawnThreshold(scene);
  scene.spawnCredit = 0;
}

export function createScene(options: {
  pageSeed: PageSeed;
  width: number;
  height: number;
  reducedMotion?: boolean;
  reviewMode?: ReviewMode | null;
  ufoTraversalClass?: UfoTraversalClass | null;
  config?: SpaceRainConfig;
}): SpaceRainScene {
  const config = validateConfig(options.config ?? DEFAULT_SPACE_RAIN_CONFIG);
  const reducedMotion = options.reducedMotion ?? false;
  const reviewMode = options.reviewMode ?? null;
  const scene: SpaceRainScene = {
    pageSeed: options.pageSeed,
    pageSeedId: pageSeedIdentifier(options.pageSeed),
    nextSpawnOrdinal: 0n,
    config,
    width: options.width,
    height: options.height,
    reducedMotion,
    reviewMode,
    instances: [],
    targetCount: 0,
    spawnCredit: 0,
    nextSpawnThreshold: 1,
    recycleCount: 0,
    ufoScheduler: createUfoScheduler({
      pageSeed: options.pageSeed,
      reviewMode: reviewMode === "ufo",
      reducedMotion: reducedMotion || reviewMode === "black-hole",
      forcedTraversalClass: options.ufoTraversalClass ?? null,
    }),
    blackHoleScheduler: createBlackHoleScheduler({
      pageSeed: options.pageSeed,
      reviewMode: reviewMode === "black-hole",
      reducedMotion,
    }),
  };
  const count = desiredSceneCount(
    options.width,
    options.height,
    reducedMotion,
    reviewMode,
    config,
  );
  scene.targetCount = count;
  while (scene.instances.length < count) {
    const instance = appendInstance(scene);
    placeReviewInstance(instance, scene, scene.instances.length);
    scene.instances.push(instance);
  }
  scene.nextSpawnThreshold = nextSpawnThreshold(scene);
  return scene;
}

export function resizeScene(
  scene: SpaceRainScene,
  width: number,
  height: number,
): void {
  const shrinking = width < scene.width || height < scene.height;
  scene.width = width;
  scene.height = height;
  scene.targetCount = desiredSceneCount(
    width,
    height,
    scene.reducedMotion,
    scene.reviewMode,
    scene.config,
  );
  if (scene.reducedMotion || isInspectionMode(scene.reviewMode)) {
    reconcileStaticScene(scene);
    return;
  }
  if (shrinking && !isInspectionMode(scene.reviewMode)) {
    const { horizontalMargin, exitMargin } = scene.config;
    scene.instances = scene.instances.filter(
      (instance) =>
        instance.x >= -horizontalMargin &&
        instance.x <= width + horizontalMargin &&
        instance.y <= height + exitMargin,
    );
  }
}

export function setReducedMotion(
  scene: SpaceRainScene,
  reducedMotion: boolean,
): void {
  scene.reducedMotion = reducedMotion;
  setUfoReducedMotion(
    scene.ufoScheduler,
    reducedMotion || scene.reviewMode === "black-hole",
  );
  setBlackHoleReducedMotion(scene.blackHoleScheduler, reducedMotion);
  resizeScene(scene, scene.width, scene.height);
}

export function advanceScene(
  scene: SpaceRainScene,
  elapsedSeconds: number,
): void {
  if (scene.reducedMotion) return;
  for (let index = scene.instances.length - 1; index >= 0; index -= 1) {
    const instance = scene.instances[index];
    if (!instance) continue;
    instance.y += instance.speed * elapsedSeconds;
    instance.x += instance.drift * elapsedSeconds;
    if (instance.planetRotation) {
      advancePlanetRotation(instance.planetRotation, elapsedSeconds);
    } else if (instance.volumetricMotion) {
      advanceVolumetricMotion(instance.volumetricMotion, elapsedSeconds);
    } else {
      instance.rotation += instance.spin * elapsedSeconds;
    }
    instance.age += elapsedSeconds;
    if (instance.y > scene.height + scene.config.exitMargin) {
      if (
        isInspectionMode(scene.reviewMode) ||
        scene.instances.length <= scene.targetCount
      ) {
        replaceAtTop(scene, index);
      } else {
        scene.instances.splice(index, 1);
      }
    } else if (instance.x < -scene.config.horizontalMargin) {
      instance.x = scene.width + scene.config.horizontalMargin;
    } else if (instance.x > scene.width + scene.config.horizontalMargin) {
      instance.x = -scene.config.horizontalMargin;
    }
  }
  if (
    !isInspectionMode(scene.reviewMode) &&
    scene.reviewMode !== "ufo" &&
    scene.instances.length < scene.targetCount
  ) {
    scene.spawnCredit +=
      elapsedSeconds *
      scene.config.referenceSpawnRate *
      (scene.width / scene.config.referenceWidth);
    while (
      scene.instances.length < scene.targetCount &&
      scene.spawnCredit >= scene.nextSpawnThreshold
    ) {
      scene.spawnCredit -= scene.nextSpawnThreshold;
      scene.instances.push(appendInstance(scene, "top"));
      scene.nextSpawnThreshold = nextSpawnThreshold(scene);
    }
  } else {
    scene.spawnCredit = 0;
  }
  advanceUfoScheduler(scene.ufoScheduler, elapsedSeconds);
  advanceBlackHoleScheduler(scene.blackHoleScheduler, elapsedSeconds);
}

export function familyHistogram(
  instances: readonly RainInstance[],
): Record<RainFamily, number> {
  const result: Record<RainFamily, number> = {
    planet: 0,
    galaxy: 0,
    nebula: 0,
  };
  for (const instance of instances) result[instance.family] += 1;
  return result;
}

export function seedClassHistogram(
  instances: readonly RainInstance[],
): Record<string, number> {
  const result: Record<string, number> = {};
  for (const instance of instances) {
    const key = `${instance.family}:${instance.seedClassName}`;
    result[key] = (result[key] ?? 0) + 1;
  }
  return result;
}

export function activeVisualSeparation(scene: SpaceRainScene): number {
  let minimum = Number.POSITIVE_INFINITY;
  for (let first = 0; first < scene.instances.length; first += 1) {
    const a = scene.instances[first];
    if (!a) continue;
    for (let second = first + 1; second < scene.instances.length; second += 1) {
      const b = scene.instances[second];
      if (!b || a.family !== b.family || a.seedClass !== b.seedClass) continue;
      minimum = Math.min(
        minimum,
        visualDistance(a.visualVector, b.visualVector),
      );
    }
  }
  return minimum;
}

export function parameterFingerprint(instance: RainInstance): string {
  return [
    instance.family,
    instance.seedClass,
    ...instance.traitsA,
    ...instance.traitsB,
    ...instance.colorLow,
    ...instance.colorMid,
    ...instance.colorHigh,
    instance.x,
    instance.y,
    instance.depth,
    instance.rotation,
    instance.phase,
    ...(instance.planetRotation
      ? [
          instance.planetRotation.axisInclination,
          instance.planetRotation.axisPosition,
          instance.planetRotation.surfaceAngle,
          instance.planetRotation.surfaceRate,
          instance.planetRotation.cloudAngle,
          instance.planetRotation.cloudRate,
          instance.planetRotation.ringAngle,
          instance.planetRotation.ringRate,
        ]
      : []),
    ...(instance.volumetricMotion
      ? [
          instance.volumetricMotion.axisInclination,
          instance.volumetricMotion.axisPosition,
          instance.volumetricMotion.bulkAngle,
          instance.volumetricMotion.bulkRate,
          instance.volumetricMotion.internalAngle,
          instance.volumetricMotion.internalRate,
          instance.volumetricMotion.evolutionAngle,
          instance.volumetricMotion.evolutionRate,
        ]
      : []),
  ].join(":");
}
