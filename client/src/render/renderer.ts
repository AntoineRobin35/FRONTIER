// Rendu de la carte avec WebGL2. La carte entière tient dans quelques
// textures (1 texel = 1 case) et les shaders calculent chaque pixel :
// le coût ne dépend pas de la taille de la carte, et le zoom est gratuit.
//
// Passes : 1. terrain (couleur + relief + grain)  2. rivières (maillage)
//          3. territoires (remplissage + frontières, transparents)

import type { MapPayload } from '../engine/protocol';
import { Camera } from './camera';
import { computeHillshade, LIGHT_MAX, LIGHT_MIN } from './hillshade';
import { BACKGROUND, BIOME_COLORS, RIVER_BANK, RIVER_WATER, type Rgb } from './palette';

const FULLSCREEN_VS = `#version 300 es
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

const COMMON = `
precision highp float;
precision highp int;
precision highp usampler2D;
uniform vec2 uCam;
uniform float uZoom;
uniform float uCanvasH;
uniform vec2 uMapSize;
vec2 worldPos() {
  return uCam + vec2(gl_FragCoord.x, uCanvasH - gl_FragCoord.y) / uZoom;
}
bool outside(vec2 t) {
  return t.x < 0.0 || t.y < 0.0 || t.x >= uMapSize.x || t.y >= uMapSize.y;
}
float hash(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

float vnoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  float a = hash(i);
  float b = hash(i + vec2(1.0, 0.0));
  float c = hash(i + vec2(0.0, 1.0));
  float d = hash(i + vec2(1.0, 1.0));
  return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Déforme légèrement la grille au zoom : les bords des cases deviennent
// irréguliers (côtes, biomes, frontières) au lieu de gros carrés.
// Le terrain et les territoires utilisent la même déformation.
vec2 warped(vec2 t) {
  vec2 w = vec2(vnoise(t * 1.7 + 3.1), vnoise(t * 1.7 + 9.7)) - 0.5;
  w += (vec2(vnoise(t * 4.3 + 1.3), vnoise(t * 4.3 + 5.9)) - 0.5) * 0.4;
  float amount = 0.85 * smoothstep(2.5, 7.0, uZoom);
  return clamp(t + w * amount, vec2(0.0), uMapSize - 0.001);
}`;

const TERRAIN_FS = `#version 300 es
${COMMON}
uniform usampler2D uTerrain;
uniform usampler2D uOwnerT;
uniform sampler2D uLight;
uniform vec3 uBiome[12];
uniform vec3 uBackground;
uniform float uScale;
uniform vec2 uLightRange;
out vec4 outColor;


void main() {
  vec2 t = worldPos();
  if (outside(t)) {
    outColor = vec4(uBackground / 255.0, 1.0);
    return;
  }
  ivec2 ti = ivec2(warped(t));
  int b = int(texelFetch(uTerrain, ti, 0).r);
  vec3 base = uBiome[b];

  float light = mix(uLightRange.x, uLightRange.y, texture(uLight, t / uMapSize).r);
  bool water = b <= 1;
  if (b == 8 || b == 9 || b == 10) light = clamp(light * 1.08, 0.70, 1.30);

  // Grain à plusieurs échelles, comme le prototype.
  float broad = vnoise(t / (5.0 * uScale)) - 0.5;
  float medium = vnoise(t / uScale + 17.0) - 0.5;
  float cell = 0.125 * uScale;
  float fine = (hash(floor(t / cell)) - 0.5) * smoothstep(1.0, 3.0, cell * uZoom);
  float tex = broad * 0.15 + medium * 0.10 + fine * 0.055;
  if (water) tex *= 0.35;

  vec3 color = base * light * (1.0 + tex) / 255.0;
  // Lisibilité : sous un territoire, le terrain s'efface un peu (désaturé,
  // éclairci) pour laisser la couleur du joueur ressortir.
  if (texelFetch(uOwnerT, ti, 0).r != 0u) {
    float grey = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(color, vec3(grey), 0.45) * 0.9 + 0.06;
  }
  outColor = vec4(clamp(color, 0.0, 1.0), 1.0);
}`;

const TERRITORY_FS = `#version 300 es
${COMMON}
uniform usampler2D uOwner;
uniform sampler2D uPalette;
uniform float uBorder;
uniform uint uHover;
out vec4 outColor;

uint ownerAt(ivec2 p) {
  if (p.x < 0 || p.y < 0 || p.x >= int(uMapSize.x) || p.y >= int(uMapSize.y)) return 0u;
  return texelFetch(uOwner, p, 0).r;
}

void main() {
  vec2 t0 = worldPos();
  if (outside(t0)) discard;
  vec2 t = warped(t0);
  ivec2 ti = ivec2(t);
  uint o = ownerAt(ti);
  if (o == 0u) discard;

  vec3 color = texelFetch(uPalette, ivec2(int(o), 0), 0).rgb;
  vec2 f = fract(t);
  float w = uBorder;
  bool edge =
      (f.x < w && ownerAt(ti + ivec2(-1, 0)) != o) ||
      (f.x > 1.0 - w && ownerAt(ti + ivec2(1, 0)) != o) ||
      (f.y < w && ownerAt(ti + ivec2(0, -1)) != o) ||
      (f.y > 1.0 - w && ownerAt(ti + ivec2(0, 1)) != o);

  float alpha = edge ? 0.92 : (o == uHover ? 0.48 : 0.36);
  outColor = vec4(color, alpha);
}`;

const RIVER_VS = `#version 300 es
in vec2 aCenter;
in vec2 aNormal;
in float aHalf;
uniform vec2 uCam;
uniform float uZoom;
uniform vec2 uCanvas;
uniform float uExtra;
uniform float uMinHalf;
void main() {
  vec2 p = aCenter + aNormal * (max(aHalf, uMinHalf) + uExtra * 0.5);
  vec2 s = (p - uCam) * uZoom;
  gl_Position = vec4(s.x / uCanvas.x * 2.0 - 1.0, 1.0 - s.y / uCanvas.y * 2.0, 0.0, 1.0);
}`;

const RIVER_FS = `#version 300 es
precision highp float;
uniform vec3 uColor;
out vec4 outColor;
void main() {
  outColor = vec4(uColor / 255.0, 1.0);
}`;

interface Program {
  program: WebGLProgram;
  /** Emplacement d'un uniform (null s'il est inutilisé par le shader). */
  loc(name: string): WebGLUniformLocation | null;
}

export class Renderer {
  readonly canvas: HTMLCanvasElement;
  private gl: WebGL2RenderingContext;
  private terrainProg: Program;
  private territoryProg: Program;
  private riverProg: Program;
  private emptyVao: WebGLVertexArrayObject;

  private terrainTex: WebGLTexture | null = null;
  private lightTex: WebGLTexture | null = null;
  private ownerTex: WebGLTexture | null = null;
  private paletteTex: WebGLTexture | null = null;
  private riverVao: WebGLVertexArrayObject | null = null;
  private riverBuffer: WebGLBuffer | null = null;
  private riverVertexCount = 0;

  private mapW = 0;
  private mapH = 0;
  private scale = 1;
  private hoverOwner = 0;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const gl = canvas.getContext('webgl2', { antialias: true, alpha: false });
    if (!gl) throw new Error('WebGL2 indisponible : mets à jour ton navigateur ou tes pilotes graphiques.');
    this.gl = gl;
    this.terrainProg = this.compile(FULLSCREEN_VS, TERRAIN_FS);
    this.territoryProg = this.compile(FULLSCREEN_VS, TERRITORY_FS);
    this.riverProg = this.compile(RIVER_VS, RIVER_FS);
    this.emptyVao = gl.createVertexArray()!;
  }

  /** Adapte la taille du canevas à l'écran. Renvoie la taille en pixels physiques. */
  resize(): [number, number] {
    const dpr = window.devicePixelRatio || 1;
    const w = Math.max(1, Math.round(this.canvas.clientWidth * dpr));
    const h = Math.max(1, Math.round(this.canvas.clientHeight * dpr));
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width = w;
      this.canvas.height = h;
    }
    return [w, h];
  }

  setMap(map: MapPayload): void {
    const gl = this.gl;
    this.dispose();
    this.mapW = map.width;
    this.mapH = map.height;
    this.scale = map.width / 240;

    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

    this.terrainTex = this.texture(gl.NEAREST);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8UI, map.width, map.height, 0, gl.RED_INTEGER, gl.UNSIGNED_BYTE, map.terrain);

    const light = computeHillshade(map.elevation, map.width, map.height, this.scale);
    this.lightTex = this.texture(gl.LINEAR);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, map.width, map.height, 0, gl.RED, gl.UNSIGNED_BYTE, light);

    this.ownerTex = this.texture(gl.NEAREST);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R16UI, map.width, map.height, 0, gl.RED_INTEGER, gl.UNSIGNED_SHORT, map.owners);

    this.paletteTex = this.texture(gl.NEAREST);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 256, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(256 * 4));

    this.buildRivers(map.rivers);
  }

  /** Couleurs des joueurs, indexées par identifiant. */
  setPalette(colors: Map<number, Rgb>): void {
    const gl = this.gl;
    const data = new Uint8Array(256 * 4);
    for (const [id, [r, g, b]] of colors) {
      if (id < 0 || id > 255) continue;
      data.set([r, g, b, 255], id * 4);
    }
    gl.bindTexture(gl.TEXTURE_2D, this.paletteTex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 256, 1, gl.RGBA, gl.UNSIGNED_BYTE, data);
  }

  /** Envoie au GPU les lignes `rowMin..=rowMax` de la grille des propriétaires. */
  updateOwners(owners: Uint16Array, rowMin: number, rowMax: number): void {
    if (rowMax < rowMin) return;
    const gl = this.gl;
    gl.bindTexture(gl.TEXTURE_2D, this.ownerTex);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    const rows = rowMax - rowMin + 1;
    gl.texSubImage2D(
      gl.TEXTURE_2D, 0, 0, rowMin, this.mapW, rows,
      gl.RED_INTEGER, gl.UNSIGNED_SHORT,
      owners.subarray(rowMin * this.mapW, (rowMax + 1) * this.mapW),
    );
  }

  setHoverOwner(id: number): void {
    this.hoverOwner = id;
  }

  draw(cam: Camera): void {
    const gl = this.gl;
    const [cw, ch] = [this.canvas.width, this.canvas.height];
    const dpr = window.devicePixelRatio || 1;
    gl.viewport(0, 0, cw, ch);
    gl.clearColor(BACKGROUND[0] / 255, BACKGROUND[1] / 255, BACKGROUND[2] / 255, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    if (!this.terrainTex) return;

    // 1. Terrain
    gl.disable(gl.BLEND);
    let p = this.terrainProg;
    gl.useProgram(p.program);
    this.setView(p, cam, ch);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.terrainTex);
    gl.uniform1i(p.loc('uTerrain'), 0);
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.lightTex);
    gl.uniform1i(p.loc('uLight'), 1);
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.ownerTex);
    gl.uniform1i(p.loc('uOwnerT'), 2);
    gl.uniform3fv(p.loc('uBiome[0]'), BIOME_COLORS.flat());
    gl.uniform3fv(p.loc('uBackground'), BACKGROUND);
    gl.uniform1f(p.loc('uScale'), this.scale);
    gl.uniform2f(p.loc('uLightRange'), LIGHT_MIN, LIGHT_MAX);
    gl.bindVertexArray(this.emptyVao);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // 2. Rivières : berges puis eau
    if (this.riverVao && this.riverVertexCount > 0) {
      p = this.riverProg;
      gl.useProgram(p.program);
      gl.uniform2f(p.loc('uCam'), cam.x, cam.y);
      gl.uniform1f(p.loc('uZoom'), cam.zoom);
      gl.uniform2f(p.loc('uCanvas'), cw, ch);
      gl.uniform1f(p.loc('uMinHalf'), (0.45 * dpr) / cam.zoom);
      gl.bindVertexArray(this.riverVao);
      gl.uniform1f(p.loc('uExtra'), Math.max(0.18 * this.scale, (1.5 * dpr) / cam.zoom));
      gl.uniform3fv(p.loc('uColor'), RIVER_BANK);
      gl.drawArrays(gl.TRIANGLES, 0, this.riverVertexCount);
      gl.uniform1f(p.loc('uExtra'), 0);
      gl.uniform3fv(p.loc('uColor'), RIVER_WATER);
      gl.drawArrays(gl.TRIANGLES, 0, this.riverVertexCount);
    }

    // 3. Territoires
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    p = this.territoryProg;
    gl.useProgram(p.program);
    this.setView(p, cam, ch);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.ownerTex);
    gl.uniform1i(p.loc('uOwner'), 0);
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.paletteTex);
    gl.uniform1i(p.loc('uPalette'), 1);
    gl.uniform1f(p.loc('uBorder'), Math.min(0.5, (1.6 * dpr) / cam.zoom));
    gl.uniform1ui(p.loc('uHover'), this.hoverOwner);
    gl.bindVertexArray(this.emptyVao);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.bindVertexArray(null);
  }

  dispose(): void {
    const gl = this.gl;
    for (const tex of [this.terrainTex, this.lightTex, this.ownerTex, this.paletteTex]) {
      if (tex) gl.deleteTexture(tex);
    }
    if (this.riverBuffer) gl.deleteBuffer(this.riverBuffer);
    if (this.riverVao) gl.deleteVertexArray(this.riverVao);
    this.terrainTex = this.lightTex = this.ownerTex = this.paletteTex = null;
    this.riverBuffer = null;
    this.riverVao = null;
    this.riverVertexCount = 0;
  }

  // ------------------------------------------------------------------

  private setView(p: Program, cam: Camera, canvasH: number): void {
    const gl = this.gl;
    gl.uniform2f(p.loc('uCam'), cam.x, cam.y);
    gl.uniform1f(p.loc('uZoom'), cam.zoom);
    gl.uniform1f(p.loc('uCanvasH'), canvasH);
    gl.uniform2f(p.loc('uMapSize'), this.mapW, this.mapH);
  }

  /** Construit le maillage des rivières : deux triangles par segment. */
  private buildRivers(flat: Float32Array): void {
    const gl = this.gl;
    const verts: number[] = [];
    let k = 0;
    const pathCount = flat[k++] ?? 0;
    for (let p = 0; p < pathCount; p++) {
      const n = flat[k++];
      const pts: [number, number, number][] = [];
      for (let i = 0; i < n; i++) {
        pts.push([flat[k], flat[k + 1], flat[k + 2]]);
        k += 3;
      }
      // Normale en chaque point.
      const ring: [number, number, number, number, number][] = [];
      for (let i = 0; i < pts.length; i++) {
        const a = pts[Math.max(0, i - 1)];
        const b = pts[Math.min(pts.length - 1, i + 1)];
        const dx = b[0] - a[0];
        const dy = b[1] - a[1];
        const len = Math.hypot(dx, dy);
        if (len < 1e-4) continue;
        ring.push([pts[i][0], pts[i][1], -dy / len, dx / len, pts[i][2] / 2]);
      }
      for (let i = 0; i + 1 < ring.length; i++) {
        const [ax, ay, anx, any, ah] = ring[i];
        const [bx, by, bnx, bny, bh] = ring[i + 1];
        // gauche A, droite A, gauche B | droite A, droite B, gauche B
        verts.push(ax, ay, anx, any, ah, ax, ay, -anx, -any, ah, bx, by, bnx, bny, bh);
        verts.push(ax, ay, -anx, -any, ah, bx, by, -bnx, -bny, bh, bx, by, bnx, bny, bh);
      }
    }

    this.riverVertexCount = verts.length / 5;
    if (this.riverVertexCount === 0) return;

    this.riverVao = gl.createVertexArray();
    gl.bindVertexArray(this.riverVao);
    this.riverBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.riverBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(verts), gl.STATIC_DRAW);
    const prog = this.riverProg.program;
    const stride = 5 * 4;
    const attr = (name: string, size: number, offset: number) => {
      const loc = gl.getAttribLocation(prog, name);
      if (loc < 0) return;
      gl.enableVertexAttribArray(loc);
      gl.vertexAttribPointer(loc, size, gl.FLOAT, false, stride, offset);
    };
    attr('aCenter', 2, 0);
    attr('aNormal', 2, 8);
    attr('aHalf', 1, 16);
    gl.bindVertexArray(null);
  }

  private texture(filter: number): WebGLTexture {
    const gl = this.gl;
    const tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, filter);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, filter);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return tex;
  }

  private compile(vsSource: string, fsSource: string): Program {
    const gl = this.gl;
    const shader = (type: number, source: string) => {
      const s = gl.createShader(type)!;
      gl.shaderSource(s, source);
      gl.compileShader(s);
      if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
        throw new Error(`Shader : ${gl.getShaderInfoLog(s)}`);
      }
      return s;
    };
    const program = gl.createProgram()!;
    gl.attachShader(program, shader(gl.VERTEX_SHADER, vsSource));
    gl.attachShader(program, shader(gl.FRAGMENT_SHADER, fsSource));
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      throw new Error(`Programme : ${gl.getProgramInfoLog(program)}`);
    }
    const cache = new Map<string, WebGLUniformLocation | null>();
    const loc = (name: string) => {
      if (!cache.has(name)) cache.set(name, gl.getUniformLocation(program, name));
      return cache.get(name) ?? null;
    };
    return { program, loc };
  }
}
