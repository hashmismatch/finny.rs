import { h } from 'preact';
import htm from 'htm';

export const html = htm.bind(h);

/** `my_crate::module::MyType<T>` -> `MyType<T>` */
export function shortName(typeName) {
  if (!typeName) return '';
  let depth = 0;
  let start = 0;
  for (let i = 0; i < typeName.length; i++) {
    const c = typeName[i];
    if (c === '<') depth++;
    else if (c === '>') depth--;
    else if (depth === 0 && c === ':' && typeName[i + 1] === ':') start = i + 2;
  }
  return typeName.slice(start);
}

export function formatTime(ms) {
  const d = new Date(ms);
  const pad = (n, w = 2) => String(n).padStart(w, '0');
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}.${pad(d.getMilliseconds(), 3)}`;
}

export function formatDuration(us) {
  if (us < 1000) return `${us} µs`;
  if (us < 1000000) return `${(us / 1000).toFixed(us < 10000 ? 2 : 1)} ms`;
  return `${(us / 1000000).toFixed(2)} s`;
}

function singleKey(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const keys = Object.keys(value);
  return keys.length === 1 ? keys[0] : null;
}

/**
 * The label of a snapshot's event. An event for a sub-machine is named after the sub-machine's
 * events enum, `CycleEvents`, it's shown as `Cycle › Next`. The inner event comes from the
 * serialized value or else from the trace.
 */
export function eventLabel(event, trace) {
  switch (event.kind) {
    case 'Start': return 'Start';
    case 'Stop': return 'Stop';
    case 'Timer': return `Timer ${event.timer}`;
  }

  const sub = singleKey(event.value);
  if (sub && event.name === `${sub}Events`) {
    const inner = singleKey(event.value[sub]);
    if (inner) return `${sub} › ${inner}`;
  }
  if (trace && event.name.endsWith('Events')) {
    const nested = trace.find((e) => e.kind === 'event' && e.depth > 0);
    if (nested) return `${event.name.slice(0, -'Events'.length)} › ${eventLabel(nested.event)}`;
  }
  return event.name;
}

/** localStorage can be unavailable, the UI works without it. */
export const storage = {
  get(key, fallback) {
    try {
      const v = localStorage.getItem(key);
      return v == null ? fallback : JSON.parse(v);
    } catch {
      return fallback;
    }
  },
  set(key, value) {
    try {
      if (value === undefined) localStorage.removeItem(key);
      else localStorage.setItem(key, JSON.stringify(value));
    } catch { /* ignored */ }
  }
};

export function getIn(value, path) {
  let v = value;
  for (const key of path) {
    if (v == null) return undefined;
    v = v[key];
  }
  return v;
}

export function pathKey(path) {
  return path.join(' > ');
}
