// Point d'entrée : un seul worker moteur, un menu, un écran de jeu.

import { GameScreen } from './app/game';
import { Menu, type MenuResult } from './app/menu';
import type { FromWorker } from './engine/protocol';

const app = document.getElementById('app')!;
const worker = new Worker(new URL('./engine/worker.ts', import.meta.url), { type: 'module' });

let game: GameScreen | null = null;

const menu = new Menu((result: MenuResult) => startGame(result));
app.appendChild(menu.el);

function startGame(result: MenuResult): void {
  menu.showError('');
  menu.el.classList.add('hidden');
  game?.destroy();
  try {
    game = new GameScreen(worker, result, backToMenu);
    app.appendChild(game.el);
  } catch (err) {
    game = null;
    menu.el.classList.remove('hidden');
    menu.showError(err instanceof Error ? err.message : String(err));
  }
}

function backToMenu(): void {
  game?.destroy();
  game = null;
  menu.el.classList.remove('hidden');
}

worker.onmessage = (event: MessageEvent<FromWorker>) => {
  const msg = event.data;
  if (msg.kind === 'agents') {
    menu.setAgents(msg.agents);
  } else if (msg.kind === 'error' && !game) {
    menu.showError(msg.message);
  } else {
    game?.handle(msg);
  }
};

worker.onerror = (event) => {
  menu.showError(`Le moteur n’a pas pu démarrer : ${event.message}`);
};

worker.postMessage({ kind: 'hello' });
