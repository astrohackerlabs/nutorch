import {
  SPACE_RAIN_DUST_CELL_PX,
  SPACE_RAIN_STAR_CELL_PX,
  SPACE_RAIN_STAR_CORE_PX,
  SPACE_RAIN_STAR_DRIFT_PX,
} from "./config";

function objectVertexShader(planet: boolean): string {
  const pointSetup = planet
    ? `float extent = aMotion.y > 1.5 && aMotion.y < 2.5 ? 1.58 : 1.0;
  vec2 localPoint = aCorner * extent;
  vec2 point = localPoint;`
    : `float extent = 1.18;
  vec2 localPoint = aCorner * extent;
  vec2 point = localPoint;`;
  return `#version 300 es
precision highp float;
layout(location=0) in vec2 aCorner;
layout(location=1) in vec4 aPlacement;
layout(location=2) in vec4 aMotion;
layout(location=3) in vec4 aTraitsA;
layout(location=4) in vec4 aTraitsB;
layout(location=5) in vec4 aColorLowDetail;
layout(location=6) in vec4 aColorMidAge;
layout(location=7) in vec4 aColorHigh;
layout(location=8) in vec4 aSpatialState;
uniform vec2 uResolution;
uniform float uTime;
out vec2 vPoint;
out float vDepth;
out float vSeedClass;
out float vPhase;
out float vRotation;
out float vIntensity;
out float vDetail;
out float vAge;
out vec4 vTraitsA;
out vec4 vTraitsB;
out vec3 vColorLow;
out vec3 vColorMid;
out vec3 vColorHigh;
out vec4 vSpatialState;

void main() {
  ${pointSetup}
  float radius = aPlacement.w;
  vec2 clipRadius = vec2(radius * 2.0 / uResolution.x, radius * 2.0 / uResolution.y);
  gl_Position = vec4(aPlacement.xy + point * clipRadius, 0.0, 1.0);
  vPoint = localPoint;
  vDepth = aPlacement.z;
  vSeedClass = aMotion.y;
  vPhase = aMotion.z;
  vRotation = aMotion.x;
  vIntensity = aMotion.w;
  vTraitsA = aTraitsA;
  vTraitsB = aTraitsB;
  vColorLow = aColorLowDetail.xyz;
  vDetail = aColorLowDetail.w;
  vColorMid = aColorMidAge.xyz;
  vAge = aColorMidAge.w;
  vColorHigh = aColorHigh.xyz;
  vSpatialState = aSpatialState;
}`;
}

export const OBJECT_VERTEX_SHADER = objectVertexShader(false);
export const PLANET_VERTEX_SHADER = objectVertexShader(true);

export const BACKDROP_VERTEX_SHADER = `#version 300 es
precision highp float;
layout(location=0) in vec2 aCorner;
out vec2 vPoint;
void main() {
  vPoint = aCorner;
  gl_Position = vec4(aCorner, 0.0, 1.0);
}`;

const COMMON_FRAGMENT = `
precision highp float;
in vec2 vPoint;
in float vDepth;
in float vSeedClass;
in float vPhase;
in float vRotation;
in float vIntensity;
in float vDetail;
in float vAge;
in vec4 vTraitsA;
in vec4 vTraitsB;
in vec3 vColorLow;
in vec3 vColorMid;
in vec3 vColorHigh;
in vec4 vSpatialState;
uniform vec2 uResolution;
uniform float uTime;
out vec4 outColor;

#define PI 3.14159265359

float hash21(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

float valueNoise(vec2 p) {
  vec2 cell = floor(p);
  vec2 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash21(cell), hash21(cell + vec2(1.0, 0.0)), f.x),
             mix(hash21(cell + vec2(0.0, 1.0)), hash21(cell + 1.0), f.x), f.y);
}

float fbm(vec2 p) {
  float value = 0.0;
  float amplitude = 0.52;
  mat2 turn = mat2(0.80, -0.60, 0.60, 0.80);
  for (int i = 0; i < 4; i++) {
    value += amplitude * valueNoise(p);
    p = turn * p * 2.03 + 7.13;
    amplitude *= 0.49;
  }
  return value;
}

mat2 rotate2(float angle) {
  float c = cos(angle);
  float s = sin(angle);
  return mat2(c, -s, s, c);
}

vec3 spectrum(float t) {
  return mix(mix(vColorLow, vColorMid, smoothstep(0.0, 0.58, t)),
             vColorHigh, smoothstep(0.56, 1.0, t));
}

float ellipse(vec2 p, vec2 radius) {
  return length(p / radius) - 1.0;
}

void paintTranslucent(vec3 color, float alpha) {
  alpha = clamp(alpha * vIntensity, 0.0, 1.0);
  outColor = vec4(color * alpha, alpha);
}
`;

export const PLANET_FRAGMENT_SHADER = `#version 300 es
${COMMON_FRAGMENT}

float hash31(vec3 p) {
  p = fract(p * 0.1031);
  p += dot(p, p.yzx + 33.33);
  return fract((p.x + p.y) * p.z);
}

vec3 hash33(vec3 p) {
  p = fract(p * vec3(0.1031, 0.1030, 0.0973));
  p += dot(p, p.yxz + 33.33);
  return fract((p.xxy + p.yxx) * p.zyx);
}

float valueNoise3(vec3 p) {
  vec3 cell = floor(p);
  vec3 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float z0 = mix(
    mix(hash31(cell), hash31(cell + vec3(1.0, 0.0, 0.0)), f.x),
    mix(hash31(cell + vec3(0.0, 1.0, 0.0)), hash31(cell + vec3(1.0, 1.0, 0.0)), f.x),
    f.y
  );
  float z1 = mix(
    mix(hash31(cell + vec3(0.0, 0.0, 1.0)), hash31(cell + vec3(1.0, 0.0, 1.0)), f.x),
    mix(hash31(cell + vec3(0.0, 1.0, 1.0)), hash31(cell + vec3(1.0, 1.0, 1.0)), f.x),
    f.y
  );
  return mix(z0, z1, f.z);
}

float fbm3(vec3 p) {
  float value = 0.0;
  float amplitude = 0.52;
  mat3 turn = mat3(
     0.00,  0.80,  0.60,
    -0.80,  0.36, -0.48,
    -0.60, -0.48,  0.64
  );
  for (int i = 0; i < 4; i++) {
    value += amplitude * valueNoise3(p);
    p = turn * p * 2.03 + vec3(7.13, 3.71, 5.19);
    amplitude *= 0.49;
  }
  return value;
}

vec3 rotateBodyY(vec3 direction, float angle) {
  float c = cos(angle);
  float s = sin(angle);
  return vec3(
    c * direction.x + s * direction.z,
    direction.y,
    -s * direction.x + c * direction.z
  );
}

vec3 planetAxis() {
  float inclination = vSpatialState.x;
  float position = vSpatialState.y;
  float projected = sin(inclination);
  return normalize(vec3(
    projected * cos(position),
    projected * sin(position),
    cos(inclination)
  ));
}

mat3 planetBasis(vec3 axis) {
  vec3 reference = abs(axis.z) < 0.92 ? vec3(0.0, 0.0, 1.0) : vec3(0.0, 1.0, 0.0);
  vec3 east = normalize(cross(reference, axis));
  vec3 forward = normalize(cross(axis, east));
  return mat3(east, axis, forward);
}

vec3 bodyDirection(vec3 viewDirection, mat3 basis, float angle) {
  return normalize(rotateBodyY(transpose(basis) * viewDirection, -angle));
}

float craterField(vec3 direction) {
  float scale = mix(4.5, 10.5, vTraitsA.x);
  vec3 point = direction * scale + vec3(vTraitsA.y, vTraitsB.x, vTraitsB.y) * 41.0;
  vec3 cell = floor(point);
  vec3 local = fract(point) - 0.5;
  vec3 center = hash33(cell + vec3(vTraitsA.z, vTraitsA.w, vTraitsB.z) * 53.0) - 0.5;
  float radius = mix(0.08, 0.27, hash31(cell + vTraitsB.w * 71.0));
  float distanceToCenter = length(local - center);
  float bowl = 1.0 - smoothstep(radius * 0.35, radius, distanceToCenter);
  float floorCut = 1.0 - smoothstep(radius * 0.18, radius * 0.5, distanceToCenter);
  return max(0.0, bowl - floorCut * 0.72);
}

float ringField(vec2 p, vec3 axis, mat3 basis, out float planeDepth) {
  planeDepth = -(axis.x * p.x + axis.y * p.y) / max(axis.z, 0.08);
  vec3 planePoint = vec3(p, planeDepth);
  float planeRadius = length(planePoint);
  float innerRadius = mix(1.08, 1.2, vTraitsA.z);
  float outerRadius = mix(1.38, 1.54, vTraitsA.w);
  float edge = mix(0.012, 0.028, vTraitsB.w);
  float envelope = smoothstep(innerRadius - edge, innerRadius, planeRadius) *
                   (1.0 - smoothstep(outerRadius, outerRadius + edge, planeRadius));
  vec2 ringPoint = vec2(dot(planePoint, basis[0]), dot(planePoint, basis[2]));
  float azimuth = atan(ringPoint.y, ringPoint.x) + vSpatialState.w;
  float radialBands = 0.55 + 0.4 * sin(planeRadius * mix(74.0, 142.0, vTraitsB.x) + vPhase);
  float orbitingDetail = 0.76 + 0.24 * sin(azimuth * mix(2.0, 7.0, vTraitsB.y) + vTraitsA.x * 19.0);
  return envelope * radialBands * orbitingDetail;
}

void main() {
  vec2 p = vPoint;
  float disc = length(p);
  vec3 axis = planetAxis();
  mat3 basis = planetBasis(axis);
  float ring = 0.0;
  float ringDepth = -2.0;
  if (vSeedClass > 1.5 && vSeedClass < 2.5) {
    ring = ringField(p, axis, basis, ringDepth);
  }
  if (disc > 1.0) {
    if (ring < 0.01) discard;
    vec3 ringColor = mix(vColorLow, vColorHigh, clamp(ring, 0.0, 1.0));
    float ringAlpha = clamp(ring * mix(0.48, 0.78, vTraitsB.y) * vIntensity, 0.0, 0.88);
    outColor = vec4(ringColor * ringAlpha, ringAlpha);
    return;
  }

  vec3 viewNormal = vec3(p, sqrt(max(0.0, 1.0 - disc * disc)));
  vec3 surfaceDirection = bodyDirection(viewNormal, basis, vRotation);
  vec3 cloudDirection = bodyDirection(viewNormal, basis, vSpatialState.z);
  float lightAngle = mix(-2.7, 0.6, vTraitsB.x);
  vec3 lightDirection = normalize(vec3(cos(lightAngle), sin(lightAngle), mix(0.42, 0.92, vTraitsB.y)));
  float diffuse = max(0.0, dot(viewNormal, lightDirection));
  float rim = pow(1.0 - max(viewNormal.z, 0.0), mix(2.1, 4.2, vTraitsA.w));
  float latitude = asin(clamp(surfaceDirection.y, -1.0, 1.0)) / PI;
  vec3 materialOffset = vec3(vTraitsA.z, vTraitsB.x, vTraitsB.w) * 29.0;
  float terrain = fbm3(surfaceDirection * mix(1.8, 4.6, vTraitsA.x) + materialOffset);
  float fine = fbm3(surfaceDirection * mix(5.2, 10.8, vDetail) - materialOffset.yzx * 0.7);
  float cloudNoise = fbm3(cloudDirection * mix(3.2, 6.4, vTraitsB.z) + materialOffset.zxy * 1.3);
  vec3 color;

  if (vSeedClass < 0.5) {
    float craters = craterField(surfaceDirection);
    float strata = smoothstep(mix(0.36, 0.58, vTraitsB.x), mix(0.56, 0.78, vTraitsB.y), terrain);
    color = mix(vColorLow, vColorMid, strata);
    color = mix(color, vColorHigh, clamp(craters * mix(0.18, 0.52, vTraitsA.w) + fine * 0.12, 0.0, 0.62));
  } else if (vSeedClass < 1.5) {
    float bandCount = floor(mix(7.0, 19.0, vTraitsA.x));
    float bands = 0.5 + 0.5 * sin(latitude * PI * bandCount + terrain * mix(4.0, 13.0, vTraitsA.y));
    vec3 stormCenter = normalize(vec3(mix(-0.72, 0.72, vTraitsB.x), mix(-0.76, 0.76, vTraitsB.y), mix(0.35, 1.0, vTraitsB.z)));
    float storm = exp(-mix(12.0, 42.0, vTraitsB.z) * (1.0 - dot(surfaceDirection, stormCenter)));
    color = mix(vColorLow, vColorMid, bands);
    color = mix(color, vColorHigh, storm * mix(0.62, 0.94, vTraitsB.w));
    color = mix(color, vColorHigh, smoothstep(0.64, 0.84, cloudNoise) * 0.18);
  } else if (vSeedClass < 2.5) {
    float bands = 0.5 + 0.5 * sin(latitude * mix(35.0, 83.0, vTraitsA.x) + terrain * mix(3.0, 9.0, vTraitsA.y));
    color = mix(vColorLow, vColorMid, bands);
    color = mix(color, vColorHigh, fine * mix(0.08, 0.28, vTraitsB.y));
    color = mix(color, vColorHigh, smoothstep(0.68, 0.86, cloudNoise) * 0.12);
  } else if (vSeedClass < 3.5) {
    float land = smoothstep(mix(0.42, 0.56, vTraitsA.x), mix(0.55, 0.72, vTraitsA.y), terrain);
    float ice = smoothstep(mix(0.2, 0.38, vTraitsB.x), mix(0.42, 0.62, vTraitsB.y), abs(latitude));
    color = mix(vColorLow, vColorMid, land);
    color = mix(color, vColorHigh, ice);
    float cloud = smoothstep(mix(0.52, 0.68, vTraitsA.z), mix(0.7, 0.84, vTraitsA.w), cloudNoise);
    color = mix(color, vColorHigh, cloud * mix(0.24, 0.62, vTraitsB.w));
  } else {
    float crust = smoothstep(mix(0.4, 0.58, vTraitsA.x), mix(0.58, 0.76, vTraitsA.y), terrain + fine * 0.25);
    float cracks = pow(1.0 - abs(fine * 2.0 - 1.0), mix(5.0, 12.0, vTraitsB.x));
    color = mix(vColorHigh * (0.82 + cracks), vColorLow * 0.34, crust);
  }

  color *= 0.13 + diffuse * mix(0.68, 0.98, vTraitsB.z);
  color += vColorHigh * rim * mix(0.2, 0.5, vTraitsB.w);
  float veil = mix(0.34, 0.76, vIntensity);
  color *= veil;
  float bodyAlpha = 1.0 - smoothstep(0.975, 1.0, disc);
  vec3 ringColor = mix(vColorLow, vColorHigh, clamp(ring, 0.0, 1.0));
  bool ringFront = ringDepth > viewNormal.z + 0.002;

  if (bodyAlpha > 0.0) {
    if (ringFront) color = mix(color, ringColor * veil, ring * 0.72);
    outColor = vec4(color * bodyAlpha, bodyAlpha);
  } else {
    discard;
  }
}`;

const VOLUME_FRAGMENT = `
#define MAX_VOLUME_STEPS 20

float volumeHash31(vec3 p) {
  p = fract(p * 0.1031);
  p += dot(p, p.yzx + 33.33);
  return fract((p.x + p.y) * p.z);
}

float volumeNoise3(vec3 p) {
  vec3 cell = floor(p);
  vec3 f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float z0 = mix(
    mix(volumeHash31(cell), volumeHash31(cell + vec3(1.0, 0.0, 0.0)), f.x),
    mix(volumeHash31(cell + vec3(0.0, 1.0, 0.0)), volumeHash31(cell + vec3(1.0, 1.0, 0.0)), f.x),
    f.y
  );
  float z1 = mix(
    mix(volumeHash31(cell + vec3(0.0, 0.0, 1.0)), volumeHash31(cell + vec3(1.0, 0.0, 1.0)), f.x),
    mix(volumeHash31(cell + vec3(0.0, 1.0, 1.0)), volumeHash31(cell + vec3(1.0, 1.0, 1.0)), f.x),
    f.y
  );
  return mix(z0, z1, f.z);
}

float volumeFbm3(vec3 p) {
  float value = 0.0;
  float amplitude = 0.55;
  mat3 turn = mat3(
     0.00,  0.80,  0.60,
    -0.80,  0.36, -0.48,
    -0.60, -0.48,  0.64
  );
  for (int octave = 0; octave < 3; octave++) {
    value += amplitude * volumeNoise3(p);
    p = turn * p * 2.03 + vec3(7.13, 3.71, 5.19);
    amplitude *= 0.47;
  }
  return value;
}

vec3 rotateSpatialY(vec3 value, float angle) {
  float c = cos(angle);
  float s = sin(angle);
  return vec3(c * value.x + s * value.z, value.y, -s * value.x + c * value.z);
}

vec3 spatialAxis() {
  float projected = sin(vSpatialState.x);
  return normalize(vec3(
    projected * cos(vSpatialState.y),
    projected * sin(vSpatialState.y),
    cos(vSpatialState.x)
  ));
}

mat3 spatialBasis(vec3 axis) {
  vec3 reference = abs(axis.z) < 0.92 ? vec3(0.0, 0.0, 1.0) : vec3(0.0, 1.0, 0.0);
  vec3 east = normalize(cross(reference, axis));
  vec3 forward = normalize(cross(axis, east));
  return mat3(east, axis, forward);
}

vec3 viewToVolumePoint(vec3 point, mat3 basis) {
  return rotateSpatialY(transpose(basis) * point, -vRotation);
}

bool ellipsoidInterval(
  vec3 origin,
  vec3 direction,
  vec3 radii,
  out float entry,
  out float exit
) {
  vec3 scaledOrigin = origin / radii;
  vec3 scaledDirection = direction / radii;
  float a = dot(scaledDirection, scaledDirection);
  float b = 2.0 * dot(scaledOrigin, scaledDirection);
  float c = dot(scaledOrigin, scaledOrigin) - 1.0;
  float discriminant = b * b - 4.0 * a * c;
  if (discriminant < 0.0 || a < 0.000001) return false;
  float root = sqrt(discriminant);
  entry = (-b - root) / (2.0 * a);
  exit = (-b + root) / (2.0 * a);
  return exit > max(entry, 0.0);
}

int volumeStepCount() {
  float amount = clamp(vDepth * 0.58 + vDetail * 0.42, 0.0, 1.0);
  return int(floor(mix(6.0, 20.0, amount) + 0.5));
}

float stableVolumeJitter() {
  vec2 objectPixel = floor((vPoint + vec2(1.25)) * 257.0);
  return hash21(objectPixel + vec2(vPhase * 17.0, vTraitsA.x * 41.0));
}

void paintVolume(vec3 premultipliedColor, float alpha) {
  if (alpha < 0.002) discard;
  float finalAlpha = clamp(alpha * vIntensity, 0.0, 0.94);
  vec3 straightColor = premultipliedColor / max(alpha, 0.0001);
  outColor = vec4(straightColor * finalAlpha, finalAlpha);
}
`;

export const GALAXY_FRAGMENT_SHADER = `#version 300 es
${COMMON_FRAGMENT}
${VOLUME_FRAGMENT}

float spiralThickness() {
  return mix(0.07, 0.17, vTraitsA.x);
}

float spiralRadialExtent() {
  return mix(0.96, 1.06, vTraitsA.y);
}

void galaxySample(vec3 point, out vec3 emission, out float extinction) {
  float radial = length(point.xz);
  float height = abs(point.y);
  float grain = volumeFbm3(
    point * mix(3.8, 7.2, vDetail) + vec3(vTraitsA.zw, vTraitsB.x) * 37.0
  );
  float edge = 1.0 - smoothstep(0.82, 1.06, radial);
  float armCount = floor(mix(2.0, 7.0, vTraitsA.z));
  float pitch = mix(1.35, 3.25, vTraitsA.w);
  float differentialPhase = vSpatialState.z * mix(0.72, 1.48, radial);
  float angle = atan(point.z, point.x) - differentialPhase;
  float arms = pow(
    0.5 + 0.5 * cos(armCount * (angle - log(radial + 0.055) * pitch) + grain * 2.2),
    mix(3.8, 8.4, vTraitsB.y)
  );
  float thickness = mix(0.045, 0.16, vTraitsA.x);
  float disc = exp(-height / thickness) * exp(-radial * mix(2.4, 3.8, vTraitsB.z)) * edge;
  float bulge = exp(-length(point / vec3(0.31, 0.24, 0.31)) * mix(2.2, 4.0, vTraitsB.w));
  float density = 0.0;
  float dust = 0.0;
  vec3 color = spectrum(0.3 + grain * 0.66);

  if (vSeedClass < 0.5) {
    float spiralLod = smoothstep(0.48, 0.88, vDetail);
    float lowArmCount = min(armCount, 3.0);
    float spiralPhase = angle - log(radial + 0.07) * pitch;
    float lowCarrier = cos(
      lowArmCount * spiralPhase + grain * mix(0.42, 0.82, vTraitsB.x)
    );
    float fullCarrier = cos(
      armCount * spiralPhase + grain * mix(1.1, 2.2, vTraitsB.x)
    );
    float armCarrier = 0.5 + 0.5 * mix(lowCarrier, fullCarrier, spiralLod);
    float armAntialias = max(fwidth(armCarrier) * 0.82, 0.018);
    float spiralArms = smoothstep(
      mix(0.24, 0.4, spiralLod) - armAntialias,
      mix(0.78, 0.9, spiralLod) + armAntialias,
      armCarrier
    );
    float radialExtent = spiralRadialExtent();
    float spiralEdge = 1.0 - smoothstep(radialExtent - 0.2, radialExtent, radial);
    float armAnnulus = smoothstep(0.12, 0.3, radial) * spiralEdge;
    float spiralDisc = exp(-height / spiralThickness()) *
      exp(-radial * mix(1.55, 2.55, vTraitsB.z)) * spiralEdge;
    spiralArms *= armAnnulus * mix(0.86, 1.14, grain);
    density = spiralDisc *
      (0.075 + spiralArms * mix(1.45, 2.15, vTraitsB.w) + grain * 0.11) +
      bulge * 0.42;
    dust = spiralDisc * (1.0 - spiralArms) * mix(0.42, 0.78, vTraitsA.y);
  } else if (vSeedClass < 1.5) {
    float barAngle = vSpatialState.z * 0.82;
    vec2 barPoint = rotate2(-barAngle) * point.xz;
    float bar = exp(-abs(barPoint.y) * mix(16.0, 34.0, vTraitsB.x)) *
                (1.0 - smoothstep(0.12, mix(0.48, 0.82, vTraitsB.z), abs(barPoint.x))) *
                exp(-height / (thickness * 1.35));
    density = disc * (0.1 + arms * 1.25 + grain * 0.18) + bar * 0.82 + bulge;
    dust = disc * (1.0 - arms) * 0.72;
  } else if (vSeedClass < 2.5) {
    vec3 scales = vec3(
      mix(0.68, 0.98, vTraitsA.x),
      mix(0.44, 0.72, vTraitsA.y),
      mix(0.56, 0.9, vTraitsB.x)
    );
    float radius = length(point / scales);
    float shells = 0.72 + 0.28 * sin(radius * mix(18.0, 38.0, vTraitsB.y) + grain * 3.2 + vSpatialState.w);
    density = exp(-pow(radius * mix(1.3, 1.9, vTraitsB.w), mix(1.15, 1.8, vTraitsA.z))) * shells;
    color = mix(vColorLow, vColorHigh, clamp(1.0 - radius, 0.0, 1.0));
  } else {
    float accretion = disc * (0.18 + arms * 1.12 + grain * 0.2);
    float axial = abs(point.y);
    float jetRadius = length(point.xz);
    float coneRadius = mix(0.025, 0.055, vTraitsA.y) + axial * mix(0.055, 0.13, vTraitsB.x);
    float jet = exp(-pow(jetRadius / coneRadius, 2.0)) *
                smoothstep(0.08, 0.2, axial) *
                (1.0 - smoothstep(0.78, 1.04, axial));
    float transport = 0.58 + 0.42 * volumeNoise3(vec3(
      point.xz * 18.0,
      point.y * 12.0 - sign(point.y) * vSpatialState.w * 2.4
    ));
    jet *= transport;
    float pulse = 0.74 + 0.26 * sin(vSpatialState.w * mix(1.0, 2.2, vTraitsB.z) + vPhase);
    float core = exp(-dot(point, point) * mix(95.0, 210.0, vTraitsB.w)) * pulse;
    density = accretion + bulge * 0.8 + core * 2.4 + jet * mix(0.72, 1.26, vTraitsB.z);
    color += vColorHigh * jet * 1.6 + mix(vColorMid, vec3(1.0), 0.62) * core * 2.0;
  }

  float stars = pow(max(0.0, grain - mix(0.69, 0.82, vTraitsB.w)), 3.0) *
                exp(-radial * 2.2) * exp(-height * 5.0) * 42.0;
  emission = color * (density * mix(0.7, 1.18, vTraitsB.x) + stars);
  extinction = max(0.0, density * mix(0.7, 1.2, vTraitsA.y) + dust * 1.7);
}

bool spiralDiscInterval(
  vec3 origin,
  vec3 direction,
  float outerEntry,
  float outerExit,
  out float discEntry,
  out float discExit
) {
  bool hit = ellipsoidInterval(
    origin,
    direction,
    vec3(spiralRadialExtent(), spiralThickness() * 3.0, spiralRadialExtent()),
    discEntry,
    discExit
  );
  if (!hit) return false;
  discEntry = max(outerEntry, discEntry);
  discExit = min(outerExit, discExit);
  return discExit - discEntry > 0.0001;
}

void spiralSampleSchedule(
  int index,
  int steps,
  float outerEntry,
  float outerExit,
  float discEntry,
  float discExit,
  float jitter,
  out float distance,
  out float stepLength
) {
  float frontLength = max(0.0, discEntry - outerEntry);
  float rearLength = max(0.0, outerExit - discExit);
  int reservedDiscSteps = min(
    steps,
    max(4, int(floor(float(steps) * 0.62 + 0.5)))
  );
  int haloSteps = steps - reservedDiscSteps;
  int frontSteps = 0;
  int rearSteps = 0;

  if (frontLength > 0.0001 && rearLength > 0.0001 && haloSteps >= 2) {
    float frontShare = frontLength / (frontLength + rearLength);
    frontSteps = clamp(
      int(floor(float(haloSteps) * frontShare + 0.5)),
      1,
      haloSteps - 1
    );
    rearSteps = haloSteps - frontSteps;
  } else if (frontLength > 0.0001) {
    frontSteps = haloSteps;
  } else if (rearLength > 0.0001) {
    rearSteps = haloSteps;
  }

  int discSteps = steps - frontSteps - rearSteps;
  if (index < frontSteps) {
    stepLength = frontLength / float(frontSteps);
    distance = outerEntry + (float(index) + jitter) * stepLength;
  } else if (index < frontSteps + discSteps) {
    int localIndex = index - frontSteps;
    stepLength = (discExit - discEntry) / float(discSteps);
    distance = discEntry + (float(localIndex) + jitter) * stepLength;
  } else {
    int localIndex = index - frontSteps - discSteps;
    stepLength = rearLength / float(rearSteps);
    distance = discExit + (float(localIndex) + jitter) * stepLength;
  }
}

void main() {
  vec3 axis = spatialAxis();
  mat3 basis = spatialBasis(axis);
  vec3 viewOrigin = vec3(vPoint, 1.7);
  vec3 viewDirection = vec3(0.0, 0.0, -1.0);
  vec3 origin = viewToVolumePoint(viewOrigin, basis);
  vec3 direction = normalize(viewToVolumePoint(viewDirection, basis));
  float entry;
  float exit;
  if (!ellipsoidInterval(origin, direction, vec3(1.08), entry, exit)) discard;
  entry = max(entry, 0.0);
  int steps = volumeStepCount();
  float uniformStepLength = (exit - entry) / float(steps);
  float jitter = stableVolumeJitter();
  float distance = entry + uniformStepLength * jitter;
  float discEntry = entry;
  float discExit = exit;
  bool useSpiralSchedule = vSeedClass < 0.5 && spiralDiscInterval(
    origin,
    direction,
    entry,
    exit,
    discEntry,
    discExit
  );
  vec3 accumulatedColor = vec3(0.0);
  float transmittance = 1.0;
  for (int index = 0; index < MAX_VOLUME_STEPS; index++) {
    if (index >= steps || transmittance < 0.035) break;
    float stepLength = uniformStepLength;
    if (useSpiralSchedule) {
      spiralSampleSchedule(
        index,
        steps,
        entry,
        exit,
        discEntry,
        discExit,
        jitter,
        distance,
        stepLength
      );
    }
    if (distance > exit) break;
    vec3 emission;
    float extinction;
    galaxySample(origin + direction * distance, emission, extinction);
    float sampleAlpha = 1.0 - exp(-extinction * stepLength * 2.7);
    accumulatedColor += transmittance * emission * sampleAlpha;
    transmittance *= 1.0 - sampleAlpha;
    if (!useSpiralSchedule) distance += uniformStepLength;
  }
  paintVolume(accumulatedColor, 1.0 - transmittance);
}`;

export const NEBULA_FRAGMENT_SHADER = `#version 300 es
${COMMON_FRAGMENT}
${VOLUME_FRAGMENT}

void nebulaSample(vec3 point, out vec3 emission, out float extinction) {
  vec3 flow = vec3(
    sin(point.y * 3.1 + vSpatialState.z),
    sin(point.z * 2.7 - vSpatialState.z * 0.73),
    cos(point.x * 3.4 + vSpatialState.z * 0.61)
  ) * mix(0.08, 0.22, vTraitsA.w);
  vec3 warped = point + flow;
  vec3 seedOffset = vec3(vTraitsB.zw, vTraitsA.x) * 31.0;
  float gas = volumeFbm3(warped * mix(2.1, 4.4, vDetail) + seedOffset);
  float envelope = 1.0 - smoothstep(0.72, 1.04, length(point / vec3(
    mix(0.72, 1.0, vTraitsB.x),
    mix(0.62, 0.94, vTraitsA.y),
    mix(0.68, 1.0, vTraitsB.y)
  )));
  float density = 0.0;
  float absorption = 0.0;
  vec3 color = spectrum(gas);

  if (vSeedClass < 0.5) {
    float filaments = pow(abs(gas * 2.0 - 1.0), mix(0.3, 0.66, vTraitsA.x));
    density = smoothstep(mix(0.32, 0.46, vTraitsB.x), mix(0.62, 0.79, vTraitsB.y), gas) * envelope;
    color += vColorHigh * (1.0 - filaments) * mix(0.32, 0.72, vTraitsB.z);
  } else if (vSeedClass < 1.5) {
    float lane = abs(warped.y + mix(0.08, 0.24, vTraitsA.w) *
      sin(warped.x * mix(3.0, 7.0, vTraitsA.x) + warped.z * 2.3 + vSpatialState.z));
    float darkLane = 1.0 - smoothstep(mix(0.08, 0.16, vTraitsA.y), mix(0.2, 0.38, vTraitsA.z), lane);
    density = smoothstep(0.36, mix(0.61, 0.76, vTraitsB.x), gas) * envelope * (1.0 - darkLane * 0.72);
    absorption = darkLane * envelope * mix(0.9, 2.1, vTraitsB.y);
    color = mix(vColorLow * 0.18, spectrum(0.64 + vTraitsB.y * 0.25), 1.0 - darkLane);
  } else if (vSeedClass < 2.5) {
    vec3 scales = vec3(
      mix(0.72, 0.98, vTraitsA.x),
      mix(0.5, 0.78, vTraitsA.y),
      mix(0.66, 0.92, vTraitsB.x)
    );
    vec3 shaped = warped / scales;
    float radius = length(shaped);
    float shellRadius = mix(0.38, 0.68, vTraitsA.w);
    float shell = exp(-pow((radius - shellRadius - (gas - 0.5) * 0.1) * mix(10.0, 18.0, vTraitsB.y), 2.0));
    float torus = exp(-pow((length(shaped.xz) - mix(0.3, 0.52, vTraitsB.z)) * 13.0, 2.0)) * exp(-abs(shaped.y) * 7.0);
    float lobes = exp(-length(shaped.xz) * mix(4.0, 7.5, vTraitsA.z)) * smoothstep(0.12, 0.78, abs(shaped.y));
    float pulse = 0.72 + 0.28 * sin(vSpatialState.w + vPhase);
    density = (shell + torus * 0.72 + lobes * 0.48) * envelope;
    color = mix(vColorMid, vColorHigh, clamp(shell * pulse, 0.0, 1.0));
  } else {
    vec3 scales = vec3(
      mix(0.82, 1.0, vTraitsA.x),
      mix(0.38, 0.52, vTraitsA.y),
      mix(0.58, 0.82, vTraitsB.x)
    );
    vec3 shaped = warped / scales;
    float radius = length(shaped);
    vec3 direction = shaped / max(radius, 0.0001);
    float azimuth = atan(direction.z, direction.x);
    float polarLobe = direction.y * direction.y - 0.3333333;
    float shape = 1.0 +
      mix(0.12, 0.22, vTraitsA.z) * sin(2.0 * azimuth + vTraitsB.y * 6.0) +
      mix(0.15, 0.28, vTraitsA.w) * polarLobe +
      (gas - 0.5) * mix(0.12, 0.24, vTraitsB.z);
    float cycle = fract(vSpatialState.w / (2.0 * PI));
    float front = mix(0.08, mix(0.76, 0.98, vTraitsB.y), smoothstep(0.03, 0.82, cycle));
    float shellWidth = mix(mix(42.0, 60.0, vTraitsB.z), mix(12.0, 21.0, vTraitsB.w), cycle);
    float holes = smoothstep(mix(0.3, 0.48, vTraitsA.x), mix(0.58, 0.74, vTraitsA.y), gas);
    float shell = exp(-pow((radius - front * shape) * shellWidth, 2.0)) * (0.34 + holes * 0.86);
    float flash = exp(-radius * radius * mix(28.0, 54.0, vTraitsA.z)) *
                  pow(1.0 - cycle, mix(8.0, 14.0, vTraitsA.w)) *
                  mix(2.2, 3.8, vTraitsB.x);
    float ejecta = holes * (1.0 - smoothstep(front * 0.26, front, radius));
    float fade = 1.0 - smoothstep(0.78, 1.0, cycle);
    density = (shell * 1.42 + ejecta * 0.48 + flash) * fade;
    color = mix(vColorHigh, vColorMid, cycle) + vec3(flash);
  }

  emission = color * density * mix(0.72, 1.18, vTraitsB.w);
  extinction = max(0.0, density * mix(0.72, 1.28, vTraitsA.x) + absorption);
}

void main() {
  vec3 axis = spatialAxis();
  mat3 basis = spatialBasis(axis);
  vec3 viewOrigin = vec3(vPoint, 1.7);
  vec3 viewDirection = vec3(0.0, 0.0, -1.0);
  vec3 origin = viewToVolumePoint(viewOrigin, basis);
  vec3 direction = normalize(viewToVolumePoint(viewDirection, basis));
  float entry;
  float exit;
  if (!ellipsoidInterval(origin, direction, vec3(1.08), entry, exit)) discard;
  entry = max(entry, 0.0);
  int steps = volumeStepCount();
  float stepLength = (exit - entry) / float(steps);
  float distance = entry + stepLength * stableVolumeJitter();
  vec3 accumulatedColor = vec3(0.0);
  float transmittance = 1.0;
  for (int index = 0; index < MAX_VOLUME_STEPS; index++) {
    if (index >= steps || distance > exit || transmittance < 0.035) break;
    vec3 emission;
    float extinction;
    nebulaSample(origin + direction * distance, emission, extinction);
    float sampleAlpha = 1.0 - exp(-extinction * stepLength * 2.35);
    accumulatedColor += transmittance * emission * sampleAlpha;
    transmittance *= 1.0 - sampleAlpha;
    distance += stepLength;
  }
  paintVolume(accumulatedColor, 1.0 - transmittance);
}`;

export const UFO_VERTEX_SHADER = `#version 300 es
precision highp float;
layout(location=0) in vec2 aCorner;
layout(location=1) in vec4 aPlacement;
layout(location=2) in vec4 aOrientation;
layout(location=3) in vec4 aShapeA;
layout(location=4) in vec4 aShapeB;
layout(location=5) in vec4 aBodyColorLightCount;
layout(location=6) in vec4 aTrimColorLightOffset;
layout(location=7) in vec4 aLightColorAPulseFrequency;
layout(location=8) in vec4 aLightColorBPulsePhase;
uniform vec2 uResolution;
out vec2 vPoint;
out vec4 vOrientation;
out vec4 vShapeA;
out vec4 vShapeB;
out vec4 vBodyColorLightCount;
out vec4 vTrimColorLightOffset;
out vec4 vLightColorAPulseFrequency;
out vec4 vLightColorBPulsePhase;
out float vDomeTint;

void main() {
  float extent = 1.24;
  vec2 localPoint = aCorner * extent;
  vec2 physicalRadius = vec2(
    aPlacement.w * 2.0 / uResolution.x,
    aPlacement.w * 2.0 / uResolution.y
  );
  gl_Position = vec4(
    aPlacement.xy + localPoint * physicalRadius,
    0.0,
    aPlacement.z
  );
  vPoint = localPoint;
  vOrientation = aOrientation;
  vShapeA = aShapeA;
  vShapeB = aShapeB;
  vBodyColorLightCount = aBodyColorLightCount;
  vTrimColorLightOffset = aTrimColorLightOffset;
  vLightColorAPulseFrequency = aLightColorAPulseFrequency;
  vLightColorBPulsePhase = aLightColorBPulsePhase;
  vDomeTint = aLightColorBPulsePhase.w;
}`;

export const UFO_FRAGMENT_SHADER = `#version 300 es
precision highp float;
in vec2 vPoint;
in vec4 vOrientation;
in vec4 vShapeA;
in vec4 vShapeB;
in vec4 vBodyColorLightCount;
in vec4 vTrimColorLightOffset;
in vec4 vLightColorAPulseFrequency;
in vec4 vLightColorBPulsePhase;
in float vDomeTint;
out vec4 outColor;

#define PI 3.14159265359
#define UFO_MARCH_STEPS 56

mat3 rotateX(float angle) {
  float c = cos(angle);
  float s = sin(angle);
  return mat3(1.0, 0.0, 0.0, 0.0, c, s, 0.0, -s, c);
}

mat3 rotateY(float angle) {
  float c = cos(angle);
  float s = sin(angle);
  return mat3(c, 0.0, -s, 0.0, 1.0, 0.0, s, 0.0, c);
}

mat3 rotateZ(float angle) {
  float c = cos(angle);
  float s = sin(angle);
  return mat3(c, s, 0.0, -s, c, 0.0, 0.0, 0.0, 1.0);
}

float sdEllipsoid(vec3 point, vec3 radii) {
  float k0 = length(point / radii);
  float k1 = length(point / (radii * radii));
  return k0 * (k0 - 1.0) / max(k1, 0.0001);
}

float sdTorus(vec3 point, vec2 radii) {
  vec2 q = vec2(length(point.xz) - radii.x, point.y);
  return length(q) - radii.y;
}

float smoothUnion(float first, float second, float amount) {
  float h = clamp(0.5 + 0.5 * (second - first) / amount, 0.0, 1.0);
  return mix(second, first, h) - amount * h * (1.0 - h);
}

vec2 saucerField(vec3 point) {
  float hullThickness = vShapeA.x;
  float hull = sdEllipsoid(point, vec3(1.0, hullThickness, 1.0));
  float rim = sdTorus(point, vec2(0.82, vShapeA.y));
  float body = smoothUnion(hull, rim, 0.045);
  float dome = sdEllipsoid(
    point - vec3(0.0, hullThickness * 0.58, 0.0),
    vec3(vShapeA.z, vShapeA.w, vShapeA.z)
  );
  float underside = sdEllipsoid(
    point + vec3(0.0, vShapeB.x * 0.7, 0.0),
    vec3(0.6, vShapeB.x, 0.6)
  );
  vec2 result = vec2(body, 1.0);
  if (rim < result.x + 0.012) result = vec2(rim, 2.0);
  if (dome < result.x) result = vec2(dome, 3.0);
  if (underside < result.x) result = vec2(underside, 4.0);
  result.x = min(smoothUnion(body, dome, 0.035), underside);
  return result;
}

vec3 saucerNormal(vec3 point) {
  vec2 e = vec2(0.0025, 0.0);
  return normalize(vec3(
    saucerField(point + e.xyy).x - saucerField(point - e.xyy).x,
    saucerField(point + e.yxy).x - saucerField(point - e.yxy).x,
    saucerField(point + e.yyx).x - saucerField(point - e.yyx).x
  ));
}

float wrappedAngle(float angle) {
  return atan(sin(angle), cos(angle));
}

void main() {
  mat3 orientation =
    rotateZ(vOrientation.z) * rotateX(vOrientation.y) * rotateY(vOrientation.x);
  mat3 viewToObject = transpose(orientation);
  vec3 origin = viewToObject * vec3(vPoint, 2.45);
  vec3 direction = normalize(viewToObject * vec3(0.0, 0.0, -1.0));
  float distanceAlongRay = 0.0;
  float material = 0.0;
  bool hit = false;
  for (int index = 0; index < UFO_MARCH_STEPS; index++) {
    vec3 point = origin + direction * distanceAlongRay;
    vec2 field = saucerField(point);
    if (field.x < 0.0025) {
      material = field.y;
      hit = true;
      break;
    }
    distanceAlongRay += max(field.x * 0.72, 0.003);
    if (distanceAlongRay > 4.9) break;
  }
  if (!hit) discard;

  vec3 point = origin + direction * distanceAlongRay;
  vec3 normal = saucerNormal(point);
  vec3 lightDirection = normalize(
    viewToObject * normalize(vec3(-0.46, 0.72, 0.82))
  );
  vec3 viewDirection = normalize(-direction);
  float diffuse = max(0.0, dot(normal, lightDirection));
  float fresnel = pow(1.0 - max(0.0, dot(normal, viewDirection)), 3.0);
  float highlight = pow(
    max(0.0, dot(reflect(-lightDirection, normal), viewDirection)),
    mix(18.0, 76.0, 1.0 - vShapeB.z)
  );
  vec3 bodyColor = vBodyColorLightCount.xyz;
  vec3 trimColor = vTrimColorLightOffset.xyz;
  vec3 color = bodyColor * (0.22 + diffuse * 0.7);
  color += vec3(highlight) * mix(0.25, 0.72, vShapeB.y);

  float radial = length(point.xz);
  float angle = atan(point.z, point.x);
  float panels = 0.5 + 0.5 * cos(angle * vShapeB.w + radial * 16.0);
  color = mix(color, trimColor * (0.3 + diffuse * 0.55), panels * 0.12);
  if (material > 1.5 && material < 2.5) {
    color = mix(color, trimColor, 0.62) + trimColor * fresnel * 0.32;
  } else if (material > 2.5 && material < 3.5) {
    vec3 domeColor = mix(bodyColor, trimColor, vDomeTint);
    color = domeColor * (0.2 + diffuse * 0.48) +
      mix(trimColor, vec3(0.86, 0.96, 1.0), 0.62) * fresnel * 0.9;
  } else if (material > 3.5) {
    color *= 0.68;
  }

  float packedLights = vBodyColorLightCount.w;
  float lightCount = floor(packedLights);
  float spacingJitter = fract(packedLights);
  float spacing = 2.0 * PI / lightCount;
  float rawIndex = (angle - vTrimColorLightOffset.w) / spacing;
  float lightIndex = floor(rawIndex + 0.5);
  float jitter = sin(
    lightIndex * 12.9898 + vTrimColorLightOffset.w * 7.137
  ) * spacing * spacingJitter;
  float targetAngle =
    vTrimColorLightOffset.w + lightIndex * spacing + jitter;
  float arcDistance = abs(wrappedAngle(angle - targetAngle)) *
    max(radial, 0.4);
  float rimBand = smoothstep(0.66, 0.79, radial) *
    (1.0 - smoothstep(0.98, 1.04, radial));
  float verticalBand = exp(-point.y * point.y * 310.0);
  float lamp = exp(-arcDistance * arcDistance * 1800.0) *
    rimBand * verticalBand;
  float pulse = 0.68 + 0.32 * sin(
    vOrientation.w * vLightColorAPulseFrequency.w + lightIndex * 1.73
  );
  vec3 lampColor = mix(
    vLightColorAPulseFrequency.xyz,
    vLightColorBPulsePhase.xyz,
    mod(lightIndex, 2.0)
  );
  color += lampColor * lamp * pulse * 2.25;

  float alpha = 1.0 - smoothstep(0.0, 0.0045, saucerField(point).x);
  alpha = max(alpha, smoothstep(0.002, 0.0, abs(saucerField(point).x)));
  alpha = clamp(alpha, 0.0, 1.0);
  outColor = vec4(color * alpha, alpha);
}`;

export const BACKDROP_FRAGMENT_SHADER = `#version 300 es
precision highp float;
in vec2 vPoint;
uniform vec2 uResolution;
uniform float uTime;
uniform vec4 uPageVariation;
out vec4 outColor;

float hash(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

void main() {
  vec2 pixel = vec2(
    (vPoint.x * 0.5 + 0.5) * uResolution.x,
    (0.5 - vPoint.y * 0.5) * uResolution.y
  );
  vec2 pageOffset =
    (uPageVariation.xy * 173.0 + uPageVariation.zw * 71.0) * ${String(SPACE_RAIN_STAR_CELL_PX)};
  vec2 drift = vec2(
    uTime * ${String(SPACE_RAIN_STAR_DRIFT_PX[0])},
    uTime * ${String(SPACE_RAIN_STAR_DRIFT_PX[1])}
  );
  vec2 starGrid = (pixel + drift + pageOffset) / ${String(SPACE_RAIN_STAR_CELL_PX)};
  vec2 cell = floor(starGrid);
  vec2 local = (fract(starGrid) - 0.5) * ${String(SPACE_RAIN_STAR_CELL_PX)};
  float random = hash(cell + pageOffset);
  float star =
    (1.0 - smoothstep(0.0, ${String(SPACE_RAIN_STAR_CORE_PX)}, length(local))) *
    step(0.982, random);
  float shimmer = 0.68 + 0.32 * sin(uTime * (0.7 + random * 2.2) + random * 31.0);
  vec3 starColor = mix(vec3(0.32, 0.48, 0.82), vec3(0.86, 0.96, 1.0), random);
  float dust = hash(floor(
    (pixel + drift * 0.42 + pageOffset) / ${String(SPACE_RAIN_DUST_CELL_PX)}
  ));
  dust = step(0.993, dust) * 0.22;
  float alpha = star * shimmer * 0.56 + dust;
  outColor = vec4(starColor * alpha, alpha);
}`;

export const BLACK_HOLE_FRAGMENT_SHADER = `#version 300 es
precision highp float;
in vec2 vPoint;
uniform sampler2D uSceneTexture;
uniform vec2 uResolution;
uniform vec2 uBlackHolePosition;
uniform vec4 uBlackHoleLens;
uniform vec4 uBlackHoleSpin;
out vec4 outColor;

#define PI 3.14159265359

vec2 rotatePoint(vec2 point, float angle) {
  float sine = sin(angle);
  float cosine = cos(angle);
  return mat2(cosine, -sine, sine, cosine) * point;
}

void main() {
  vec2 uv = vPoint * 0.5 + 0.5;
  vec2 center = uBlackHolePosition * 0.5 + 0.5;
  vec2 deltaPixels = (uv - center) * uResolution;
  float distancePixels = length(deltaPixels);
  float shadowPixels = max(uBlackHoleLens.x, 1.0);
  float radius = distancePixels / shadowPixels;
  float influenceRadius = max(uBlackHoleLens.y, 1.1);

  if (radius >= influenceRadius) {
    outColor = texture(uSceneTexture, uv);
    return;
  }

  float antialiasWidth = 1.25 / shadowPixels;
  if (radius <= 1.0 - antialiasWidth) {
    outColor = vec4(0.0, 0.0, 0.0, 1.0);
    return;
  }

  float safeRadius = max(radius, 0.35);
  float criticalRadius = uBlackHoleLens.w;
  float edgeWeight = 1.0 - smoothstep(
    max(criticalRadius + uBlackHoleSpin.x * 2.0, influenceRadius * 0.72),
    influenceRadius,
    radius
  );
  float inverseImpact =
    criticalRadius * criticalRadius * uBlackHoleLens.z / safeRadius;
  float sourceRadius = mix(radius, radius - inverseImpact, edgeWeight);
  vec2 direction = distancePixels > 0.0001
    ? deltaPixels / distancePixels
    : vec2(1.0, 0.0);
  float twist = uBlackHoleSpin.y * edgeWeight *
    (0.9 + 0.1 * sin(uBlackHoleSpin.w)) /
    max(radius * radius, 0.6);
  direction = rotatePoint(direction, twist);
  vec2 sourceUv = center +
    direction * sourceRadius * shadowPixels / uResolution;
  sourceUv = clamp(sourceUv, vec2(0.0), vec2(1.0));

  vec4 source = texture(uSceneTexture, sourceUv);
  float criticalBand = exp(-pow(
    (radius - criticalRadius) / max(uBlackHoleSpin.x, 0.02),
    2.0
  ));
  float sourceRadiance = max(source.r, max(source.g, source.b));
  float gain = 1.0 + criticalBand * uBlackHoleSpin.z *
    smoothstep(0.015, 0.24, sourceRadiance);
  vec4 lensed = vec4(source.rgb * gain, source.a);
  float silhouette = smoothstep(
    1.0 - antialiasWidth,
    1.0 + antialiasWidth,
    radius
  );
  outColor = mix(vec4(0.0, 0.0, 0.0, 1.0), lensed, silhouette);
}`;

export const FAMILY_FRAGMENT_SHADERS = {
  planet: PLANET_FRAGMENT_SHADER,
  galaxy: GALAXY_FRAGMENT_SHADER,
  nebula: NEBULA_FRAGMENT_SHADER,
} as const;
