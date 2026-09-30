// The panels next to the diagram: the values, the event, the trace and the history.

import { useState } from 'preact/hooks';
import { html, shortName, formatTime, formatDuration, eventLabel, getIn, pathKey } from './util.js';
import { JsonTree, NO_PREV } from './json-tree.js';
import { current, previous, snapshots, selectedNode, selectedTransition, selectSeq, meta, transitionNames } from './api.js';

const OPT_IN_HINT = html`
  <div class="hint">
    <p>This FSM doesn't serialize its values.</p>
    <p>Opt in with <code>fsm.serde();</code> in the FSM's builder function and derive
    <code>serde::Serialize</code> on its context, states and events. Needs finny's <code>serde</code> feature.</p>
  </div>`;

function ResultBadge({ snapshot }) {
  return snapshot.error
    ? html`<span class="badge badge-error" title="The dispatch failed">${snapshot.error}</span>`
    : html`<span class="badge badge-ok">handled</span>`;
}

function Section({ title, children, aside }) {
  return html`
    <section class="section">
      <header class="section-head"><h3>${title}</h3>${aside}</header>
      ${children}
    </section>`;
}

function prevAt(path) {
  const p = previous.value;
  if (!p || !p.values) return NO_PREV;
  return getIn(p.values, path);
}

function ActiveStates({ snapshot }) {
  const info = meta.value?.info;
  const machineName = (path) => {
    if (!path.length) return info?.id ?? 'FSM';
    return shortName(path[path.length - 1]);
  };
  return html`
    <table class="kv">
      ${snapshot.active.map((a) => html`
        <tr key=${pathKey(a.path)}>
          <th style=${{ paddingLeft: `${a.path.length * 12 + 4}px` }}>${machineName(a.path)}</th>
          <td>${a.states.map((s, i) => html`
            <span class=${s == null ? 'chip chip-muted' : 'chip chip-active'} title=${`region ${i}`}>${s ?? 'stopped'}</span>`)}
          </td>
        </tr>`)}
    </table>`;
}

export function StatePanel() {
  const snapshot = current.value;
  if (!snapshot) return html`<div class="empty">No snapshots yet. Start the FSM.</div>`;
  const node = selectedNode.value;
  const values = snapshot.values;

  const valuesError = snapshot.values_error
    ? html`<div class="hint hint-error">The values failed to serialize: ${snapshot.values_error}</div>`
    : null;

  if (node) {
    const value = values ? getIn(values, node.valuePath) : undefined;
    return html`
      <${Section}
        title=${html`${node.isMachine ? 'Sub-machine' : 'State'} <code>${node.label}</code>`}
        aside=${html`<button class="link" onClick=${() => { selectedNode.value = null; }}>Show all</button>`}>
        <div class="type-name" title=${node.typeName}>${node.typeName}</div>
        ${valuesError}
        ${values ? html`<${JsonTree} value=${value} prev=${prevAt(node.valuePath)} openDepth=${3} />` : OPT_IN_HINT}
      <//>`;
  }

  return html`
    <${Section} title="Current states"><${ActiveStates} snapshot=${snapshot} /><//>
    ${valuesError}
    ${values
      ? html`
        <${Section} title="Context"><${JsonTree} value=${values.context} prev=${prevAt(['context'])} /><//>
        <${Section} title="States"><${JsonTree} value=${values.states} prev=${prevAt(['states'])} openDepth=${1} /><//>`
      : OPT_IN_HINT}`;
}

export function EventPanel() {
  const snapshot = current.value;
  if (!snapshot) return html`<div class="empty">No snapshots yet.</div>`;
  const e = snapshot.event;
  return html`
    <${Section} title=${html`Event <code>${eventLabel(e, snapshot.trace)}</code>`} aside=${html`<${ResultBadge} snapshot=${snapshot} />`}>
      <table class="kv">
        <tr><th>Sequence</th><td>#${snapshot.seq}</td></tr>
        <tr><th>Kind</th><td>${e.kind}</td></tr>
        <tr><th>Time</th><td>${formatTime(snapshot.timestamp_ms)}</td></tr>
        <tr><th>Duration</th><td>${formatDuration(snapshot.duration_us)}</td></tr>
        ${snapshot.error && html`<tr><th>Error</th><td class="error-text">${snapshot.error}</td></tr>`}
      </table>
    <//>
    ${e.kind === 'Event' && html`
      <${Section} title="Payload">
        ${e.value !== null && e.value !== undefined ? html`<${JsonTree} value=${e.value} openDepth=${4} />` : OPT_IN_HINT}
      <//>`}`;
}

const TRACE_ICONS = {
  event: '✉', transition: '➜', sub_machine: '⊞', timer: '⏱', guard: '◆', state_exit: '⇤', state_enter: '⇥',
  action: '⚙', dispatch_error: '✗', error: '⚠', info: 'ℹ'
};

/** A transition, described by the FSM's metadata, the type name in the tooltip. */
function TransitionName({ typeName }) {
  const name = transitionNames.value.get(typeName);
  return html`<span class="transition-name" title=${typeName}>${name ?? shortName(typeName)}</span>`;
}

function TraceText({ entry }) {
  switch (entry.kind) {
    case 'event': return html`Event <b>${eventLabel(entry.event)}</b>`;
    case 'transition': return html`Transition <${TransitionName} typeName=${entry.type_name} />`;
    case 'sub_machine': return html`Into the sub-machine <b title=${entry.type_name}>${shortName(entry.type_name)}</b>`;
    case 'timer': return html`Timer <code>${entry.timer}</code>`;
    case 'guard': return html`Guard of <${TransitionName} typeName=${entry.type_name} />${' '}
      <span class=${entry.result ? 'pass' : 'fail'}>${entry.result ? 'passed' : 'rejected'}</span>`;
    case 'state_exit': return html`Exit <b>${entry.state}</b>`;
    case 'state_enter': return html`Enter <b>${entry.state}</b>`;
    case 'action': return html`Action of <${TransitionName} typeName=${entry.type_name} />`;
    case 'dispatch_error': return html`<span class="error-text">Failed: ${entry.error}</span>`;
    case 'error': return html`<span class="error-text">${entry.message}: ${entry.error}</span>`;
    case 'info': return html`<span class="muted">${entry.message}</span>`;
    default: return html`${entry.kind}`;
  }
}

export function TracePanel() {
  const snapshot = current.value;
  if (!snapshot) return html`<div class="empty">No snapshots yet.</div>`;
  return html`
    <${Section} title=${html`Trace of <code>${eventLabel(snapshot.event, snapshot.trace)}</code>`} aside=${html`<${ResultBadge} snapshot=${snapshot} />`}>
      ${snapshot.trace.length === 0
        ? html`<div class="muted">Nothing happened, no transition handled the event.</div>`
        : html`<ol class="trace">
          ${snapshot.trace.map((entry, i) => html`
            <li key=${i} class=${`trace-${entry.kind}`} style=${{ '--depth': entry.depth }}>
              <span class="trace-icon" aria-hidden="true">${TRACE_ICONS[entry.kind] ?? '•'}</span>
              <span class="trace-text"><${TraceText} entry=${entry} /></span>
            </li>`)}
        </ol>`}
    <//>`;
}

/** The transitions of a snapshot, as `from → to`. */
function transitionsSummary(snapshot) {
  const out = [];
  let exit = null;
  for (const e of snapshot.trace) {
    if (e.kind === 'state_exit') exit = e.state;
    if (e.kind === 'state_enter') {
      out.push(exit ? `${exit} → ${e.state}` : `→ ${e.state}`);
      exit = null;
    }
  }
  return out.join(', ');
}

export function HistoryPanel() {
  const [query, setQuery] = useState('');
  const list = snapshots.value;
  const selected = current.value?.seq;
  const transition = selectedTransition.value;
  const q = query.trim().toLowerCase();

  const rows = list.filter((s) => {
    if (transition && !s.trace.some((e) => e.kind === 'transition' && e.type_name === transition.typeName)) return false;
    if (!q) return true;
    return eventLabel(s.event, s.trace).toLowerCase().includes(q)
      || transitionsSummary(s).toLowerCase().includes(q)
      || (s.error ?? '').toLowerCase().includes(q)
      || String(s.seq) === q;
  }).reverse();

  return html`
    <div class="history-tools">
      <input type="search" placeholder="Filter by event, state or error" value=${query}
        onInput=${(e) => setQuery(e.currentTarget.value)} aria-label="Filter the history" />
      ${transition && html`
        <span class="chip chip-filter">
          ${transitionNames.value.get(transition.typeName) ?? transition.label ?? shortName(transition.typeName)}
          <button class="link" aria-label="Clear the transition filter" onClick=${() => { selectedTransition.value = null; }}>✕</button>
        </span>`}
    </div>
    ${rows.length === 0
      ? html`<div class="empty">${list.length ? 'No matching events.' : 'No snapshots yet.'}</div>`
      : html`<ul class="history">
        ${rows.map((s) => html`
          <li key=${s.seq}>
            <button class=${`history-row${s.seq === selected ? ' selected' : ''}${s.error ? ' failed' : ''}`} onClick=${() => selectSeq(s.seq)}>
              <span class="h-seq">#${s.seq}</span>
              <span class="h-event">${eventLabel(s.event, s.trace)}</span>
              <span class="h-time">${formatTime(s.timestamp_ms)}</span>
              <span class="h-detail">${s.error ? html`<span class="error-text">${s.error}</span>` : transitionsSummary(s) || html`<span class="muted">no state change</span>`}</span>
            </button>
          </li>`)}
      </ul>`}`;
}
