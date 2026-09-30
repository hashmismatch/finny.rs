# Vendored frontend libraries

The frontend has no build step: the browser loads these files as they are, mapped by the import
map in `index.html`. They are embedded into the binary. The `sourceMappingURL` comments were
removed.

| File | Package | Version | License | Source |
|------|---------|---------|---------|--------|
| `preact.module.js` | preact | 10.29.8 | MIT | `https://cdn.jsdelivr.net/npm/preact@10.29.8/dist/preact.module.js` |
| `hooks.module.js` | preact/hooks | 10.29.8 | MIT | `https://cdn.jsdelivr.net/npm/preact@10.29.8/hooks/dist/hooks.module.js` |
| `htm.module.js` | htm | 3.1.1 | Apache-2.0 | `https://cdn.jsdelivr.net/npm/htm@3.1.1/dist/htm.module.js` |
| `signals-core.module.js` | @preact/signals-core | 1.14.4 | MIT | `https://cdn.jsdelivr.net/npm/@preact/signals-core@1.14.4/dist/signals-core.module.js` |
| `signals.module.js` | @preact/signals | 2.11.3 | MIT | `https://cdn.jsdelivr.net/npm/@preact/signals@2.11.3/dist/signals.module.js` |
| `cytoscape.esm.min.mjs` | cytoscape | 3.34.3 | MIT | `https://cdn.jsdelivr.net/npm/cytoscape@3.34.3/dist/cytoscape.esm.min.mjs` |
| `elk.bundled.js` | elkjs | 0.12.0 | EPL-2.0 | `https://cdn.jsdelivr.net/npm/elkjs@0.12.0/lib/elk.bundled.js` |

To update a library, download the new version from the same path and update this table.
