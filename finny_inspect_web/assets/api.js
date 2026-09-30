// The state of the inspector and its connection to the backend.

import { signal, computed, batch } from '@preact/signals';
import { storage } from './util.js';

export const instances = signal([]);
export const selectedId = signal(null);
/** `{ info, plantuml }` of the selected instance. */
export const meta = signal(null);
/** The snapshots of the selected instance, ordered by `seq`. */
export const snapshots = signal([]);
/** The inspected snapshot, `null` follows the newest one. */
export const selectedSeq = signal(null);
export const connection = signal('connecting');
/** The clicked state or sub-machine of the diagram: `{ nodeId, label, valuePath }`. */
export const selectedNode = signal(null);
/** The clicked transition, filters the history: `{ typeName, label }`. */
export const selectedTransition = signal(null);
export const tab = signal(storage.get('finny-inspect:tab', 'state'));

export const follow = computed(() => selectedSeq.value == null);

export const currentIndex = computed(() => {
  const list = snapshots.value;
  if (!list.length) return -1;
  if (selectedSeq.value == null) return list.length - 1;
  const i = list.findIndex((s) => s.seq === selectedSeq.value);
  return i < 0 ? list.length - 1 : i;
});

export const current = computed(() => snapshots.value[currentIndex.value] ?? null);
export const previous = computed(() => snapshots.value[currentIndex.value - 1] ?? null);

export const selectedInstance = computed(() => instances.value.find((i) => i.id === selectedId.value) ?? null);

/** Transition type name -> `Idle → Running on Go`, including the sub-machines' transitions. */
export const transitionNames = computed(() => {
  const names = new Map();
  const walk = (machine) => {
    for (const region of machine.regions) {
      for (const t of region.transitions) {
        const event = t.event.kind === 'Event' ? t.event.id : t.event.kind;
        let label;
        switch (t.kind.kind) {
          case 'Normal': label = `${t.kind.from ?? '[*]'} → ${t.kind.to ?? '[*]'}`; break;
          case 'SelfTransition': label = `${t.kind.state} ↻`; break;
          default: label = `${t.kind.state} (internal)`;
        }
        names.set(t.type_name, `${label} on ${event}`);
      }
      for (const s of region.states) {
        if (s.sub_machine) walk(s.sub_machine);
      }
    }
  };
  if (meta.value) walk(meta.value.info);
  return names;
});

export function selectSeq(seq) {
  const list = snapshots.value;
  selectedSeq.value = list.length && seq === list[list.length - 1].seq ? null : seq;
}

export function step(delta) {
  const list = snapshots.value;
  if (!list.length) return;
  const i = Math.max(0, Math.min(list.length - 1, currentIndex.value + delta));
  selectSeq(list[i].seq);
}

export function setTab(t) {
  tab.value = t;
  storage.set('finny-inspect:tab', t);
}

const api = (path) => `api/instances${path}`;
const instancePath = (id) => api(`/${encodeURIComponent(id)}`);

async function getJson(url) {
  const r = await fetch(url, { cache: 'no-store' });
  if (!r.ok) throw new Error(`${r.status} ${await r.text()}`);
  return r.json();
}

let instanceSource = null;
let loadedAttachedAt = null;

function historyLen() {
  return selectedInstance.value?.history_len || 100;
}

function append(snapshot) {
  const list = snapshots.value;
  if (list.length && snapshot.seq <= list[list.length - 1].seq) return;
  const next = list.concat([snapshot]);
  const max = Math.max(historyLen(), 1);
  snapshots.value = next.length > max ? next.slice(next.length - max) : next;
  // an inspected snapshot that fell out of the history
  if (selectedSeq.value != null && !snapshots.value.some((s) => s.seq === selectedSeq.value)) {
    selectedSeq.value = snapshots.value[0]?.seq ?? null;
  }
}

async function reloadSnapshots(id) {
  const list = await getJson(`${instancePath(id)}/snapshots`);
  if (selectedId.value !== id) return;
  snapshots.value = list;
}

export async function selectInstance(id) {
  if (instanceSource) {
    instanceSource.close();
    instanceSource = null;
  }

  batch(() => {
    selectedId.value = id;
    meta.value = null;
    snapshots.value = [];
    selectedSeq.value = null;
    selectedNode.value = null;
    selectedTransition.value = null;
  });

  if (id == null) return;
  try {
    history.replaceState(null, '', `#${encodeURIComponent(id)}`);
  } catch { /* ignored */ }

  loadedAttachedAt = instances.value.find((i) => i.id === id)?.attached_at_ms ?? null;
  connection.value = 'connecting';
  try {
    const [m, list] = await Promise.all([getJson(`${instancePath(id)}/meta`), getJson(`${instancePath(id)}/snapshots`)]);
    if (selectedId.value !== id) return;
    batch(() => {
      meta.value = m;
      snapshots.value = list;
    });
  } catch (e) {
    console.error(e);
    connection.value = 'error';
    return;
  }

  const last = snapshots.value[snapshots.value.length - 1]?.seq;
  const source = new EventSource(`${instancePath(id)}/stream${last != null ? `?after=${last}` : ''}`);
  instanceSource = source;
  source.addEventListener('open', () => { connection.value = 'live'; });
  source.addEventListener('error', () => { connection.value = source.readyState === EventSource.CLOSED ? 'error' : 'connecting'; });
  source.addEventListener('snapshot', (e) => {
    if (instanceSource !== source) return;
    append(JSON.parse(e.data));
  });
  // the stream fell behind
  source.addEventListener('reset', () => reloadSnapshots(id).catch(console.error));
}

export function connect() {
  const source = new EventSource(api('/stream'));
  source.addEventListener('instances', (e) => {
    const list = JSON.parse(e.data);
    instances.value = list;

    const current = list.find((i) => i.id === selectedId.value);
    if (!current) {
      let wanted = null;
      try {
        wanted = decodeURIComponent(location.hash.slice(1));
      } catch { /* ignored */ }
      const pick = list.find((i) => i.id === wanted) ?? list[0];
      if (pick && pick.id !== selectedId.value) selectInstance(pick.id);
    } else if (loadedAttachedAt != null && current.attached_at_ms !== loadedAttachedAt) {
      // the application was restarted, the same name is a new instance
      selectInstance(current.id);
    }
  });
  source.addEventListener('error', () => {
    if (!instanceSource) connection.value = 'error';
  });
}
