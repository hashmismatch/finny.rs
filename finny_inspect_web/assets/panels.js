// The panels next to the diagram: the values, the event, the trace and the history.

import { useState } from 'preact/hooks';
import { html, shortName, formatTime, formatDuration, formatMs, eventLabel, getIn, pathKey, timerDue, timerKey, useTimersNow } from './util.js';
import { JsonTree, NO_PREV } from './json-tree.js';
import { current, previous, snapshots, selectedNode, selectedTransition, selectSeq, meta, transitionNames, follow } from './api.js';

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

/**
 * All the timers declared by the machine and its sub-machines, grouped by machine:
 * `[{ path, name, timers: [{ timer, stateId, node }] }]`. `node` is the state, as selected in the diagram.
 */
export function declaredTimers(info) {
  const machines = [];
  const walk = (machine, path, valuePrefix) => {
    const group = { path, name: path.length ? shortName(path[path.length - 1]) : machine.id, timers: [] };
    machines.push(group);
    for (const region of machine.regions) {
      for (const s of region.states) {
        const valuePath = [...valuePrefix, 'states', s.storage_field];
        const node = { nodeId: `state:${pathKey(path)}|${s.id}`, label: s.id, typeName: s.type_name, valuePath, isMachine: !!s.sub_machine };
        for (const t of s.timers) group.timers.push({ timer: t.id, stateId: s.id, node });
        if (s.sub_machine) walk(s.sub_machine, [...path, s.type_name], valuePath);
      }
    }
  };
  if (info) walk(info, [], []);
  return machines.filter((m) => m.timers.length);
}

const TIMER_BADGES = {
  running: 'badge-running',
  expired: 'badge-ok',
  failed: 'badge-error',
  cancelled: 'badge-muted',
  disabled: 'badge-muted'
};

function TimerRow({ entry, status, active, now }) {
  const selected = selectedNode.value?.nodeId === entry.node.nodeId;
  let progress = null;
  let when = null;
  let title;
  if (status?.status === 'running') {
    const since = status.last_triggered_ms ?? status.started_ms;
    progress = status.timeout_ms > 0 ? Math.min(1, (now - since) / status.timeout_ms) : 1;
    const left = timerDue(status) - now;
    if (left > 0) {
      when = `${formatMs(left)} left`;
    } else {
      when = `due ${formatMs(-left)} ago`;
      title = 'Due, waiting for the timers to be dispatched, for example with dispatch_timer_events';
    }
  } else if (status?.status === 'expired') {
    progress = 1;
    when = `fired at ${formatTime(status.last_triggered_ms)}`;
  } else if (status?.status === 'cancelled') {
    when = 'cancelled on exit';
  } else if (status?.status === 'failed') {
    when = 'the timers service failed to create it';
  } else if (status?.status === 'disabled') {
    when = 'disabled by its setup';
  }

  const details = [];
  if (status && status.status !== 'failed' && status.status !== 'disabled') {
    details.push(status.renew ? `↻ every ${formatMs(status.timeout_ms)}` : `once after ${formatMs(status.timeout_ms)}`);
    if (status.triggers) {
      details.push(`triggered ${status.triggers}×`);
      if (status.renew) details.push(`last at ${formatTime(status.last_triggered_ms)}`);
    }
    if (!status.cancel_on_state_exit) details.push('kept on exit');
  }

  return html`
    <li>
      <button class=${`timer-row${selected ? ' selected' : ''}`} onClick=${() => { selectedNode.value = selected ? null : entry.node; }}
        title=${`Select the state ${entry.stateId} in the diagram`}>
        <span class="timer-head">
          <span class="timer-name">⏱ <b>${entry.timer}</b></span>
          <span class=${active ? 'chip chip-active' : 'chip chip-muted'} title=${active ? 'The current state' : 'Not the current state'}>${entry.stateId}</span>
          <span class="spacer"></span>
          ${status?.restored && html`<span class="badge badge-muted" title="Re-created when the machine was restored">restored</span>`}
          <span class=${`badge ${status ? TIMER_BADGES[status.status] : 'badge-muted'}`}>${status?.status ?? 'idle'}</span>
        </span>
        ${progress != null && html`
          <span class=${`timer-bar${status.status === 'running' && progress >= 1 ? ' due' : ''}${status.status === 'expired' ? ' done' : ''}`}
            role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow=${Math.round(progress * 100)}>
            <span style=${{ width: `${progress * 100}%` }}></span>
          </span>`}
        <span class="timer-detail" title=${title}>
          ${when ?? html`<span class="muted">not started yet</span>`}${details.length ? html`<span class="muted"> · ${details.join(' · ')}</span>` : ''}
        </span>
      </button>
    </li>`;
}

export function TimersPanel() {
  const snapshot = current.value;
  const live = follow.value;
  const now = useTimersNow(snapshot, live);
  const machines = declaredTimers(meta.value?.info);

  if (!meta.value) return html`<div class="empty">No FSM selected.</div>`;
  if (!machines.length) return html`<div class="empty">This FSM has no timers.</div>`;

  const statuses = new Map((snapshot?.timers ?? []).map((t) => [timerKey(t.path, t.timer), t]));
  const isActive = (path, stateId) => {
    const key = pathKey(path);
    return !!snapshot?.active.some((a) => pathKey(a.path) === key && a.states.includes(stateId));
  };

  return html`
    <${Section} title="Timers"
      aside=${html`<span class="muted">${snapshot ? (live ? 'live' : html`at #${snapshot.seq}, ${formatTime(now)}`) : 'no snapshots yet'}</span>`}>
      ${machines.map((m) => html`
        <div key=${pathKey(m.path)} class="timer-group" style=${{ paddingLeft: `${m.path.length * 12}px` }}>
          ${machines.length > 1 && html`<h4 title=${m.path[m.path.length - 1] ?? ''}>${m.name}</h4>`}
          <ul class="timers">
            ${m.timers.map((entry) => html`
              <${TimerRow} key=${entry.timer} entry=${entry} status=${statuses.get(timerKey(m.path, entry.timer))}
                active=${isActive(m.path, entry.stateId)} now=${now} />`)}
          </ul>
        </div>`)}
    <//>`;
}
