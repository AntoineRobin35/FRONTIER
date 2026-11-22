/** Caméra 2D. Coordonnées en cases ; `zoom` = pixels (physiques) par case. */
export class Camera {
  x = 0;
  y = 0;
  zoom = 4;
  minZoom = 0.5;
  maxZoom = 48;

  /** Cadre toute la carte dans la vue. */
  fit(mapW: number, mapH: number, viewW: number, viewH: number): void {
    this.zoom = Math.min(viewW / mapW, viewH / mapH) * 0.98;
    this.minZoom = this.zoom * 0.7;
    this.x = (mapW - viewW / this.zoom) / 2;
    this.y = (mapH - viewH / this.zoom) / 2;
  }

  /** Centre la vue sur une case avec un zoom donné. */
  centerOn(tx: number, ty: number, zoom: number, viewW: number, viewH: number): void {
    this.zoom = Math.min(this.maxZoom, Math.max(this.minZoom, zoom));
    this.x = tx - viewW / this.zoom / 2;
    this.y = ty - viewH / this.zoom / 2;
  }

  zoomAt(px: number, py: number, factor: number): void {
    const wx = this.x + px / this.zoom;
    const wy = this.y + py / this.zoom;
    this.zoom = Math.min(this.maxZoom, Math.max(this.minZoom, this.zoom * factor));
    this.x = wx - px / this.zoom;
    this.y = wy - py / this.zoom;
  }

  /** Déplace la vue de `dx`, `dy` pixels physiques. */
  pan(dx: number, dy: number): void {
    this.x += dx / this.zoom;
    this.y += dy / this.zoom;
  }

  /** Empêche de perdre la carte de vue. */
  clamp(mapW: number, mapH: number, viewW: number, viewH: number): void {
    const visW = viewW / this.zoom;
    const visH = viewH / this.zoom;
    const marginX = visW * 0.25;
    const marginY = visH * 0.25;
    if (visW >= mapW) this.x = (mapW - visW) / 2;
    else this.x = Math.max(-marginX, Math.min(this.x, mapW - visW + marginX));
    if (visH >= mapH) this.y = (mapH - visH) / 2;
    else this.y = Math.max(-marginY, Math.min(this.y, mapH - visH + marginY));
  }

  toTile(px: number, py: number): [number, number] {
    return [this.x + px / this.zoom, this.y + py / this.zoom];
  }
}
