import { render } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import { html, eventLabel, formatTime, storage, useTimersNow } from './util.js';
import { Diagram } from './graph.js';
import { StatePanel, EventPanel, TracePanel, HistoryPanel, TimersPanel } from './panels.js';
import {
  instances, selectedId, selectedInstance, meta, snapshots, current, currentIndex, connection, follow,
  selectedNode, selectedTransition, tab, setTab, selectInstance, selectSeq, step, connect
} from './api.js';

function Toast({ message }) {
  return message ? html`<div class="toast" role="status">${message}</div>` : null;
}

function useToast() {
  const [message, setMessage] = useState(null);
  const timer = useRef(null);
  const show = (m) => {
    setMessage(m);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setMessage(null), 1800);
  };
  return [message, show];
}

function TopBar() {
  const list = instances.value;
  const instance = selectedInstance.value;
  const status = connection.value;
  const statusLabel = { live: 'Live', connecting: 'Connecting…', error: 'Disconnected' }[status];

  return html`
    <header class="topbar">
      <div class="brand">
        <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
          <rect x="2" y="3" width="9" height="6" rx="2" fill="none" stroke="currentColor" stroke-width="1.6"/>
          <rect x="13" y="15" width="9" height="6" rx="2" fill="none" stroke="currentColor" stroke-width="1.6"/>
          <path d="M6.5 9v5.5a2 2 0 0 0 2 2H12" fill="none" stroke="currentColor" stroke-width="1.6"/>
          <path d="M11 14.5l2 2-2 2" fill="none" stroke="currentColor" stroke-width="1.6"/>
        </svg>
        <span>finny <b>inspector</b></span>
      </div>
      <label class="instance-picker">
        <span class="sr-only">FSM instance</span>
        <select value=${selectedId.value ?? ''} onChange=${(e) => selectInstance(e.currentTarget.value)} disabled=${!list.length}>
          ${!list.length && html`<option value="">No FSMs attached</option>`}
          ${list.map((i) => html`<option value=${i.id}>${i.id} — ${i.fsm}${i.attached ? '' : ' (detached)'}</option>`)}
        </select>
      </label>
      ${instance && !instance.attached && html`<span class="badge badge-muted" title="The FSM was dropped, its history is kept">detached</span>`}
      <div class="spacer"></div>
      ${meta.value && html`<code class="type-name topbar-type" title=${meta.value.info.type_name}>${meta.value.info.type_name}</code>`}
      <span class=${`status status-${status}`}><span class="dot"></span>${statusLabel}</span>
    </header>`;
}

function DiagramView() {
  const container = useRef(null);
  const diagram = useRef(null);
  const [message, toast] = useToast();
  const info = meta.value?.info;

  useEffect(() => {
    if (!info || !container.current) return;
    const d = new Diagram(container.current, info, {
      onSelectNode: (n) => {
        selectedNode.value = n;
        if (n) setTab('state');
      },
      onSelectEdge: (t) => {
        selectedTransition.value = t;
        if (t) setTab('history');
      }
    });
    diagram.current = d;
    d.ready.then(() => {
      d.show(current.peek());
      d.showTimers(current.peek(), Date.now());
    });
    const resize = new ResizeObserver(() => d.cy.resize());
    resize.observe(container.current);
    return () => {
      resize.disconnect();
      d.destroy();
      diagram.current = null;
    };
  }, [info]);

  const snapshot = current.value;
  useEffect(() => { diagram.current?.show(snapshot); }, [snapshot, info]);
  const timersNow = useTimersNow(snapshot, follow.value);
  useEffect(() => { diagram.current?.showTimers(snapshot, timersNow); }, [snapshot, timersNow, info]);

  const nodeId = selectedNode.value?.nodeId;
  const transition = selectedTransition.value?.typeName;
  useEffect(() => { diagram.current?.select(nodeId, transition); }, [nodeId, transition, info]);

  const copyPlantUml = async () => {
    try {
      await navigator.clipboard.writeText(meta.value.plantuml);
      toast('PlantUML copied to the clipboard');
    } catch {
      const w = window.open('', '_blank');
      if (w) w.document.body.innerText = meta.value.plantuml;
    }
  };

  const exportPng = async () => {
    const blob = await diagram.current?.png();
    if (!blob) return;
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${info.id}.png`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
  };

  return html`
    <section class="diagram" aria-label="State diagram">
      <div class="diagram-canvas" ref=${container}></div>
      ${!info && html`<div class="diagram-empty">${instances.value.length ? 'Loading…' : html`
        <div><h2>Waiting for an FSM</h2><p>Attach one with <code>${'inspector.attach::<MyFsm>("name")'}</code>.</p></div>`}</div>`}
      ${snapshot?.error && html`
        <div class="diagram-banner" role="status">
          <b>${eventLabel(snapshot.event, snapshot.trace)}</b> wasn't handled: ${snapshot.error}
        </div>`}
      ${info && html`
        <div class="toolbar" role="toolbar" aria-label="Diagram">
          <button title="Zoom in" aria-label="Zoom in" onClick=${() => diagram.current?.zoom(1.25)}>+</button>
          <button title="Zoom out" aria-label="Zoom out" onClick=${() => diagram.current?.zoom(0.8)}>−</button>
          <button title="Fit to screen" aria-label="Fit to screen" onClick=${() => diagram.current?.fit()}>⤢</button>
          <button title="Reset the layout, discarding the dragged positions" aria-label="Reset the layout" onClick=${() => diagram.current?.layout(true)}>↺</button>
          <span class="toolbar-sep"></span>
          <button title="Export as PNG" onClick=${exportPng}>PNG</button>
          <button title="Copy the PlantUML source" onClick=${copyPlantUml}>UML</button>
        </div>
        <div class="legend" aria-hidden="true">
          <span><i class="lg-active"></i>current</span>
          <span><i class="lg-taken"></i>taken</span>
          <span><i class="lg-rejected"></i>guard rejected</span>
          <span><i class="lg-timer"></i>timer running</span>
        </div>`}
      <${Toast} message=${message} />
    </section>`;
}

const TABS = [
  ['state', 'State', StatePanel],
  ['event', 'Event', EventPanel],
  ['trace', 'Trace', TracePanel],
  ['timers', 'Timers', TimersPanel],
  ['history', 'History', HistoryPanel]
];

/** The number after the tab's label. */
function tabCount(id) {
  if (id === 'history') return snapshots.value.length;
  if (id === 'timers') return current.value?.timers?.filter((t) => t.status === 'running').length ?? 0;
  return 0;
}

function SidePanel() {
  const active = tab.value;
  const Panel = (TABS.find(([id]) => id === active) ?? TABS[0])[2];
  const onKey = (e, i) => {
    const next = e.key === 'ArrowRight' ? i + 1 : e.key === 'ArrowLeft' ? i - 1 : null;
    if (next == null) return;
    e.preventDefault();
    const [id] = TABS[(next + TABS.length) % TABS.length];
    setTab(id);
    document.getElementById(`tab-${id}`)?.focus();
  };
  return html`
    <aside class="side">
      <div class="tabs" role="tablist">
        ${TABS.map(([id, label], i) => html`
          <button id=${`tab-${id}`} role="tab" aria-selected=${active === id} tabindex=${active === id ? 0 : -1}
            aria-controls="side-panel" onClick=${() => setTab(id)} onKeyDown=${(e) => onKey(e, i)}>
            ${label}${tabCount(id) ? html` <span class="count">${tabCount(id)}</span>` : ''}
          </button>`)}
      </div>
      <div class="side-body" id="side-panel" role="tabpanel"><${Panel} /></div>
    </aside>`;
}

function Timeline() {
  const list = snapshots.value;
  const index = currentIndex.value;
  const strip = useRef(null);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    if (!playing) return;
    const t = setInterval(() => {
      if (currentIndex.peek() >= snapshots.peek().length - 1) setPlaying(false);
      else step(1);
    }, 600);
    return () => clearInterval(t);
  }, [playing]);

  const pick = (e) => {
    const rect = strip.current.getBoundingClientRect();
    const i = Math.floor(((e.clientX - rect.left) / rect.width) * list.length);
    const s = list[Math.max(0, Math.min(list.length - 1, i))];
    if (s) selectSeq(s.seq);
  };
  const onPointerDown = (e) => {
    if (!list.length) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    setPlaying(false);
    pick(e);
  };
  const onPointerMove = (e) => {
    if (e.buttons & 1 && e.currentTarget.hasPointerCapture(e.pointerId)) pick(e);
  };

  const snapshot = current.value;
  const first = list[0];
  const disabled = !list.length;

  return html`
    <footer class="timeline" aria-label="History timeline">
      <div class="timeline-controls">
        <button title="First" aria-label="First snapshot" disabled=${disabled} onClick=${() => selectSeq(first.seq)}>⏮</button>
        <button title="Previous (←)" aria-label="Previous snapshot" disabled=${disabled} onClick=${() => { setPlaying(false); step(-1); }}>◀</button>
        <button title=${playing ? 'Pause' : 'Replay from here'} aria-label=${playing ? 'Pause' : 'Play'} disabled=${disabled}
          onClick=${() => setPlaying(!playing)}>${playing ? '❚❚' : '▶'}</button>
        <button title="Next (→)" aria-label="Next snapshot" disabled=${disabled} onClick=${() => { setPlaying(false); step(1); }}>▶|</button>
        <button class=${`live${follow.value ? ' on' : ''}`} title="Follow the newest snapshot (L)" disabled=${disabled}
          onClick=${() => { setPlaying(false); selectSeq(list[list.length - 1].seq); }}>● Live</button>
      </div>
      <div class="timeline-track">
        <div class="strip" ref=${strip} role="slider" tabindex="0" aria-label="Snapshot"
          aria-valuemin=${first?.seq ?? 0} aria-valuemax=${list[list.length - 1]?.seq ?? 0} aria-valuenow=${snapshot?.seq ?? 0}
          onPointerDown=${onPointerDown} onPointerMove=${onPointerMove}
          onKeyDown=${(e) => {
            if (e.key === 'ArrowLeft') { e.preventDefault(); step(-1); }
            if (e.key === 'ArrowRight') { e.preventDefault(); step(1); }
          }}>
          ${list.map((s, i) => html`
            <span key=${s.seq} class=${`tick${s.error ? ' tick-error' : ''}${i === index ? ' tick-selected' : ''}`}
              title=${`#${s.seq} ${eventLabel(s.event, s.trace)}`}></span>`)}
        </div>
        <div class="timeline-caption">
          ${snapshot
            ? html`<span><b>#${snapshot.seq}</b> ${eventLabel(snapshot.event, snapshot.trace)}</span>
                <span class="muted">${formatTime(snapshot.timestamp_ms)}</span>
                <span class="muted">${index + 1} of ${list.length} kept${selectedInstance.value ? ` (max ${selectedInstance.value.history_len})` : ''}</span>`
            : html`<span class="muted">No snapshots</span>`}
        </div>
      </div>
    </footer>`;
}

/** The draggable border between the diagram and the side panel. */
function Splitter() {
  const onPointerDown = (e) => {
    const target = e.currentTarget;
    target.setPointerCapture(e.pointerId);
    const move = (ev) => {
      const width = Math.max(260, Math.min(window.innerWidth - 320, window.innerWidth - ev.clientX));
      document.documentElement.style.setProperty('--side-width', `${width}px`);
    };
    const up = () => {
      target.removeEventListener('pointermove', move);
      target.removeEventListener('pointerup', up);
      storage.set('finny-inspect:side-width', getComputedStyle(document.documentElement).getPropertyValue('--side-width').trim());
    };
    target.addEventListener('pointermove', move);
    target.addEventListener('pointerup', up);
  };
  return html`<div class="splitter" role="separator" aria-orientation="vertical" aria-label="Resize the panel" onPointerDown=${onPointerDown}></div>`;
}

function App() {
  useEffect(() => {
    const onKey = (e) => {
      if (e.target.closest('input, select, textarea, [role="slider"], [role="tab"]')) return;
      if (e.key === 'ArrowLeft') step(-1);
      else if (e.key === 'ArrowRight') step(1);
      else if (e.key === 'l' || e.key === 'L') {
        const list = snapshots.peek();
        if (list.length) selectSeq(list[list.length - 1].seq);
      } else if (e.key === 'Escape') {
        selectedNode.value = null;
        selectedTransition.value = null;
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  return html`
    <${TopBar} />
    <main class="main">
      <${DiagramView} />
      <${Splitter} />
      <${SidePanel} />
    </main>
    <${Timeline} />`;
}

const savedWidth = storage.get('finny-inspect:side-width', null);
if (savedWidth) document.documentElement.style.setProperty('--side-width', savedWidth);

render(html`<${App} />`, document.getElementById('app'));
connect();
