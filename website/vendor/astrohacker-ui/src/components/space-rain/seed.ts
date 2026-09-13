export type PageSeed = readonly [number, number, number, number];

export interface PageSeedInputs {
  timeOrigin: number;
  now: number;
  entropy: readonly number[];
}

export interface SeededRandom {
  next: () => number;
  nextUint32: () => number;
}

const UINT32_RANGE = 4_294_967_296;

function mix32(value: number): number {
  let mixed = value >>> 0;
  mixed ^= mixed >>> 16;
  mixed = Math.imul(mixed, 0x7feb_352d);
  mixed ^= mixed >>> 15;
  mixed = Math.imul(mixed, 0x846c_a68b);
  mixed ^= mixed >>> 16;
  return mixed >>> 0;
}

function numberWords(value: number): readonly [number, number] {
  const integer = Math.max(0, Math.floor(value));
  return [integer >>> 0, Math.floor(integer / UINT32_RANGE) >>> 0];
}

export function createPageSeed(inputs: PageSeedInputs): PageSeed {
  if (!Number.isFinite(inputs.timeOrigin) || !Number.isFinite(inputs.now)) {
    throw new Error("SpaceRain page-seed clocks must be finite");
  }
  if (inputs.entropy.length < 4) {
    throw new Error(
      "SpaceRain page seed requires at least 128 bits of entropy",
    );
  }
  const origin = numberWords(inputs.timeOrigin);
  const now = numberWords(inputs.now);
  const source = [origin[0], origin[1], now[0], now[1]];
  const words = source.map((clockWord, index) =>
    mix32(
      clockWord ^
        (inputs.entropy[index] ?? 0) ^
        Math.imul(index + 1, 0x9e37_79b9),
    ),
  ) as [number, number, number, number];
  for (let round = 0; round < 4; round += 1) {
    const previous = [...words] as [number, number, number, number];
    const neighbors = [
      previous[1],
      previous[2],
      previous[3],
      previous[0],
    ] as const;
    for (const index of [0, 1, 2, 3] as const) {
      words[index] = mix32(
        previous[index] ^ neighbors[index] ^ Math.imul(round + 1, 0x85eb_ca6b),
      );
    }
  }
  if (words.every((word) => word === 0)) words[0] = 1;
  return words;
}

export function freshPageSeed(): PageSeed {
  const entropy = new Uint32Array(4);
  crypto.getRandomValues(entropy);
  return createPageSeed({
    timeOrigin: performance.timeOrigin,
    now: Date.now(),
    entropy: [...entropy],
  });
}

export function pageSeedIdentifier(seed: PageSeed): string {
  return seed.map((word) => word.toString(16).padStart(8, "0")).join("");
}

export function pageSeedFromHex(value: string | null): PageSeed | null {
  if (!value || !/^[0-9a-f]{32}$/i.test(value)) return null;
  return [0, 1, 2, 3].map((index) =>
    Number.parseInt(value.slice(index * 8, index * 8 + 8), 16),
  ) as [number, number, number, number];
}

function domainWord(domain: string): number {
  let hash = 0x811c_9dc5;
  for (let index = 0; index < domain.length; index += 1) {
    hash ^= domain.charCodeAt(index);
    hash = Math.imul(hash, 0x0100_0193);
  }
  return hash >>> 0;
}

function ordinalWords(ordinal: bigint): readonly [number, number] {
  return [
    Number(ordinal & 0xffff_ffffn) >>> 0,
    Number((ordinal >> 32n) & 0xffff_ffffn) >>> 0,
  ];
}

export function randomFor(
  pageSeed: PageSeed,
  ordinal: bigint,
  domain: string,
): SeededRandom {
  const [low, high] = ordinalWords(ordinal);
  const domainSeed = domainWord(domain);
  const ordinalA = mix32(low ^ Math.imul(high, 0x9e37_79b9));
  const ordinalB = mix32(high ^ Math.imul(low, 0x85eb_ca6b));
  let a = mix32(pageSeed[0] ^ ordinalA ^ domainSeed);
  let b = mix32(pageSeed[1] ^ ordinalB ^ Math.imul(domainSeed, 3));
  let c = mix32(pageSeed[2] ^ ordinalB ^ Math.imul(domainSeed, 5));
  let d = mix32(pageSeed[3] ^ ordinalA ^ Math.imul(domainSeed, 7));
  if ((a | b | c | d) === 0) d = 1;

  const nextUint32 = (): number => {
    const result = Math.imul(((b * 5) << 7) | ((b * 5) >>> 25), 9) >>> 0;
    const temporary = (b << 9) >>> 0;
    c ^= a;
    d ^= b;
    b ^= c;
    a ^= d;
    c ^= temporary;
    d = ((d << 11) | (d >>> 21)) >>> 0;
    return result;
  };

  return {
    next: () => nextUint32() / UINT32_RANGE,
    nextUint32,
  };
}

export function objectIdentity(pageSeed: PageSeed, ordinal: bigint): string {
  return `${pageSeedIdentifier(pageSeed)}:${ordinal
    .toString(16)
    .padStart(16, "0")}`;
}

export function derivePageSeed(
  pageSeed: PageSeed,
  ordinal: bigint,
  domain: string,
): PageSeed {
  const random = randomFor(pageSeed, ordinal, domain);
  return [
    random.nextUint32(),
    random.nextUint32(),
    random.nextUint32(),
    random.nextUint32(),
  ];
}
