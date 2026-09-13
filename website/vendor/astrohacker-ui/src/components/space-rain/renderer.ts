import {
  RAIN_FAMILIES,
  SPACE_RAIN_REFERENCE_HEIGHT,
  SPACE_RAIN_REFERENCE_WIDTH,
  type RainFamily,
} from "./config";
import { currentBlackHoleState } from "./black-hole";
import type { RainInstance, SpaceRainScene } from "./model";
import { currentUfoState, type UfoEncounter, type UfoState } from "./ufo";
import {
  BACKDROP_FRAGMENT_SHADER,
  BACKDROP_VERTEX_SHADER,
  BLACK_HOLE_FRAGMENT_SHADER,
  FAMILY_FRAGMENT_SHADERS,
  OBJECT_VERTEX_SHADER,
  PLANET_VERTEX_SHADER,
  UFO_FRAGMENT_SHADER,
  UFO_VERTEX_SHADER,
} from "./shaders";

export const FLOATS_PER_INSTANCE = 32;
const OBJECT_STRIDE = FLOATS_PER_INSTANCE * Float32Array.BYTES_PER_ELEMENT;
const FAMILY_SCALE: Record<RainFamily, number> = {
  planet: 0.82,
  galaxy: 1.36,
  nebula: 1.44,
};
const BARRED_SPIRAL_SCALE = 3;
const DRAW_ORDER = ["nebula", "galaxy", "planet"] as const;

export interface ResourceCounts {
  contexts: number;
  programs: number;
  buffers: number;
  vertexArrays: number;
  textures: number;
  framebuffers: number;
}

export interface FrameReport {
  drawCalls: number;
  instanceCount: number;
  ufoActive: boolean;
  blackHoleActive: boolean;
  offscreenActive: boolean;
  offscreenBytes: number;
}

type RendererProgram = RainFamily | "backdrop" | "ufo" | "black-hole";

interface ProgramRecord {
  program: WebGLProgram;
  resolution: WebGLUniformLocation | null;
  time: WebGLUniformLocation | null;
  pageVariation: WebGLUniformLocation | null;
  sceneTexture: WebGLUniformLocation | null;
  blackHolePosition: WebGLUniformLocation | null;
  blackHoleLens: WebGLUniformLocation | null;
  blackHoleSpin: WebGLUniformLocation | null;
}

// Older browser WebGL implementations can return null on allocation/context
// loss, despite current DOM declarations promising resource objects.
function hasResource(resource: object | null): boolean {
  return resource !== null;
}

function compileShader(
  gl: WebGL2RenderingContext,
  kind: number,
  source: string,
): WebGLShader {
  const shader = gl.createShader(kind);
  if (!shader) throw new Error("Unable to allocate a SpaceRain shader");
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(shader) ?? "Unknown compile failure";
    gl.deleteShader(shader);
    throw new Error(`SpaceRain shader compile failed: ${log}`);
  }
  return shader;
}

function createProgram(
  gl: WebGL2RenderingContext,
  vertexSource: string,
  fragmentSource: string,
): ProgramRecord {
  const vertex = compileShader(gl, gl.VERTEX_SHADER, vertexSource);
  let fragment: WebGLShader | null = null;
  let program: WebGLProgram | null = null;
  try {
    fragment = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
    program = gl.createProgram();
    if (!hasResource(program))
      throw new Error("Unable to allocate a SpaceRain program");
    gl.attachShader(program, vertex);
    gl.attachShader(program, fragment);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      const log = gl.getProgramInfoLog(program) ?? "Unknown link failure";
      throw new Error(`SpaceRain program link failed: ${log}`);
    }
    return {
      program,
      resolution: gl.getUniformLocation(program, "uResolution"),
      time: gl.getUniformLocation(program, "uTime"),
      pageVariation: gl.getUniformLocation(program, "uPageVariation"),
      sceneTexture: gl.getUniformLocation(program, "uSceneTexture"),
      blackHolePosition: gl.getUniformLocation(program, "uBlackHolePosition"),
      blackHoleLens: gl.getUniformLocation(program, "uBlackHoleLens"),
      blackHoleSpin: gl.getUniformLocation(program, "uBlackHoleSpin"),
    };
  } catch (error) {
    if (program) gl.deleteProgram(program);
    throw error;
  } finally {
    gl.deleteShader(vertex);
    if (fragment) gl.deleteShader(fragment);
  }
}

export function orderedFamilyInstances(
  instances: readonly RainInstance[],
  family: RainFamily,
): RainInstance[] {
  return instances
    .filter((instance) => instance.family === family)
    .sort((first, second) => first.depth - second.depth);
}

export function pixelPositionToClip(
  x: number,
  y: number,
  width: number,
  height: number,
): readonly [number, number] {
  return [(x / Math.max(1, width)) * 2 - 1, 1 - (y / Math.max(1, height)) * 2];
}

export function instanceData(
  instances: readonly RainInstance[],
  family: RainFamily,
  width = SPACE_RAIN_REFERENCE_WIDTH,
  height = SPACE_RAIN_REFERENCE_HEIGHT,
): Float32Array {
  const data = new Float32Array(instances.length * FLOATS_PER_INSTANCE);
  const scale = FAMILY_SCALE[family];
  for (let index = 0; index < instances.length; index += 1) {
    const instance = instances[index];
    if (!instance) continue;
    const offset = index * FLOATS_PER_INSTANCE;
    const planetRotation = instance.planetRotation;
    const volumetricMotion = instance.volumetricMotion;
    const classScale =
      family === "galaxy" && instance.seedClass === 1 ? BARRED_SPIRAL_SCALE : 1;
    const position = pixelPositionToClip(instance.x, instance.y, width, height);
    data.set(
      [
        ...position,
        instance.depth,
        instance.size * scale * classScale,
        planetRotation?.surfaceAngle ??
          volumetricMotion?.bulkAngle ??
          instance.rotation,
        instance.seedClass,
        instance.phase,
        instance.intensity,
        ...instance.traitsA,
        ...instance.traitsB,
        ...instance.colorLow,
        instance.detail,
        ...instance.colorMid,
        instance.age,
        ...instance.colorHigh,
        0,
        planetRotation?.axisInclination ??
          volumetricMotion?.axisInclination ??
          0,
        planetRotation?.axisPosition ?? volumetricMotion?.axisPosition ?? 0,
        planetRotation?.cloudAngle ?? volumetricMotion?.internalAngle ?? 0,
        planetRotation?.ringAngle ?? volumetricMotion?.evolutionAngle ?? 0,
      ],
      offset,
    );
  }
  return data;
}

export function ufoInstanceData(
  encounter: UfoEncounter,
  state: UfoState,
): Float32Array {
  const { appearance } = encounter;
  const wrapAngle = (angle: number): number => {
    const fullTurn = Math.PI * 2;
    return ((((angle + Math.PI) % fullTurn) + fullTurn) % fullTurn) - Math.PI;
  };
  const lightTime =
    state.localTime +
    appearance.lightPulsePhase / appearance.lightPulseFrequency;
  return new Float32Array([
    state.cameraPosition[0],
    state.cameraPosition[1],
    state.cameraPosition[2],
    encounter.referenceRadius,
    wrapAngle(state.yaw),
    state.pitch,
    wrapAngle(state.roll),
    lightTime,
    appearance.hullThickness,
    appearance.rimBevel,
    appearance.domeRadius,
    appearance.domeHeight,
    appearance.undersideDepth,
    appearance.metallicity,
    appearance.roughness,
    appearance.panelFrequency,
    ...appearance.bodyColor,
    appearance.lightCount + appearance.lightSpacingJitter,
    ...appearance.trimColor,
    appearance.lightOffset,
    ...appearance.lightColorA,
    appearance.lightPulseFrequency,
    ...appearance.lightColorB,
    appearance.domeTint,
  ]);
}

export class SpaceRainRenderer {
  readonly gl: WebGL2RenderingContext;
  readonly resources: ResourceCounts = {
    contexts: 1,
    programs: 0,
    buffers: 0,
    vertexArrays: 0,
    textures: 0,
    framebuffers: 0,
  };
  readonly shaderStatus: string[] = [];
  private readonly programs = new Map<RendererProgram, ProgramRecord>();
  private readonly vertexArray: WebGLVertexArrayObject;
  private readonly quadBuffer: WebGLBuffer;
  private readonly instanceBuffer: WebGLBuffer;
  private disposed = false;
  private cssWidth = 1;
  private cssHeight = 1;
  private sceneTexture: WebGLTexture | null = null;
  private sceneFramebuffer: WebGLFramebuffer | null = null;
  private targetWidth = 0;
  private targetHeight = 0;

  constructor(private readonly canvas: HTMLCanvasElement) {
    const gl = canvas.getContext("webgl2", {
      alpha: true,
      antialias: true,
      depth: false,
      desynchronized: true,
      powerPreference: "high-performance",
      premultipliedAlpha: true,
      preserveDrawingBuffer: false,
      stencil: false,
    });
    if (!gl) throw new Error("WebGL2 is unavailable");
    this.gl = gl;

    const vertexArray = gl.createVertexArray();
    const quadBuffer = gl.createBuffer();
    const instanceBuffer = gl.createBuffer();
    if (
      !hasResource(vertexArray) ||
      !hasResource(quadBuffer) ||
      !hasResource(instanceBuffer)
    ) {
      if (hasResource(instanceBuffer)) gl.deleteBuffer(instanceBuffer);
      if (hasResource(quadBuffer)) gl.deleteBuffer(quadBuffer);
      if (hasResource(vertexArray)) gl.deleteVertexArray(vertexArray);
      this.resources.contexts = 0;
      throw new Error("Unable to allocate SpaceRain geometry");
    }
    this.vertexArray = vertexArray;
    this.quadBuffer = quadBuffer;
    this.instanceBuffer = instanceBuffer;
    this.resources.buffers = 2;
    this.resources.vertexArrays = 1;

    gl.bindVertexArray(vertexArray);
    gl.bindBuffer(gl.ARRAY_BUFFER, quadBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      gl.STATIC_DRAW,
    );
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    gl.bindBuffer(gl.ARRAY_BUFFER, instanceBuffer);
    for (let slot = 0; slot < 8; slot += 1) {
      const location = slot + 1;
      gl.enableVertexAttribArray(location);
      gl.vertexAttribPointer(
        location,
        4,
        gl.FLOAT,
        false,
        OBJECT_STRIDE,
        slot * 4 * Float32Array.BYTES_PER_ELEMENT,
      );
      gl.vertexAttribDivisor(location, 1);
    }

    try {
      this.addProgram(
        "backdrop",
        BACKDROP_VERTEX_SHADER,
        BACKDROP_FRAGMENT_SHADER,
      );
      for (const family of RAIN_FAMILIES) {
        this.addProgram(
          family,
          family === "planet" ? PLANET_VERTEX_SHADER : OBJECT_VERTEX_SHADER,
          FAMILY_FRAGMENT_SHADERS[family],
        );
      }
      this.addProgram("ufo", UFO_VERTEX_SHADER, UFO_FRAGMENT_SHADER);
      this.addProgram(
        "black-hole",
        BACKDROP_VERTEX_SHADER,
        BLACK_HOLE_FRAGMENT_SHADER,
      );
    } catch (error) {
      for (const record of this.programs.values()) {
        gl.deleteProgram(record.program);
      }
      this.programs.clear();
      gl.deleteBuffer(instanceBuffer);
      gl.deleteBuffer(quadBuffer);
      gl.deleteVertexArray(vertexArray);
      this.resources.contexts = 0;
      this.resources.programs = 0;
      this.resources.buffers = 0;
      this.resources.vertexArrays = 0;
      this.resources.textures = 0;
      this.resources.framebuffers = 0;
      throw error;
    }

    gl.enable(gl.BLEND);
    gl.blendEquation(gl.FUNC_ADD);
    gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
    gl.disable(gl.DEPTH_TEST);
    gl.clearColor(0, 0, 0, 0);
    gl.bindVertexArray(null);
  }

  private addProgram(
    family: RendererProgram,
    vertex: string,
    fragment: string,
  ): void {
    const record = createProgram(this.gl, vertex, fragment);
    this.programs.set(family, record);
    this.resources.programs += 1;
    this.shaderStatus.push(`${family}: linked`);
  }

  private setUniforms(
    record: ProgramRecord,
    scene: SpaceRainScene,
    timeSeconds: number,
  ): void {
    const gl = this.gl;
    gl.uniform2f(record.resolution, this.cssWidth, this.cssHeight);
    gl.uniform1f(record.time, timeSeconds);
    gl.uniform4f(
      record.pageVariation,
      scene.pageSeed[0] / 4_294_967_296,
      scene.pageSeed[1] / 4_294_967_296,
      scene.pageSeed[2] / 4_294_967_296,
      scene.pageSeed[3] / 4_294_967_296,
    );
  }

  resize(cssWidth: number, cssHeight: number, dpr: number): void {
    this.cssWidth = Math.max(1, cssWidth);
    this.cssHeight = Math.max(1, cssHeight);
    const width = Math.max(1, Math.round(this.cssWidth * dpr));
    const height = Math.max(1, Math.round(this.cssHeight * dpr));
    if (this.canvas.width !== width) this.canvas.width = width;
    if (this.canvas.height !== height) this.canvas.height = height;
    this.gl.viewport(0, 0, width, height);
  }

  private ensureSceneTarget(): void {
    const gl = this.gl;
    if (!this.sceneTexture) {
      this.sceneTexture = gl.createTexture();
      if (!hasResource(this.sceneTexture)) {
        throw new Error("Unable to allocate SpaceRain scene texture");
      }
      this.resources.textures = 1;
    }
    if (!this.sceneFramebuffer) {
      this.sceneFramebuffer = gl.createFramebuffer();
      if (!hasResource(this.sceneFramebuffer)) {
        gl.deleteTexture(this.sceneTexture);
        this.sceneTexture = null;
        this.resources.textures = 0;
        throw new Error("Unable to allocate SpaceRain scene framebuffer");
      }
      this.resources.framebuffers = 1;
    }
    if (
      this.targetWidth === this.canvas.width &&
      this.targetHeight === this.canvas.height
    ) {
      return;
    }
    gl.bindTexture(gl.TEXTURE_2D, this.sceneTexture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA8,
      this.canvas.width,
      this.canvas.height,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      null,
    );
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.sceneFramebuffer);
    gl.framebufferTexture2D(
      gl.FRAMEBUFFER,
      gl.COLOR_ATTACHMENT0,
      gl.TEXTURE_2D,
      this.sceneTexture,
      0,
    );
    if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.bindTexture(gl.TEXTURE_2D, null);
      gl.deleteFramebuffer(this.sceneFramebuffer);
      gl.deleteTexture(this.sceneTexture);
      this.sceneFramebuffer = null;
      this.sceneTexture = null;
      this.resources.framebuffers = 0;
      this.resources.textures = 0;
      throw new Error("SpaceRain scene framebuffer is incomplete");
    }
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.bindTexture(gl.TEXTURE_2D, null);
    this.targetWidth = this.canvas.width;
    this.targetHeight = this.canvas.height;
  }

  private drawScene(
    scene: SpaceRainScene,
    timeSeconds: number,
  ): Omit<
    FrameReport,
    "blackHoleActive" | "offscreenActive" | "offscreenBytes"
  > {
    const gl = this.gl;
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.bindVertexArray(this.vertexArray);

    let drawCalls = 0;
    const backdrop = this.programs.get("backdrop");
    if (backdrop) {
      gl.useProgram(backdrop.program);
      this.setUniforms(backdrop, scene, timeSeconds);
      gl.drawArrays(gl.TRIANGLES, 0, 6);
      drawCalls += 1;
    }

    for (const family of DRAW_ORDER) {
      const familyInstances = orderedFamilyInstances(scene.instances, family);
      if (familyInstances.length === 0) continue;
      const record = this.programs.get(family);
      if (!record) continue;
      gl.useProgram(record.program);
      this.setUniforms(record, scene, timeSeconds);
      gl.bindBuffer(gl.ARRAY_BUFFER, this.instanceBuffer);
      gl.bufferData(
        gl.ARRAY_BUFFER,
        instanceData(familyInstances, family, scene.width, scene.height),
        gl.DYNAMIC_DRAW,
      );
      gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, familyInstances.length);
      drawCalls += 1;
    }

    const encounter = scene.ufoScheduler.active;
    const ufoState = currentUfoState(scene.ufoScheduler);
    if (encounter && ufoState && ufoState.kind !== "complete") {
      const record = this.programs.get("ufo");
      if (record) {
        gl.useProgram(record.program);
        this.setUniforms(record, scene, timeSeconds);
        gl.bindBuffer(gl.ARRAY_BUFFER, this.instanceBuffer);
        gl.bufferData(
          gl.ARRAY_BUFFER,
          ufoInstanceData(encounter, ufoState),
          gl.DYNAMIC_DRAW,
        );
        gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, 1);
        drawCalls += 1;
      }
    }

    return {
      drawCalls,
      instanceCount: scene.instances.length,
      ufoActive: Boolean(encounter && ufoState?.kind !== "complete"),
    };
  }

  render(scene: SpaceRainScene, timeSeconds: number): FrameReport {
    if (this.disposed) {
      return {
        drawCalls: 0,
        instanceCount: 0,
        ufoActive: false,
        blackHoleActive: false,
        offscreenActive: false,
        offscreenBytes: 0,
      };
    }
    const gl = this.gl;
    const blackHole = scene.blackHoleScheduler.active;
    const blackHoleState = currentBlackHoleState(scene.blackHoleScheduler);
    const blackHoleActive = Boolean(
      blackHole && blackHoleState?.kind === "active",
    );
    if (!blackHoleActive || !blackHole || !blackHoleState) {
      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.viewport(0, 0, this.canvas.width, this.canvas.height);
      const report = this.drawScene(scene, timeSeconds);
      gl.bindVertexArray(null);
      return {
        ...report,
        blackHoleActive: false,
        offscreenActive: false,
        offscreenBytes:
          this.resources.textures > 0
            ? this.targetWidth * this.targetHeight * 4
            : 0,
      };
    }

    this.ensureSceneTarget();
    gl.bindFramebuffer(gl.FRAMEBUFFER, this.sceneFramebuffer);
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    const report = this.drawScene(scene, timeSeconds);

    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, this.canvas.width, this.canvas.height);
    gl.clear(gl.COLOR_BUFFER_BIT);
    const lens = this.programs.get("black-hole");
    if (!lens || !this.sceneTexture) {
      throw new Error("SpaceRain black-hole lens resources are unavailable");
    }
    gl.useProgram(lens.program);
    this.setUniforms(lens, scene, timeSeconds);
    const appearance = blackHole.appearance;
    gl.uniform2f(
      lens.blackHolePosition,
      blackHoleState.position[0],
      blackHoleState.position[1],
    );
    gl.uniform4f(
      lens.blackHoleLens,
      blackHole.size * appearance.shadowRadius,
      appearance.influenceRadius,
      appearance.deflectionStrength * appearance.mass,
      appearance.criticalRadius,
    );
    gl.uniform4f(
      lens.blackHoleSpin,
      appearance.criticalWidth,
      appearance.frameDragTwist * appearance.handedness,
      appearance.magnification,
      appearance.phase,
    );
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.sceneTexture);
    gl.uniform1i(lens.sceneTexture, 0);
    gl.disable(gl.BLEND);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
    gl.enable(gl.BLEND);
    gl.bindTexture(gl.TEXTURE_2D, null);

    gl.bindVertexArray(null);
    return {
      ...report,
      drawCalls: report.drawCalls + 1,
      blackHoleActive: true,
      offscreenActive: true,
      offscreenBytes: this.targetWidth * this.targetHeight * 4,
    };
  }

  dispose(loseContext = true): void {
    if (this.disposed) return;
    this.disposed = true;
    const gl = this.gl;
    for (const record of this.programs.values()) {
      gl.deleteProgram(record.program);
    }
    this.programs.clear();
    if (this.sceneFramebuffer) gl.deleteFramebuffer(this.sceneFramebuffer);
    if (this.sceneTexture) gl.deleteTexture(this.sceneTexture);
    this.sceneFramebuffer = null;
    this.sceneTexture = null;
    gl.deleteBuffer(this.instanceBuffer);
    gl.deleteBuffer(this.quadBuffer);
    gl.deleteVertexArray(this.vertexArray);
    if (loseContext) gl.getExtension("WEBGL_lose_context")?.loseContext();
    this.resources.contexts = 0;
    this.resources.programs = 0;
    this.resources.buffers = 0;
    this.resources.vertexArrays = 0;
    this.resources.textures = 0;
    this.resources.framebuffers = 0;
    this.targetWidth = 0;
    this.targetHeight = 0;
  }
}
