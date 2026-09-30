// The statechart: the FSM's description laid out with ELK and drawn with Cytoscape, in the
// style of PlantUML's state diagrams.

import cytoscape from 'cytoscape';
import { formatMs, pathKey, storage, timerDue, timerKey } from './util.js';

const FONT_FAMILY = 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif';
const STATE_FONT = `12px ${FONT_FAMILY}`;
const EDGE_FONT = `11px ${FONT_FAMILY}`;
const LINE_HEIGHT = 16;

// PlantUML's classic skin
const COLORS = {
  fill: '#FEFECE',
  border: '#A80036',
  compositeFill: '#FFFFF4',
  text: '#1b1b1b',
  edge: '#A80036',
  active: '#E8590C',
  activeFill: '#FFE7A3',
  rejected: '#C92A2A',
  selected: '#1C7ED6',
  timer: '#0C8599'
};

/** The status of a timer, after its name in the state's box. Empty when it's not running. */
export function timerSuffix(status, now) {
  if (!status) return '';
  switch (status.status) {
    case 'running': {
      const left = timerDue(status) - now;
      return left > 0 ? `▶ ${formatMs(left)}` : '▶ due';
    }
    case 'expired': return '✓ fired';
    case 'failed': return '✕ failed';
    case 'disabled': return '– disabled';
    default: return '';
  }
}

const timerLine = (id, suffix) => (suffix ? `⏱ ${id}  ${suffix}` : `⏱ ${id}`);

let measureContext = null;
function measure(text, font) {
  if (!measureContext) measureContext = document.createElement('canvas').getContext('2d');
  measureContext.font = font;
  return measureContext.measureText(text).width;
}

function transitionLabel(t) {
  if (t.event.kind === 'Start') return '';
  let label = t.event.kind === 'Stop' ? 'Stop' : t.event.id;
  if (t.has_guard) label += ' [guard]';
  if (t.has_action) label += ' / action';
  return label;
}

/**
 * The label of a simple state, with PlantUML's compartment for the timers and the internal
 * transitions. The width fits `reserved`, the widest lines that the label can have.
 */
function stateLabel(id, lines, reserved = lines) {
  if (!lines.length) return { label: id, width: Math.max(70, measure(id, STATE_FONT) + 28), height: 34, divider: '' };
  const width = Math.max(measure(id, STATE_FONT), ...reserved.map((l) => measure(l, STATE_FONT)));
  let divider = '';
  while (measure(divider, STATE_FONT) < width) divider += '─';
  return {
    label: [id, divider, ...lines].join('\n'),
    width: Math.max(70, width + 28),
    height: (lines.length + 2) * LINE_HEIGHT + 10,
    divider
  };
}

/**
 * Builds the elements of the diagram and the ELK graph for their layout.
 * `byTransition`: transition type name -> element id; `byState`: `${pathKey}|${stateId}` -> node id;
 * `byTimer`: `${pathKey}|${timerId}` -> node id.
 */
export function buildModel(info) {
  const elements = [];
  const byTransition = new Map();
  const byState = new Map();
  const byTimer = new Map();
  const elkNodes = new Map();
  const elkRoot = { id: 'root', children: [], edges: [] };

  function elkNode(id, parentId, props) {
    const node = { id, children: [], ...props };
    elkNodes.set(id, node);
    (parentId ? elkNodes.get(parentId) : elkRoot).children.push(node);
    return node;
  }

  function addNode(data, elk) {
    elements.push({ group: 'nodes', data });
    elkNode(data.id, data.parent, elk);
  }

  function addEdge(id, source, target, label, data) {
    elements.push({ group: 'edges', data: { id, source, target, label, mx: 0, my: 0, ...data } });
    const labels = label ? [{ text: label, width: measure(label, EDGE_FONT) + 8, height: 16 }] : [];
    elkRoot.edges.push({ id, sources: [source], targets: [target], labels });
  }

  function addMachine(machine, path, parentId, valuePrefix) {
    const key = pathKey(path);
    const multiRegion = machine.regions.length > 1;

    for (const region of machine.regions) {
      const regionKey = `${key}:${region.region_id}`;
      let container = parentId;
      if (multiRegion) {
        container = `region:${regionKey}`;
        addNode({ id: container, parent: parentId, kind: 'region', label: `region ${region.region_id}` },
          { layoutOptions: { 'elk.padding': '[top=26,left=14,bottom=14,right=14]' } });
      }

      const initial = `initial:${regionKey}`;
      addNode({ id: initial, parent: container, kind: 'initial', label: '' },
        { width: 16, height: 16, layoutOptions: { 'elk.layered.layering.layerConstraint': 'FIRST' } });
      let final = null;

      const internals = new Map();
      for (const t of region.transitions) {
        if (t.kind.kind === 'Internal') {
          const lines = internals.get(t.kind.state) ?? [];
          lines.push(`${transitionLabel(t)} (internal)`);
          internals.set(t.kind.state, lines);
        }
      }

      for (const s of region.states) {
        const id = `state:${key}|${s.id}`;
        byState.set(`${key}|${s.id}`, id);
        const valuePath = [...valuePrefix, 'states', s.storage_field];

        if (s.sub_machine) {
          addNode({ id, parent: container, kind: 'machine', label: s.id, stateId: s.id, typeName: s.type_name, valuePath },
            { layoutOptions: { 'elk.padding': '[top=34,left=16,bottom=16,right=16]' } });
          addMachine(s.sub_machine, [...path, s.type_name], id, valuePath);
        } else {
          const timerIds = s.timers.map((t) => t.id);
          const internalLines = internals.get(s.id) ?? [];
          const lines = [...timerIds.map((t) => timerLine(t)), ...internalLines];
          // room for the widest status of the timers
          const reserved = [...timerIds.flatMap((t) => ['▶ 00.0s', '✕ failed', '– disabled'].map((sfx) => timerLine(t, sfx))), ...internalLines];
          const { label, width, height, divider } = stateLabel(s.id, lines, reserved);
          addNode({ id, parent: container, kind: 'state', label, width, height, stateId: s.id, typeName: s.type_name, valuePath,
            timerIds, internalLines, divider }, { width, height });
          for (const t of timerIds) byTimer.set(timerKey(path, t), id);
        }
      }

      const stateNode = (stateId) => byState.get(`${key}|${stateId}`);

      for (const t of region.transitions) {
        const edgeId = `edge:${t.type_name}`;
        const data = { typeName: t.type_name, eventLabel: t.event.kind === 'Event' ? t.event.id : t.event.kind };
        switch (t.kind.kind) {
          case 'Normal': {
            const source = t.kind.from == null ? initial : stateNode(t.kind.from);
            let target;
            if (t.kind.to == null) {
              if (!final) {
                final = `final:${regionKey}`;
                addNode({ id: final, parent: container, kind: 'final', label: '' }, { width: 20, height: 20 });
              }
              target = final;
            } else {
              target = stateNode(t.kind.to);
            }
            if (!source || !target) break;
            addEdge(edgeId, source, target, transitionLabel(t), data);
            byTransition.set(t.type_name, edgeId);
            break;
          }
          case 'SelfTransition': {
            const node = stateNode(t.kind.state);
            if (!node) break;
            addEdge(edgeId, node, node, transitionLabel(t), data);
            byTransition.set(t.type_name, edgeId);
            break;
          }
          case 'Internal': {
            const node = stateNode(t.kind.state);
            if (node) byTransition.set(t.type_name, node);
            break;
          }
        }
      }
    }
  }

  addMachine(info, [], null, []);

  // Leaf nodes need a size for ELK, compound nodes get theirs from the children.
  for (const node of elkNodes.values()) {
    if (!node.children.length) delete node.children;
  }

  return { elements, elkRoot, byTransition, byState, byTimer };
}

const ELK_OPTIONS = {
  'elk.algorithm': 'layered',
  'elk.direction': 'DOWN',
  'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
  'elk.layered.spacing.nodeNodeBetweenLayers': '64',
  'elk.layered.spacing.edgeNodeBetweenLayers': '24',
  'elk.spacing.edgeNode': '24',
  'elk.layered.considerModelOrder.strategy': 'NODES_AND_EDGES',
  'elk.spacing.nodeNode': '36',
  'elk.spacing.edgeLabel': '4',
  'elk.spacing.componentComponent': '48',
  'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
  'elk.edgeLabels.placement': 'CENTER',
  'elk.edgeRouting': 'ORTHOGONAL',
  'elk.padding': '[top=24,left=24,bottom=24,right=24]',
  // all the coordinates are absolute
  'elk.json.shapeCoords': 'ROOT',
  'elk.json.edgeCoords': 'ROOT'
};

/**
 * The layout by ELK: the centers of the leaf nodes, and the routes of the edges with the
 * positions of their labels.
 */
async function elkLayout(model) {
  const elk = new window.ELK();
  const graph = structuredClone(model.elkRoot);
  graph.layoutOptions = ELK_OPTIONS;
  const result = await elk.layout(graph);

  const positions = {};
  const routes = {};
  const walk = (node) => {
    // ELK moves the edges into the containers of their states
    for (const edge of node.edges ?? []) {
      const sections = edge.sections ?? [];
      if (!sections.length) continue;
      const first = sections[0];
      const last = sections[sections.length - 1];
      const label = edge.labels?.[0];
      routes[edge.id] = {
        start: first.startPoint,
        end: last.endPoint,
        bends: sections.flatMap((s, i) => (i > 0 ? [s.startPoint] : []).concat(s.bendPoints ?? [])),
        label: label && label.x != null ? { x: label.x + label.width / 2, y: label.y + label.height / 2 } : null
      };
    }
    if (node.children?.length) {
      for (const child of node.children) walk(child);
    } else if (node.id !== 'root') {
      positions[node.id] = { x: node.x + node.width / 2, y: node.y + node.height / 2 };
    }
  };
  walk(result);
  return { positions, routes };
}

const ROUTE_STYLE = ['curve-style', 'segment-distances', 'segment-weights', 'segment-radii', 'edge-distances', 'source-endpoint', 'target-endpoint'];

function stylesheet() {
  return [
    {
      selector: 'node',
      style: {
        'font-family': FONT_FAMILY,
        'font-size': 12,
        color: COLORS.text,
        label: 'data(label)',
        'text-wrap': 'wrap',
        'text-valign': 'center',
        'text-halign': 'center',
        'border-color': COLORS.border,
        'border-width': 1.4,
        'background-color': COLORS.fill,
        'underlay-color': COLORS.active,
        'underlay-padding': 0,
        'underlay-opacity': 0,
        'underlay-shape': 'round-rectangle',
      }
    },
    {
      selector: 'node[kind = "state"]',
      style: {
        shape: 'round-rectangle',
        width: 'data(width)',
        height: 'data(height)',
        'line-height': 1.33
      }
    },
    {
      selector: 'node[kind = "machine"]',
      style: {
        shape: 'round-rectangle',
        'background-color': COLORS.compositeFill,
        'text-valign': 'top',
        'text-margin-y': 22,
        'font-weight': 600,
        padding: 26
      }
    },
    {
      selector: 'node[kind = "region"]',
      style: {
        shape: 'rectangle',
        'background-opacity': 0,
        'border-style': 'dashed',
        'border-width': 1,
        'border-color': '#C4A3AE',
        'text-valign': 'top',
        'text-halign': 'center',
        'text-margin-y': 16,
        'font-size': 10,
        color: '#8C6F78',
        padding: 18
      }
    },
    {
      selector: 'node[kind = "initial"]',
      style: { shape: 'ellipse', width: 14, height: 14, 'background-color': '#222', 'border-width': 0 }
    },
    {
      selector: 'node[kind = "final"]',
      style: { shape: 'ellipse', width: 18, height: 18, 'background-color': '#222', 'border-width': 3, 'border-color': '#fff', 'outline-width': 1.4, 'outline-color': '#222' }
    },
    {
      selector: 'node.active',
      style: {
        'background-color': COLORS.activeFill,
        'border-color': COLORS.active,
        'border-width': 2.6,
        'underlay-opacity': 0.18,
        'underlay-padding': 6
      }
    },
    {
      selector: 'node[kind = "machine"].active',
      style: { 'background-color': '#FFF8E1' }
    },
    {
      selector: 'node.timer-running',
      style: { 'outline-width': 2, 'outline-color': COLORS.timer, 'outline-offset': 3, 'outline-style': 'dashed' }
    },
    {
      selector: 'node.internal-taken',
      style: { 'underlay-opacity': 0.35, 'underlay-padding': 9 }
    },
    {
      selector: 'node:selected',
      style: { 'border-color': COLORS.selected, 'border-width': 2.6 }
    },
    {
      selector: 'edge',
      style: {
        width: 1.3,
        'curve-style': 'bezier',
        'line-color': COLORS.edge,
        'target-arrow-color': COLORS.edge,
        'target-arrow-shape': 'triangle',
        'arrow-scale': 0.9,
        label: 'data(label)',
        'font-family': FONT_FAMILY,
        'font-size': 11,
        color: '#5c1030',
        'text-background-color': '#ffffff',
        'text-background-opacity': 0.85,
        'text-background-padding': 2,
        'text-background-shape': 'round-rectangle',
        'text-margin-x': 'data(mx)',
        'text-margin-y': 'data(my)',
        'loop-direction': '-45deg',
        'loop-sweep': '-70deg',
        'control-point-step-size': 48,
      }
    },
    {
      selector: 'edge.taken',
      style: { width: 3.2, 'line-color': COLORS.active, 'target-arrow-color': COLORS.active, color: '#8a2f00', 'font-weight': 600, 'z-index': 10 }
    },
    {
      selector: 'edge.pulse',
      style: { width: 6 }
    },
    {
      selector: 'edge.rejected',
      style: { 'line-style': 'dashed', 'line-color': COLORS.rejected, 'target-arrow-color': COLORS.rejected }
    },
    {
      selector: 'edge:selected',
      style: { 'line-color': COLORS.selected, 'target-arrow-color': COLORS.selected, width: 3 }
    }
  ];
}

function layoutKey(info) {
  return `finny-inspect:layout:${info.type_name}`;
}

/** Identifies the structure, stale saved positions are ignored. */
function signature(model) {
  let hash = 0;
  const text = model.elements.map((e) => e.data.id).sort().join('\n');
  for (let i = 0; i < text.length; i++) hash = (hash * 31 + text.charCodeAt(i)) | 0;
  return hash;
}

export class Diagram {
  constructor(container, info, { onSelectNode, onSelectEdge }) {
    container.classList.remove('laid-out');
    this.info = info;
    this.model = buildModel(info);
    this.signature = signature(this.model);
    this.lastSeq = null;
    this.cy = cytoscape({
      container,
      elements: this.model.elements,
      style: stylesheet(),
      layout: { name: 'preset' },
      minZoom: 0.15,
      maxZoom: 4,
      boxSelectionEnabled: false,
      autoungrabify: false
    });

    // only the states and the pseudo-states are dragged, the containers follow their children
    this.cy.nodes('[kind = "region"]').ungrabify();

    this.cy.on('tap', 'node[kind = "state"], node[kind = "machine"]', (e) => {
      const d = e.target.data();
      onSelectNode({ nodeId: d.id, label: d.stateId, typeName: d.typeName, valuePath: d.valuePath, isMachine: d.kind === 'machine' });
    });
    this.cy.on('tap', 'edge', (e) => {
      const d = e.target.data();
      onSelectEdge({ typeName: d.typeName, label: d.label || d.eventLabel });
    });
    this.cy.on('tap', (e) => {
      if (e.target === this.cy) {
        onSelectNode(null);
        onSelectEdge(null);
      }
    });
    // A dragged state keeps the routes of the edges within it, the others become curves.
    this.cy.on('grab', 'node', (e) => {
      const moved = e.target.union(e.target.descendants());
      // the containers are resized, their edges move too
      const affected = moved.union(e.target.ancestors());
      this.cy.batch(() => {
        affected.connectedEdges().forEach((edge) => {
          if (edge.data('routed') && !(moved.contains(edge.source()) && moved.contains(edge.target()))) {
            edge.removeStyle(ROUTE_STYLE.join(' '));
            edge.data({ routed: false, mx: 0, my: 0 });
          }
        });
      });
    });
    this.cy.on('dragfree', 'node', () => {
      this.relaxLabels();
      this.saveLayout();
    });

    this.ready = this.layout(false);
  }

  async layout(reset) {
    const saved = reset ? null : storage.get(layoutKey(this.info), null);
    let layout = null;
    if (saved && saved.signature === this.signature) {
      layout = saved;
    } else {
      try {
        layout = await elkLayout(this.model);
      } catch (e) {
        console.error('ELK layout failed, falling back to a grid', e);
      }
      if (reset) storage.set(layoutKey(this.info), undefined);
    }

    const cy = this.cy;
    if (cy.destroyed()) return;
    cy.batch(() => {
      cy.edges().forEach((edge) => {
        edge.removeStyle(ROUTE_STYLE.join(' '));
        edge.data({ routed: false, mx: 0, my: 0 });
      });
      if (layout) {
        for (const [id, p] of Object.entries(layout.positions)) {
          const node = cy.getElementById(id);
          if (node.nonempty() && node.isChildless()) node.position(p);
        }
      }
    });

    if (!layout) {
      cy.layout({ name: 'grid' }).run();
    } else if (layout.routes) {
      const routed = this.applyRoutes(layout.routes);
      // the compound nodes are updated with the next render, the edges' midpoints depend on them
      await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      if (cy.destroyed()) return;
      this.placeLabels(routed);
    } else if (layout.edges) {
      // a saved layout
      cy.batch(() => {
        for (const [id, e] of Object.entries(layout.edges)) {
          const edge = cy.getElementById(id);
          if (edge.empty()) continue;
          edge.style(e.style);
          edge.data({ routed: true, mx: e.mx, my: e.my });
        }
      });
    }

    this.relaxLabels();
    cy.fit(undefined, 30);
    cy.container().classList.add('laid-out');
  }

  /** Draws the edges along ELK's orthogonal routes, with the labels where ELK placed them. */
  applyRoutes(routes) {
    const cy = this.cy;
    const px = (v) => `${v.toFixed(1)}px`;
    const routed = [];

    // The position of a compound node isn't updated until the next render, it's the center of
    // its children.
    const center = (node) => {
      if (!node.isParent()) return node.position();
      const bb = node.descendants().filter((d) => d.isChildless()).boundingBox({ includeLabels: false, includeOverlays: false });
      return { x: (bb.x1 + bb.x2) / 2, y: (bb.y1 + bb.y2) / 2 };
    };

    cy.batch(() => {
      for (const [id, route] of Object.entries(routes)) {
        const edge = cy.getElementById(id);
        if (edge.empty() || edge.isLoop()) continue;
        const s = center(edge.source());
        const t = center(edge.target());
        const style = {
          'source-endpoint': `${px(route.start.x - s.x)} ${px(route.start.y - s.y)}`,
          'target-endpoint': `${px(route.end.x - t.x)} ${px(route.end.y - t.y)}`
        };

        // The bend points, relative to the line between the states' centers.
        const dx = t.x - s.x;
        const dy = t.y - s.y;
        const len2 = dx * dx + dy * dy;
        if (route.bends.length && len2 > 0) {
          const len = Math.sqrt(len2);
          style['curve-style'] = 'round-segments';
          style['edge-distances'] = 'node-position';
          style['segment-radii'] = 8;
          style['segment-weights'] = route.bends.map((p) => (((p.x - s.x) * dx + (p.y - s.y) * dy) / len2).toFixed(4)).join(' ');
          style['segment-distances'] = route.bends.map((p) => (((p.x - s.x) * -dy + (p.y - s.y) * dx) / len).toFixed(1)).join(' ');
        } else {
          style['curve-style'] = 'straight';
        }

        edge.style(style);
        edge.data({ routed: true });
        routed.push([edge, route]);
      }
    });
    return routed;
  }

  /** The labels, relative to where Cytoscape puts them: the middle of the edge. */
  placeLabels(routed) {
    this.cy.batch(() => {
      for (const [edge, route] of routed) {
        if (!route.label || !edge.data('label')) continue;
        const mid = edge.midpoint();
        edge.data({ mx: route.label.x - mid.x, my: route.label.y - mid.y });
      }
    });
  }

  /**
   * Moves the overlapping labels of the edges that aren't routed by ELK, typically the edges of
   * a dragged state, out of the way of the other labels.
   */
  relaxLabels() {
    const labeled = this.cy.edges().filter((e) => e.data('label'));
    const movable = new Set(labeled.filter((e) => !e.data('routed')).map((e) => e.id()));
    if (!movable.size) return;
    this.cy.batch(() => labeled.forEach((e) => { if (movable.has(e.id())) e.data({ mx: 0, my: 0 }); }));
    const labelBox = (e) => e.boundingBox({ includeNodes: false, includeEdges: false, includeLabels: true, includeOverlays: false });
    const gap = 3;
    const shift = (e, axis, amount) => e.data(axis, e.data(axis) + amount);

    for (let iteration = 0; iteration < 16; iteration++) {
      const boxes = labeled.map((e) => ({ e, b: labelBox(e), movable: movable.has(e.id()) }));
      let moved = false;
      for (let i = 0; i < boxes.length; i++) {
        for (let j = i + 1; j < boxes.length; j++) {
          const [p, q] = [boxes[i], boxes[j]];
          if (!p.movable && !q.movable) continue;
          const [a, b] = [p.b, q.b];
          const ox = Math.min(a.x2, b.x2) - Math.max(a.x1, b.x1) + gap;
          const oy = Math.min(a.y2, b.y2) - Math.max(a.y1, b.y1) + gap;
          if (ox <= 0 || oy <= 0) continue;

          // apart along the axis with the smaller overlap, split between the movable ones
          const [axis, overlap, delta] = ox < oy
            ? ['mx', ox, (a.x1 + a.x2 - b.x1 - b.x2) / 2]
            : ['my', oy, (a.y1 + a.y2 - b.y1 - b.y2) / 2];
          const direction = delta >= 0 ? 1 : -1;
          const share = p.movable && q.movable ? overlap / 2 : overlap;
          if (p.movable) shift(p.e, axis, direction * share);
          if (q.movable) shift(q.e, axis, -direction * share);
          moved = true;
        }
      }
      if (!moved) break;
    }
  }

  saveLayout() {
    const positions = {};
    this.cy.nodes().filter((n) => n.isChildless()).forEach((n) => { positions[n.id()] = { ...n.position() }; });
    const edges = {};
    this.cy.edges().filter((e) => e.data('routed')).forEach((e) => {
      const style = {};
      for (const name of ROUTE_STYLE) {
        const v = e.style(name);
        if (v != null) style[name] = v;
      }
      edges[e.id()] = { style, mx: e.data('mx'), my: e.data('my') };
    });
    storage.set(layoutKey(this.info), { signature: this.signature, positions, edges });
  }

  /** Highlights the current states and the transitions of the snapshot's event. */
  show(snapshot) {
    const cy = this.cy;
    // class -> the ids of the elements that have it
    const wanted = { active: new Set(), taken: new Set(), rejected: new Set(), 'internal-taken': new Set() };

    if (snapshot) {
      for (const a of snapshot.active) {
        const key = pathKey(a.path);
        for (const state of a.states) {
          if (state == null) continue;
          const id = this.model.byState.get(`${key}|${state}`);
          if (id) wanted.active.add(id);
        }
      }

      for (const entry of snapshot.trace) {
        const id = this.model.byTransition.get(entry.type_name);
        if (!id) continue;
        const isEdge = id.startsWith('edge:');
        if (entry.kind === 'transition') wanted[isEdge ? 'taken' : 'internal-taken'].add(id);
        else if (entry.kind === 'guard' && !entry.result && isEdge) wanted.rejected.add(id);
      }
    }

    // Only the changes are applied, removing and adding the same class in one batch can leave
    // stale styles in the rendering.
    cy.batch(() => {
      for (const [cls, ids] of Object.entries(wanted)) {
        cy.elements(`.${cls}`).forEach((el) => { if (!ids.has(el.id())) el.removeClass(cls); });
        for (const id of ids) {
          const el = cy.getElementById(id);
          if (!el.hasClass(cls)) el.addClass(cls);
        }
      }
    });

    // animate the newly taken transitions
    if (snapshot && snapshot.seq !== this.lastSeq) {
      for (const id of wanted.taken) cy.getElementById(id).flashClass('pulse', 350);
    }
    this.lastSeq = snapshot?.seq ?? null;
  }

  /**
   * The status of the timers after their names in the states' boxes, at the time `now`. The
   * states with a running timer are outlined.
   */
  showTimers(snapshot, now) {
    const cy = this.cy;
    const byNode = new Map();
    for (const t of snapshot?.timers ?? []) {
      const id = this.model.byTimer.get(timerKey(t.path, t.timer));
      if (!id) continue;
      if (!byNode.has(id)) byNode.set(id, new Map());
      byNode.get(id).set(t.timer, t);
    }

    cy.batch(() => {
      cy.nodes('[kind = "state"]').forEach((node) => {
        const d = node.data();
        if (!d.timerIds?.length) return;
        const statuses = byNode.get(node.id());
        const lines = d.timerIds.map((t) => timerLine(t, timerSuffix(statuses?.get(t), now)));
        const label = [d.stateId, d.divider, ...lines, ...d.internalLines].join('\n');
        if (label !== d.label) node.data('label', label);

        const running = !!statuses && [...statuses.values()].some((t) => t.status === 'running');
        if (running !== node.hasClass('timer-running')) node.toggleClass('timer-running', running);
      });
    });
  }

  select(nodeId, transitionTypeName) {
    const cy = this.cy;
    cy.$(':selected').unselect();
    if (nodeId) cy.getElementById(nodeId).select();
    if (transitionTypeName) {
      const id = this.model.byTransition.get(transitionTypeName);
      if (id) cy.getElementById(id).select();
    }
  }

  zoom(factor) {
    const cy = this.cy;
    cy.zoom({ level: cy.zoom() * factor, renderedPosition: { x: cy.width() / 2, y: cy.height() / 2 } });
  }

  fit() {
    this.cy.animate({ fit: { padding: 30 } }, { duration: 200 });
  }

  png() {
    return this.cy.png({ full: true, scale: 2, bg: '#ffffff', output: 'blob' });
  }

  destroy() {
    this.cy.destroy();
  }
}
