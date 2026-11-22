// Éclairage du relief, calculé une fois par carte : altitude floutée, puis
// pente éclairée depuis le haut-gauche (même formule que le prototype).

const LIGHT_MIN = 0.76;
const LIGHT_MAX = 1.24;

export { LIGHT_MIN, LIGHT_MAX };

function blur(src: Float32Array, w: number, h: number, sigma: number): Float32Array {
  const radius = Math.max(1, Math.ceil(sigma * 3));
  const kernel = new Float32Array(radius * 2 + 1);
  let sum = 0;
  for (let i = -radius; i <= radius; i++) {
    const v = Math.exp(-(i * i) / (2 * sigma * sigma));
    kernel[i + radius] = v;
    sum += v;
  }
  for (let i = 0; i < kernel.length; i++) kernel[i] /= sum;

  const tmp = new Float32Array(w * h);
  const out = new Float32Array(w * h);
  for (let y = 0; y < h; y++) {
    const row = y * w;
    for (let x = 0; x < w; x++) {
      let acc = 0;
      for (let k = -radius; k <= radius; k++) {
        const xx = Math.min(w - 1, Math.max(0, x + k));
        acc += src[row + xx] * kernel[k + radius];
      }
      tmp[row + x] = acc;
    }
  }
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      let acc = 0;
      for (let k = -radius; k <= radius; k++) {
        const yy = Math.min(h - 1, Math.max(0, y + k));
        acc += tmp[yy * w + x] * kernel[k + radius];
      }
      out[y * w + x] = acc;
    }
  }
  return out;
}

/** Renvoie la lumière par case, encodée sur 0..255 entre LIGHT_MIN et LIGHT_MAX. */
export function computeHillshade(elevation: Uint16Array, w: number, h: number, scale: number): Uint8Array {
  const e = new Float32Array(w * h);
  for (let i = 0; i < e.length; i++) e[i] = elevation[i] / 65535;
  const b = blur(e, w, h, 1.2 * scale);

  const out = new Uint8Array(w * h);
  const strength = 3 * scale;
  for (let y = 0; y < h; y++) {
    const up = Math.max(0, y - 1) * w;
    const down = Math.min(h - 1, y + 1) * w;
    for (let x = 0; x < w; x++) {
      const left = Math.max(0, x - 1);
      const right = Math.min(w - 1, x + 1);
      const gx = (b[y * w + right] - b[y * w + left]) / 2;
      const gy = (b[down + x] - b[up + x]) / 2;
      const light = Math.min(LIGHT_MAX, Math.max(LIGHT_MIN, 1 - strength * (gx + gy)));
      out[y * w + x] = Math.round(((light - LIGHT_MIN) / (LIGHT_MAX - LIGHT_MIN)) * 255);
    }
  }
  return out;
}
