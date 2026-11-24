// Fait tourner le moteur (WebAssembly) hors du fil principal, pour que
// l'affichage reste fluide. En solo, ce worker joue le rôle du « serveur » :
// il cadence les tours et fait jouer les IA.

import init, { WasmSession, agents_json } from '../../wasm-pkg/frontier.js';
import type { FromWorker, MapPayload, ToWorker } from './protocol';

const ctx = self as unknown as Worker;
const ready = init();

let session: WasmSession | null = null;
let seed = 0;
let human = 0;
let tickMs = 100;
let speed = 1;
let paused = false;
let timer: ReturnType<typeof setTimeout> | undefined;
let last = 0;
let accumulated = 0;
/** Part des soldats envoyés par un clic (pour les estimations d'attaque). */
let ratio = 500;
let lastInsight = 0;
/** Les aperçus coûtent plus cher qu'un tour : quelques fois par seconde suffit. */
const INSIGHT_INTERVAL_MS = 300;

const MAX_STEPS_PER_FRAME = 8;

function post(msg: FromWorker, transfer: Transferable[] = []): void {
  ctx.postMessage(msg, transfer);
}

function stop(): void {
  if (timer !== undefined) clearTimeout(timer);
  timer = undefined;
  session?.free();
  session = null;
}

function loop(): void {
  timer = setTimeout(loop, 8);
  const now = performance.now();
  const dt = now - last;
  last = now;
  // Aperçus aussi en pause : c'est là qu'on prend le temps de décider.
  if (session && now - lastInsight >= INSIGHT_INTERVAL_MS) postInsight(now);
  if (!session || paused) return;

  accumulated += dt * speed;
  let steps = 0;
  while (accumulated >= tickMs && steps < MAX_STEPS_PER_FRAME) {
    accumulated -= tickMs;
    steps += 1;
    const changes = session.step();
    const snapshot = JSON.parse(session.snapshot_json());
    post({ kind: 'tick', snapshot, changes }, [changes.buffer]);
  }
  // Si le moteur prend du retard, on ralentit plutôt que d'accumuler.
  if (steps === MAX_STEPS_PER_FRAME) accumulated = 0;
}

function postInsight(now: number): void {
  if (!session) return;
  lastInsight = now;
  post({ kind: 'insight', insight: JSON.parse(session.insight_json(human, ratio)) });
}

ctx.onmessage = async (event: MessageEvent<ToWorker>) => {
  const msg = event.data;
  try {
    await ready;
    switch (msg.kind) {
      case 'hello':
        post({ kind: 'agents', agents: JSON.parse(agents_json()) });
        break;

      case 'start': {
        stop();
        session = new WasmSession(JSON.stringify(msg.setup));
        seed = msg.setup.seed;
        human = msg.human;
        tickMs = session.tick_ms();
        const map: MapPayload = {
          seed,
          width: session.width(),
          height: session.height(),
          tickMs,
          terrain: session.terrain(),
          elevation: session.elevation(),
          river: session.river(),
          rivers: session.rivers(),
          deposits: session.deposits(),
          depositSites: session.deposit_sites(),
          owners: session.owners(),
          rules: JSON.parse(session.config_json()),
        };
        post({ kind: 'map', map }, [
          map.terrain.buffer,
          map.elevation.buffer,
          map.river.buffer,
          map.rivers.buffer,
          map.deposits.buffer,
          map.depositSites.buffer,
          map.owners.buffer,
        ]);
        paused = false;
        accumulated = 0;
        last = performance.now();
        loop();
        break;
      }

      case 'intent':
        session?.submit(human, JSON.stringify(msg.intent));
        break;

      case 'speed':
        speed = msg.speed;
        break;

      case 'pause':
        paused = msg.paused;
        break;

      case 'ratio':
        ratio = msg.ratio;
        postInsight(performance.now());
        break;

      case 'stop':
        stop();
        break;
    }
  } catch (err) {
    post({ kind: 'error', message: err instanceof Error ? err.message : String(err) });
  }
};
