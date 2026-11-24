// Calque 2D par-dessus la carte : gisements, bâtiments, bateaux, noms des
// civilisations, case survolée.

import { Camera } from './camera';
import { rgbCss, type Rgb } from './palette';
import type { Deposit } from '../engine/protocol';
import { formatNumber } from '../ui/format';
import { DEPOSIT_INFO } from '../ui/labels';

export interface Label {
  /** Position en cases. */
  x: number;
  y: number;
  tiles: number;
  name: string;
  /** Soldats à la maison (ce qui peut frapper en retour). */
  soldiers: number;
  /** Force de la garnison (défense pure, entraînement compris). */
  garrison: number;
  color: Rgb;
}

/** Éléments à dessiner, en coordonnées de case (centre). */
export interface Markers {
  deposits: { x: number; y: number; kind: Deposit }[];
  /** `progress` : 1 = terminé, sinon avancement du chantier. */
  buildings: { x: number; y: number; glyph: string; color: Rgb; progress: number }[];
  boats: { x: number; y: number; tx: number; ty: number; color: Rgb; troops: number }[];
}

export class Overlay {
  readonly canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
  }

  resize(): void {
    const dpr = window.devicePixelRatio || 1;
    const w = Math.round(this.canvas.clientWidth * dpr);
    const h = Math.round(this.canvas.clientHeight * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width = w;
      this.canvas.height = h;
    }
  }

  draw(
    cam: Camera,
    labels: Label[],
    markers: Markers,
    hover: [number, number] | null,
    hoverColor: string,
    highlightDeposits: Deposit | 'all' | null,
  ): void {
    const ctx = this.ctx;
    const dpr = window.devicePixelRatio || 1;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const scale = cam.zoom / dpr; // pixels CSS par case

    if (hover) {
      const [tx, ty] = hover;
      const sx = (tx - cam.x) * scale;
      const sy = (ty - cam.y) * scale;
      ctx.strokeStyle = hoverColor;
      ctx.lineWidth = 2;
      const size = Math.max(scale, 6);
      ctx.strokeRect(sx + scale / 2 - size / 2, sy + scale / 2 - size / 2, size, size);
    }

    this.drawDeposits(cam, scale, markers, highlightDeposits);
    this.drawBuildings(cam, scale, markers);
    this.drawBoats(cam, scale, markers);

    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.lineJoin = 'round';
    for (const label of labels) {
      const size = Math.min(34, Math.sqrt(label.tiles) * scale * 0.3);
      if (size < 9) continue;
      const sx = (label.x - cam.x) * scale;
      const sy = (label.y - cam.y) * scale;
      if (sx < -200 || sy < -50 || sx > this.canvas.width / dpr + 200 || sy > this.canvas.height / dpr + 50) {
        continue;
      }
      ctx.font = `bold ${size.toFixed(1)}px Consolas, "Cascadia Mono", monospace`;
      ctx.lineWidth = Math.max(2, size * 0.18);
      ctx.strokeStyle = 'rgba(10, 14, 20, 0.85)';
      ctx.fillStyle = 'rgba(245, 240, 225, 0.96)';
      ctx.strokeText(label.name, sx, sy);
      ctx.fillText(label.name, sx, sy);

      const small = size * 0.62;
      ctx.font = `${small.toFixed(1)}px Consolas, "Cascadia Mono", monospace`;
      ctx.lineWidth = Math.max(2, small * 0.2);
      ctx.fillStyle = rgbCss(label.color.map((c) => Math.min(255, c + 60)) as Rgb);
      // Force de la garnison, puis soldats à la maison. Les troupes parties
      // en campagne ne se voient pas d'ici.
      const troops = (label.garrison > 0 ? `🛡 ${formatNumber(label.garrison)}  ` : '') + `⚔ ${formatNumber(label.soldiers)}`;
      ctx.strokeText(troops, sx, sy + size * 0.95);
      ctx.fillText(troops, sx, sy + size * 0.95);
    }
  }

  private visible(sx: number, sy: number, margin: number): boolean {
    const dpr = window.devicePixelRatio || 1;
    return sx > -margin && sy > -margin && sx < this.canvas.width / dpr + margin && sy < this.canvas.height / dpr + margin;
  }

  /** Gisements : petits symboles colorés, plus visibles en mode construction. */
  private drawDeposits(cam: Camera, scale: number, markers: Markers, focus: Deposit | 'all' | null): void {
    if (scale < 0.9 && !focus) return;
    const ctx = this.ctx;
    const base = Math.max(3, Math.min(8, scale * 1.2));
    const pulse = 1 + 0.25 * Math.sin(performance.now() / 250);
    for (const d of markers.deposits) {
      const info = DEPOSIT_INFO[d.kind];
      if (!info) continue;
      const highlight = focus === 'all' || focus === d.kind;
      if (scale < 0.9 && !highlight) continue;
      const r = highlight ? base * 1.5 * pulse : base;
      const sx = (d.x - cam.x) * scale;
      const sy = (d.y - cam.y) * scale;
      if (!this.visible(sx, sy, 20)) continue;
      ctx.save();
      ctx.translate(sx, sy);
      ctx.globalAlpha = highlight ? 1 : 0.85;
      ctx.fillStyle = info.color;
      ctx.strokeStyle = 'rgba(10, 14, 20, 0.9)';
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      switch (d.kind) {
        case 'fertile': // épis : losange
          ctx.moveTo(0, -r);
          ctx.lineTo(r * 0.7, 0);
          ctx.lineTo(0, r);
          ctx.lineTo(-r * 0.7, 0);
          break;
        case 'grove': // arbre : triangle
          ctx.moveTo(0, -r);
          ctx.lineTo(r * 0.85, r * 0.7);
          ctx.lineTo(-r * 0.85, r * 0.7);
          break;
        case 'quarry': // bloc de pierre
          ctx.rect(-r * 0.75, -r * 0.6, r * 1.5, r * 1.2);
          break;
        case 'iron': // hexagone
          for (let i = 0; i < 6; i++) {
            const a = (Math.PI / 3) * i + Math.PI / 6;
            if (i === 0) ctx.moveTo(Math.cos(a) * r, Math.sin(a) * r);
            else ctx.lineTo(Math.cos(a) * r, Math.sin(a) * r);
          }
          break;
        default:
          break;
      }
      ctx.closePath();
      ctx.stroke();
      ctx.fill();
      ctx.restore();
    }
  }

  /** Bâtiments : pastille aux couleurs du joueur avec une lettre ; anneau d'avancement pour les chantiers. */
  private drawBuildings(cam: Camera, scale: number, markers: Markers): void {
    const ctx = this.ctx;
    const r = Math.max(5, Math.min(11, scale * 1.6));
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    for (const b of markers.buildings) {
      const sx = (b.x - cam.x) * scale;
      const sy = (b.y - cam.y) * scale;
      if (!this.visible(sx, sy, 20)) continue;
      const done = b.progress >= 1;
      ctx.globalAlpha = done ? 1 : 0.75;
      ctx.beginPath();
      ctx.arc(sx, sy, r + 2, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(15, 20, 25, 1)';
      ctx.fill();
      ctx.beginPath();
      ctx.arc(sx, sy, r, 0, Math.PI * 2);
      ctx.fillStyle = rgbCss(b.color);
      ctx.fill();
      if (r >= 6) {
        ctx.font = `bold ${Math.round(r * 1.15)}px Consolas, monospace`;
        ctx.fillStyle = 'rgba(250, 245, 230, 1)';
        ctx.fillText(b.glyph, sx, sy + 0.5);
      }
      if (!done) {
        ctx.globalAlpha = 1;
        ctx.strokeStyle = 'rgba(245, 220, 140, 1)';
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.arc(sx, sy, r + 3.5, -Math.PI / 2, -Math.PI / 2 + Math.PI * 2 * b.progress);
        ctx.stroke();
      }
    }
    ctx.globalAlpha = 1;
  }

  private drawBoats(cam: Camera, scale: number, markers: Markers): void {
    const ctx = this.ctx;
    const size = Math.max(5, Math.min(12, scale * 1.8));
    for (const b of markers.boats) {
      const sx = (b.x - cam.x) * scale;
      const sy = (b.y - cam.y) * scale;
      const tx = (b.tx - cam.x) * scale;
      const ty = (b.ty - cam.y) * scale;
      // Trajet restant, en pointillés.
      ctx.setLineDash([4, 5]);
      ctx.strokeStyle = rgbCss(b.color, 0.7);
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(sx, sy);
      ctx.lineTo(tx, ty);
      ctx.stroke();
      ctx.setLineDash([]);
      // Coque orientée vers la destination.
      const angle = Math.atan2(ty - sy, tx - sx);
      ctx.save();
      ctx.translate(sx, sy);
      ctx.rotate(angle);
      ctx.beginPath();
      ctx.moveTo(size, 0);
      ctx.lineTo(-size * 0.7, -size * 0.5);
      ctx.lineTo(-size * 0.5, 0);
      ctx.lineTo(-size * 0.7, size * 0.5);
      ctx.closePath();
      ctx.fillStyle = rgbCss(b.color);
      ctx.strokeStyle = 'rgba(15, 20, 25, 1)';
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fill();
      ctx.restore();
      if (scale > 2.5) {
        ctx.font = 'bold 11px Consolas, monospace';
        ctx.textAlign = 'center';
        ctx.lineWidth = 3;
        ctx.strokeStyle = 'rgba(10, 14, 20, 0.85)';
        ctx.fillStyle = 'rgba(245, 240, 225, 0.95)';
        const text = formatNumber(b.troops);
        ctx.strokeText(text, sx, sy - size - 4);
        ctx.fillText(text, sx, sy - size - 4);
      }
    }
  }
}

/**
 * Position des noms : pour chaque joueur, la case possédée la plus proche
 * du centre de gravité de son territoire (reste dans le territoire, même
 * s'il a une forme concave).
 */
export function computeAnchors(owners: Uint16Array, width: number, maxId: number): Map<number, [number, number]> {
  const sumX = new Float64Array(maxId + 1);
  const sumY = new Float64Array(maxId + 1);
  const count = new Float64Array(maxId + 1);
  for (let i = 0; i < owners.length; i++) {
    const o = owners[i];
    if (o === 0 || o > maxId) continue;
    sumX[o] += i % width;
    sumY[o] += (i / width) | 0;
    count[o] += 1;
  }
  const best = new Float64Array(maxId + 1).fill(Infinity);
  const bestTile = new Int32Array(maxId + 1).fill(-1);
  for (let i = 0; i < owners.length; i++) {
    const o = owners[i];
    if (o === 0 || o > maxId) continue;
    const dx = (i % width) - sumX[o] / count[o];
    const dy = ((i / width) | 0) - sumY[o] / count[o];
    const d = dx * dx + dy * dy;
    if (d < best[o]) {
      best[o] = d;
      bestTile[o] = i;
    }
  }
  const anchors = new Map<number, [number, number]>();
  for (let id = 1; id <= maxId; id++) {
    const t = bestTile[id];
    if (t >= 0) anchors.set(id, [(t % width) + 0.5, ((t / width) | 0) + 0.5]);
  }
  return anchors;
}
