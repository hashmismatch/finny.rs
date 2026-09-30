// A collapsible view of a JSON value that highlights the differences to the previous snapshot.

import { useState } from 'preact/hooks';
import { html } from './util.js';

const NO_PREV = Symbol('no previous value');

function same(a, b) {
  if (a === b) return true;
  if (typeof a !== 'object' || typeof b !== 'object' || a === null || b === null) return false;
  return JSON.stringify(a) === JSON.stringify(b);
}

function Primitive({ value }) {
  if (value === null) return html`<span class="j-null">null</span>`;
  switch (typeof value) {
    case 'string': return html`<span class="j-string">"${value}"</span>`;
    case 'number': return html`<span class="j-number">${value}</span>`;
    case 'boolean': return html`<span class="j-bool">${String(value)}</span>`;
    default: return html`<span>${String(value)}</span>`;
  }
}

function Node({ name, value, prev, depth, openDepth }) {
  const isObject = value !== null && typeof value === 'object';
  const [open, setOpen] = useState(depth < openDepth);
  const changed = prev !== NO_PREV && !same(value, prev);
  const added = prev === undefined && value !== undefined;
  const cls = `j-row${changed ? (added ? ' j-added' : ' j-changed') : ''}`;
  const key = name != null ? html`<span class="j-key">${name}</span><span class="j-colon">: </span>` : null;

  if (!isObject) {
    return html`
      <div class=${cls} style=${{ '--depth': depth }} title=${changed && !added ? `was: ${JSON.stringify(prev)}` : undefined}>
        <span class="j-toggle"></span>${key}<${Primitive} value=${value} />
      </div>`;
  }

  const isArray = Array.isArray(value);
  const entries = isArray ? value.map((v, i) => [i, v]) : Object.entries(value);
  const summary = isArray ? `[${entries.length}]` : `{${entries.length}}`;
  const prevChild = (k) => {
    if (prev === NO_PREV) return NO_PREV;
    if (prev == null || typeof prev !== 'object') return undefined;
    return prev[k];
  };

  return html`
    <div class=${cls} style=${{ '--depth': depth }}>
      <button class="j-toggle" aria-expanded=${open} onClick=${() => setOpen(!open)}>${open ? '▾' : '▸'}</button>
      ${key}<span class="j-summary" onClick=${() => setOpen(!open)}>${summary}</span>
    </div>
    ${open && entries.map(([k, v]) => html`
      <${Node} key=${k} name=${k} value=${v} prev=${prevChild(k)} depth=${depth + 1} openDepth=${openDepth} />
    `)}`;
}

/** `prev` is the value in the previous snapshot, the differences are highlighted. */
export function JsonTree({ value, prev = NO_PREV, openDepth = 2 }) {
  if (value === undefined) return html`<div class="muted">No value.</div>`;
  return html`<div class="json-tree"><${Node} value=${value} prev=${prev} depth=${0} openDepth=${openDepth} /></div>`;
}

export { NO_PREV };
