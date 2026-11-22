// Direction artistique de FRONTIER : couleurs reprises du prototype.

export type Rgb = [number, number, number];

/** Indexé par l'identifiant de terrain du moteur (crates/engine/src/terrain.rs). */
export const BIOME_COLORS: Rgb[] = [
  [24, 55, 77], // océan
  [42, 91, 116], // eaux peu profondes
  [194, 177, 125], // plage
  [105, 139, 87], // prairie
  [151, 145, 98], // steppe
  [57, 103, 70], // forêt
  [78, 125, 91], // marais
  [137, 137, 103], // plateau
  [119, 119, 110], // collines
  [164, 158, 143], // montagne
  [218, 222, 222], // sommet enneigé
  [210, 216, 216], // toundra
];

export const TERRAIN_LABELS = [
  'Océan',
  'Eaux peu profondes',
  'Plage',
  'Prairie',
  'Steppe',
  'Forêt',
  'Marais',
  'Plateau',
  'Collines',
  'Montagne',
  'Sommet enneigé',
  'Toundra',
];

/** Terrains infranchissables (eau, montagnes). */
export function isPassable(terrain: number): boolean {
  return terrain > 1 && terrain !== 9 && terrain !== 10;
}

export const RIVER_BANK: Rgb = [91, 99, 77];
export const RIVER_WATER: Rgb = [55, 119, 145];
export const BACKGROUND: Rgb = [24, 55, 77];

export const PLAYER_COLORS: Rgb[] = [
  [210, 85, 75],
  [75, 130, 210],
  [220, 175, 65],
  [145, 100, 190],
  [65, 165, 125],
  [205, 115, 165],
  [130, 155, 70],
  [70, 175, 185],
  [230, 135, 60],
  [120, 120, 225],
  [190, 190, 95],
  [175, 80, 110],
];

export const CIV_NAMES = [
  'Aurelia',
  'Valoria',
  'Nerath',
  'Eldoria',
  'Dravenn',
  'Caldris',
  'Veyra',
  'Ortheon',
  'Meridia',
  'Solmara',
  'Kethra',
  'Ysmir',
];

export function rgbCss([r, g, b]: Rgb, alpha = 1): string {
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}
