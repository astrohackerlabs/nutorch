export interface SeedClassSpec<Name extends string = string> {
  name: Name;
  hueRange: readonly [number, number];
  saturationRange: readonly [number, number];
  lightnessRange: readonly [number, number];
}

export const PLANET_SEED_CLASSES = [
  {
    name: "rocky",
    hueRange: [12, 52],
    saturationRange: [0.32, 0.72],
    lightnessRange: [0.24, 0.58],
  },
  {
    name: "gas-giant",
    hueRange: [18, 315],
    saturationRange: [0.38, 0.78],
    lightnessRange: [0.34, 0.7],
  },
  {
    name: "ringed",
    hueRange: [28, 225],
    saturationRange: [0.3, 0.68],
    lightnessRange: [0.34, 0.68],
  },
  {
    name: "ocean-ice",
    hueRange: [172, 226],
    saturationRange: [0.42, 0.82],
    lightnessRange: [0.3, 0.68],
  },
  {
    name: "lava",
    hueRange: [350, 34],
    saturationRange: [0.66, 0.96],
    lightnessRange: [0.25, 0.6],
  },
] as const satisfies readonly SeedClassSpec[];

export const GALAXY_SEED_CLASSES = [
  {
    name: "spiral",
    hueRange: [188, 282],
    saturationRange: [0.38, 0.78],
    lightnessRange: [0.36, 0.7],
  },
  {
    name: "barred-spiral",
    hueRange: [205, 324],
    saturationRange: [0.4, 0.82],
    lightnessRange: [0.34, 0.7],
  },
  {
    name: "elliptical",
    hueRange: [18, 58],
    saturationRange: [0.28, 0.62],
    lightnessRange: [0.42, 0.76],
  },
  {
    name: "agn-quasar",
    hueRange: [184, 256],
    saturationRange: [0.5, 0.9],
    lightnessRange: [0.42, 0.76],
  },
] as const satisfies readonly SeedClassSpec[];

export const NEBULA_SEED_CLASSES = [
  {
    name: "emission",
    hueRange: [318, 24],
    saturationRange: [0.54, 0.92],
    lightnessRange: [0.3, 0.66],
  },
  {
    name: "reflection-dark",
    hueRange: [188, 246],
    saturationRange: [0.38, 0.8],
    lightnessRange: [0.2, 0.56],
  },
  {
    name: "planetary",
    hueRange: [146, 218],
    saturationRange: [0.44, 0.84],
    lightnessRange: [0.32, 0.7],
  },
  {
    name: "supernova-remnant",
    hueRange: [348, 218],
    saturationRange: [0.56, 0.94],
    lightnessRange: [0.34, 0.72],
  },
] as const satisfies readonly SeedClassSpec[];

export const RAIN_FAMILIES = ["planet", "galaxy", "nebula"] as const;
export const REVIEW_MODES = [
  ...RAIN_FAMILIES,
  "spiral",
  "barred-spiral",
  "ufo",
  "black-hole",
  "rain",
] as const;

export type RainFamily = (typeof RAIN_FAMILIES)[number];
export type ReviewMode = (typeof REVIEW_MODES)[number];

export const SEED_CLASSES = {
  planet: PLANET_SEED_CLASSES,
  galaxy: GALAXY_SEED_CLASSES,
  nebula: NEBULA_SEED_CLASSES,
} as const satisfies Readonly<Record<RainFamily, readonly SeedClassSpec[]>>;

export interface SpaceRainConfig {
  maxDpr: number;
  referenceWidth: number;
  referenceHeight: number;
  referenceCount: number;
  referenceReducedMotionCount: number;
  maxActiveCount: number;
  referenceSpawnRate: number;
  spawnIntervalJitterRange: readonly [number, number];
  familyWeights: Readonly<Record<RainFamily, number>>;
  classWeights: Readonly<Record<RainFamily, readonly number[]>>;
  sizeRange: readonly [number, number];
  speedRange: readonly [number, number];
  intensityRange: readonly [number, number];
  detailRange: readonly [number, number];
  driftRange: readonly [number, number];
  rotationRange: readonly [number, number];
  visualSeparation: number;
  spawnOffsetRange: readonly [number, number];
  exitMargin: number;
  horizontalMargin: number;
}

export const SPACE_RAIN_REFERENCE_WIDTH = 1_440;
export const SPACE_RAIN_REFERENCE_HEIGHT = 900;
export const SPACE_RAIN_REFERENCE_COUNT = 36;
export const SPACE_RAIN_REFERENCE_REDUCED_MOTION_COUNT = 12;
export const SPACE_RAIN_STAR_CELL_PX = SPACE_RAIN_REFERENCE_HEIGHT / (2 * 96);
export const SPACE_RAIN_STAR_CORE_PX = SPACE_RAIN_STAR_CELL_PX * 0.075;
export const SPACE_RAIN_DUST_CELL_PX = SPACE_RAIN_REFERENCE_HEIGHT / (2 * 220);
export const SPACE_RAIN_STAR_DRIFT_PX = [2.7, -8.1] as const;

export const DEFAULT_SPACE_RAIN_CONFIG: SpaceRainConfig = {
  maxDpr: 1.75,
  referenceWidth: SPACE_RAIN_REFERENCE_WIDTH,
  referenceHeight: SPACE_RAIN_REFERENCE_HEIGHT,
  referenceCount: SPACE_RAIN_REFERENCE_COUNT,
  referenceReducedMotionCount: SPACE_RAIN_REFERENCE_REDUCED_MOTION_COUNT,
  maxActiveCount: 256,
  referenceSpawnRate: 1.1,
  spawnIntervalJitterRange: [0.55, 1.45],
  familyWeights: {
    planet: 0.257158,
    galaxy: 0.5214,
    nebula: 0.221442,
  },
  classWeights: {
    planet: [0.22, 0.2, 0.2, 0.19, 0.19],
    galaxy: [0.392406, 0.341772, 0.158228, 0.107594],
    nebula: [0.31, 0.25, 0.25, 0.19],
  },
  sizeRange: [20, 100],
  speedRange: [15.75, 108],
  intensityRange: [0.26, 0.88],
  detailRange: [0.45, 1],
  driftRange: [-12.96, 12.96],
  rotationRange: [-0.22, 0.22],
  visualSeparation: 0.12,
  spawnOffsetRange: [72, 180],
  exitMargin: 126,
  horizontalMargin: 115.2,
};

function total(values: readonly number[]): number {
  return values.reduce((sum, value) => sum + value, 0);
}

function validRange(range: readonly [number, number]): boolean {
  return range.every(Number.isFinite) && range[0] <= range[1];
}

export function validateConfig(config: SpaceRainConfig): SpaceRainConfig {
  const finitePositive = [
    config.maxDpr,
    config.referenceWidth,
    config.referenceHeight,
    config.referenceCount,
    config.referenceReducedMotionCount,
    config.maxActiveCount,
    config.referenceSpawnRate,
    config.exitMargin,
    config.horizontalMargin,
    config.visualSeparation,
  ].every((value) => Number.isFinite(value) && value > 0);
  if (!finitePositive) throw new Error("SpaceRain limits must be positive");
  if (config.maxDpr > 1.75) throw new Error("SpaceRain DPR cap exceeds 1.75");
  if (config.referenceReducedMotionCount > config.referenceCount) {
    throw new Error("SpaceRain reduced-motion cap exceeded");
  }
  const fourKCount = Math.round(
    (3_840 * 2_160 * config.referenceCount) /
      (config.referenceWidth * config.referenceHeight),
  );
  if (config.maxActiveCount < fourKCount) {
    throw new Error("SpaceRain instance cap does not cover 4K");
  }
  if (config.visualSeparation > 0.35) {
    throw new Error("SpaceRain visual separation is too large");
  }
  for (const range of [
    config.spawnIntervalJitterRange,
    config.sizeRange,
    config.speedRange,
    config.intensityRange,
    config.detailRange,
    config.driftRange,
    config.rotationRange,
    config.spawnOffsetRange,
  ]) {
    if (!validRange(range)) throw new Error("Invalid SpaceRain range");
  }
  if (
    config.spawnIntervalJitterRange[0] <= 0 ||
    config.spawnOffsetRange[0] <= 0
  ) {
    throw new Error("SpaceRain spawn ranges must be positive");
  }

  const familyTotal = total(
    RAIN_FAMILIES.map((family) => config.familyWeights[family]),
  );
  if (RAIN_FAMILIES.some((family) => config.familyWeights[family] < 0)) {
    throw new Error("SpaceRain family weights cannot be negative");
  }
  if (Math.abs(familyTotal - 1) > 0.000_001) {
    throw new Error("SpaceRain family weights must sum to one");
  }

  for (const family of RAIN_FAMILIES) {
    const weights = config.classWeights[family];
    if (
      weights.some((weight) => weight < 0) ||
      weights.length !== SEED_CLASSES[family].length ||
      Math.abs(total(weights) - 1) > 0.000_001
    ) {
      throw new Error(`Invalid ${family} seed-class weights`);
    }
    for (const seedClass of SEED_CLASSES[family]) {
      if (
        !validRange(seedClass.saturationRange) ||
        !validRange(seedClass.lightnessRange)
      ) {
        throw new Error(`Invalid ${family} seed-class color range`);
      }
    }
  }
  return config;
}

export function reviewModeFromSearch(search: string): ReviewMode | null {
  const value = new URLSearchParams(search).get("space-rain-review");
  return REVIEW_MODES.find((mode) => mode === value) ?? null;
}
