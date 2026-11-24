// Textes affichés : noms, icônes et descriptions des éléments du jeu.

import type { BuildingKind, Cost, Deposit, TechId } from '../engine/protocol';
import { formatNumber } from './format';

export const RESOURCES: { key: keyof Cost; name: string; icon: string; color: string }[] = [
  { key: 'food', name: 'Nourriture', icon: '◆', color: '#e3c25a' },
  { key: 'wood', name: 'Bois', icon: '▲', color: '#7fb86a' },
  { key: 'stone', name: 'Pierre', icon: '■', color: '#b7b8bf' },
  { key: 'iron', name: 'Fer', icon: '⬣', color: '#c98a5a' },
];

export const JOB_NAMES = ['Cueilleurs', 'Bûcherons', 'Mineurs', 'Forgerons'];
/** Bâtiment qui débloque chaque métier (null : dès le départ). */
export const JOB_UNLOCK: (BuildingKind | null)[] = [null, null, 'quarry', 'mine'];
export const UNIT_NAMES = ['Guerriers', 'Archers', 'Cavaliers'];

export interface BuildingInfo {
  name: string;
  /** Lettre affichée dans l'icône sur la carte. */
  glyph: string;
  key: string;
  effect: string;
}

export const BUILDING_INFO: Record<BuildingKind, BuildingInfo> = {
  city: { name: 'Ville', glyph: 'V', key: 'V', effect: 'Logement : +population maximale.' },
  barracks: { name: 'Caserne', glyph: 'C', key: 'K', effect: 'Forme les soldats plus vite.' },
  farm: { name: 'Ferme', glyph: 'F', key: 'R', effect: 'Sur terres fertiles : emplois de cueilleurs très productifs.' },
  sawmill: { name: 'Scierie', glyph: 'S', key: 'B', effect: 'Sur bosquet : emplois de bûcherons très productifs.' },
  archery: { name: 'Archerie', glyph: 'A', key: 'G', effect: 'Débloque les archers (forts en défense et en forêt).' },
  stable: { name: 'Écurie', glyph: 'É', key: 'U', effect: 'Débloque les cavaliers (rapides en plaine).' },
  quarry: { name: 'Carrière', glyph: 'P', key: 'P', effect: 'Sur gisement de pierre : débloque les mineurs et la pierre.' },
  fort: { name: 'Fort', glyph: 'T', key: 'T', effect: 'Défense ×3 sur les cases proches. Détruit si l’ennemi le prend.' },
  mine: { name: 'Mine', glyph: 'M', key: 'M', effect: 'Sur gisement de fer : débloque les forgerons et le fer.' },
  academy: { name: 'Académie', glyph: '✦', key: 'L', effect: 'Recherches qui améliorent ton économie et ton armée. Une seule.' },
};

export interface TechInfo {
  name: string;
  /** Effet d'un niveau, en une phrase courte. */
  effect: string;
}

export const TECH_INFO: Record<TechId, TechInfo> = {
  agriculture: { name: 'Agriculture', effect: 'Nourriture +15 % par niveau' },
  forestry: { name: 'Sylviculture', effect: 'Bois +15 % par niveau' },
  masonry: { name: 'Maçonnerie', effect: 'Pierre +15 % par niveau' },
  metallurgy: { name: 'Métallurgie', effect: 'Fer +15 % par niveau' },
  architecture: { name: 'Architecture', effect: 'Chantiers 25 % plus rapides par niveau' },
  urbanism: { name: 'Urbanisme', effect: 'Logement +10 % par niveau' },
  granaries: { name: 'Greniers', effect: 'Réserves +30 % par niveau' },
  drill: { name: 'Entraînement', effect: 'Formation des soldats +25 % par niveau' },
  blades: { name: 'Lames', effect: 'Attaque des guerriers +15 % par niveau' },
  bowmanship: { name: 'Tir à l’arc', effect: 'Attaque des archers +15 % par niveau' },
  horsemanship: { name: 'Équitation', effect: 'Attaque des cavaliers +15 % par niveau' },
  shields: { name: 'Boucliers', effect: 'Défense des soldats +12 % par niveau' },
  discipline: { name: 'Discipline', effect: 'Défense de la garnison +15 % par niveau' },
  instructors: { name: 'Instructeurs', effect: 'Entraînement de la garnison +40 % plus rapide par niveau' },
  colonization: { name: 'Colonisation', effect: 'Expansion sur les terres libres +25 % par niveau' },
  logistics: { name: 'Logistique', effect: 'Pertes en retraite −25 % par niveau' },
  fortification: { name: 'Fortification', effect: 'Défense des forts +20 % par niveau' },
};

export const DEPOSIT_INFO: Record<Deposit, { name: string; color: string } | null> = {
  none: null,
  fertile: { name: 'Terres fertiles', color: '#e3c25a' },
  grove: { name: 'Bosquet', color: '#4f9a4a' },
  quarry: { name: 'Gisement de pierre', color: '#c4c5cc' },
  iron: { name: 'Gisement de fer', color: '#c27a4a' },
};

export function costText(cost: Cost | null | undefined): string {
  if (!cost) return '';
  return RESOURCES.filter((r) => cost[r.key] > 0)
    .map((r) => `${formatNumber(cost[r.key])} ${r.name.toLowerCase()}`)
    .join(' · ');
}

export function canPay(stock: Cost, cost: Cost): boolean {
  return RESOURCES.every((r) => stock[r.key] >= cost[r.key]);
}

/** Niveau en chiffres romains (I, II, III). */
export function roman(level: number): string {
  return ['', 'I', 'II', 'III', 'IV', 'V'][level] ?? String(level);
}

/** Coût en icônes : « ▲ 600 · ■ 300 ». */
export function costIcons(cost: Cost | null | undefined): string {
  if (!cost) return '';
  return RESOURCES.filter((r) => cost[r.key] > 0)
    .map((r) => `${r.icon} ${formatNumber(cost[r.key])}`)
    .join(' · ');
}

/** Nombre signé : « +12 », « −3 », « 0 ». */
export function signed(n: number): string {
  if (n > 0) return `+${formatNumber(n)}`;
  if (n < 0) return `−${formatNumber(-n)}`;
  return '0';
}
