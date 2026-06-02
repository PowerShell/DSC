// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

'use strict';

/* ═══════════════════════════════════════════════════════════════════════
   DSC Web Visualizer – Frontend Application
   ═══════════════════════════════════════════════════════════════════════ */

const VISUALIZER_VERSION = '1.0.0';
const WS_URL = `ws://${location.host}`;
const RECONNECT_DELAY_MS = 2000;
const REQUEST_TIMEOUT_MS = 30_000;

// ── State ────────────────────────────────────────────────────────────────────

const state = {
  /** @type {WebSocket|null} */
  ws: null,
  mcpInitialized: false,
  dscVersion: null,
  nextId: 1,
  /** @type {Map<number, {resolve: Function, reject: Function, timer: number}>} */
  pending: new Map(),

  /** @type {object|null} Parsed configuration document */
  config: null,
  /** Raw text loaded from file (YAML or JSON) */
  configText: '',
  /** Name of the loaded file */
  configFilename: null,
  /** Whether the config has been modified */
  dirty: false,

  /** Currently selected node id (type/name) */
  selectedNodeId: null,

  /** vis-network instance */
  network: null,

  /** Panel visibility */
  treeVisible: true,
  propsVisible: true,
  consoleVisible: true,

  /** Panel sizes (px) */
  treeWidth: 240,
  propsWidth: 300,
  consoleHeight: 180,
};

// ── Utilities ─────────────────────────────────────────────────────────────────

function el(id) { return document.getElementById(id); }

function qs(selector, root = document) { return root.querySelector(selector); }

function sanitizeText(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function deepClone(obj) { return JSON.parse(JSON.stringify(obj)); }

/** Parse a DSC resourceId expression: [resourceId('Type','Name')] */
function parseResourceId(expr) {
  const m = String(expr).match(/resourceId\(['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]\)/);
  return m ? { type: m[1], name: m[2] } : null;
}

function resourceNodeId(type, name) { return `${type}/${name}`; }

// ── Toast Notifications ───────────────────────────────────────────────────────

let toastContainer;
function initToasts() {
  toastContainer = document.createElement('div');
  toastContainer.id = 'toast-container';
  document.body.appendChild(toastContainer);
}

/**
 * @param {string} message
 * @param {'info'|'success'|'warn'|'error'} type
 * @param {number} [duration=3000]
 */
function showToast(message, type = 'info', duration = 3000) {
  const t = document.createElement('div');
  t.className = `toast ${type}`;
  t.textContent = message;
  t.setAttribute('role', 'alert');
  t.setAttribute('aria-live', 'assertive');
  toastContainer.appendChild(t);
  setTimeout(() => t.remove(), duration);
}

// ── Status Bar ────────────────────────────────────────────────────────────────

const STATUS_ICONS = { ready: '🟢', loading: '🔵', running: '🟡', error: '🔴', disconnected: '⚪' };

/**
 * @param {string} text
 * @param {keyof STATUS_ICONS} [icon]
 */
function setStatus(text, icon = 'ready') {
  el('status-text').textContent = text;
  el('status-icon').textContent = STATUS_ICONS[icon] || '⚪';
}

function setFilename(name) {
  state.configFilename = name;
  el('file-name').textContent = name || '';
  el('status-sep').style.display = name ? '' : 'none';
}

// ── Console Output ────────────────────────────────────────────────────────────

/**
 * @param {'stdout'|'trace'|'errors'} tab
 * @param {string} text
 * @param {string} [lineClass]
 */
function appendConsole(tab, text, lineClass = '') {
  const panel = el(`tab-${tab}`);
  if (!panel) return;
  const div = document.createElement('div');
  div.className = `console-line ${lineClass}`;
  div.textContent = text;
  panel.appendChild(div);
  panel.scrollTop = panel.scrollHeight;
}

function clearConsole() {
  ['stdout', 'trace', 'errors'].forEach(tab => {
    const p = el(`tab-${tab}`);
    if (p) p.innerHTML = '';
  });
}

function initConsoleTabs() {
  el('console-tabs').addEventListener('click', (e) => {
    const btn = e.target.closest('.console-tab');
    if (!btn) return;
    const tab = btn.dataset.tab;

    document.querySelectorAll('.console-tab').forEach(b => {
      b.classList.remove('active');
      b.setAttribute('aria-selected', 'false');
    });
    document.querySelectorAll('.console-content').forEach(p => {
      p.classList.remove('active');
      p.hidden = true;
    });

    btn.classList.add('active');
    btn.setAttribute('aria-selected', 'true');
    const panel = el(`tab-${tab}`);
    if (panel) { panel.classList.add('active'); panel.hidden = false; }
  });

  el('btn-clear-console').addEventListener('click', clearConsole);
}

// ── WebSocket / MCP Client ────────────────────────────────────────────────────

function connectWebSocket() {
  if (state.ws && state.ws.readyState === WebSocket.OPEN) return;

  setStatus('Connecting to DSC…', 'disconnected');
  const ws = new WebSocket(WS_URL);
  state.ws = ws;

  ws.addEventListener('open', () => {
    setStatus('Connected — initializing MCP…', 'loading');
    initMcp();
  });

  ws.addEventListener('message', (event) => {
    try {
      const envelope = JSON.parse(event.data);
      handleEnvelope(envelope);
    } catch (err) {
      console.error('WS message parse error:', err);
    }
  });

  ws.addEventListener('close', () => {
    state.mcpInitialized = false;
    state.dscVersion = null;
    setStatus('Disconnected from DSC', 'disconnected');
    el('dsc-version').textContent = 'Disconnected';
    // Attempt reconnect
    setTimeout(connectWebSocket, RECONNECT_DELAY_MS);
  });

  ws.addEventListener('error', () => {
    setStatus('Connection error', 'error');
  });
}

/** Handle an envelope message from the backend */
function handleEnvelope(envelope) {
  switch (envelope.type) {
    case 'mcp':
      handleMcpMessage(envelope.data);
      break;
    case 'stderr':
      appendConsole('trace', envelope.data);
      break;
    case 'error':
      appendConsole('errors', `[server] ${envelope.message}`, 'error');
      setStatus(`Error: ${envelope.message}`, 'error');
      showToast(envelope.message, 'error');
      break;
    case 'exit':
      appendConsole('trace', `[dsc] process exited with code ${envelope.code}`, 'warn');
      state.mcpInitialized = false;
      break;
    default:
      break;
  }
}

/** Handle a JSON-RPC message from the dsc MCP server */
function handleMcpMessage(msg) {
  if (msg.id != null && state.pending.has(msg.id)) {
    const { resolve, reject, timer } = state.pending.get(msg.id);
    state.pending.delete(msg.id);
    clearTimeout(timer);
    if (msg.error) {
      reject(new Error(msg.error.message || JSON.stringify(msg.error)));
    } else {
      resolve(msg.result);
    }
    return;
  }
  // Notification or unmatched message
  if (msg.method) {
    appendConsole('trace', `[mcp] notification: ${msg.method}`);
  }
}

/** Send a JSON-RPC request and return a Promise */
function sendMcpRequest(method, params) {
  return new Promise((resolve, reject) => {
    if (!state.ws || state.ws.readyState !== WebSocket.OPEN) {
      reject(new Error('WebSocket not connected'));
      return;
    }
    const id = state.nextId++;
    const timer = setTimeout(() => {
      if (state.pending.has(id)) {
        state.pending.delete(id);
        reject(new Error(`Request "${method}" timed out`));
      }
    }, REQUEST_TIMEOUT_MS);

    state.pending.set(id, { resolve, reject, timer });
    state.ws.send(JSON.stringify({
      type: 'mcp',
      data: { jsonrpc: '2.0', id, method, params },
    }));
  });
}

/** Send a JSON-RPC notification (no response expected) */
function sendMcpNotification(method, params) {
  if (!state.ws || state.ws.readyState !== WebSocket.OPEN) return;
  state.ws.send(JSON.stringify({
    type: 'mcp',
    data: { jsonrpc: '2.0', method, params },
  }));
}

/** Initialize the MCP session */
async function initMcp() {
  try {
    const result = await sendMcpRequest('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'DSC Web Visualizer', version: VISUALIZER_VERSION },
    });

    sendMcpNotification('notifications/initialized', {});
    state.mcpInitialized = true;

    const versionStr = result?.serverInfo?.version || result?.serverInfo?.name || '';
    state.dscVersion = versionStr;
    el('dsc-version').textContent = versionStr ? `DSC ${versionStr}` : 'DSC (connected)';
    setStatus('Ready', 'ready');
    setButtonStates();
  } catch (err) {
    setStatus(`MCP init failed: ${err.message}`, 'error');
    appendConsole('errors', `MCP init error: ${err.message}`, 'error');
    showToast(`Failed to initialize DSC: ${err.message}`, 'error', 6000);
  }
}

/**
 * Call an MCP tool and return the parsed result.
 * @template T
 * @param {string} toolName
 * @param {object} args
 * @returns {Promise<T>}
 */
async function callTool(toolName, args) {
  const result = await sendMcpRequest('tools/call', { name: toolName, arguments: args });
  if (result?.isError) {
    const msg = result?.content?.[0]?.text || 'Tool call failed';
    throw new Error(msg);
  }
  const text = result?.content?.[0]?.text;
  if (text == null) throw new Error('Empty tool response');
  return JSON.parse(text);
}

// ── Configuration Management ──────────────────────────────────────────────────

/**
 * Load a configuration from a File object.
 * @param {File} file
 */
async function loadConfigFile(file) {
  setStatus(`Loading ${file.name}…`, 'loading');
  try {
    const text = await file.text();
    let parsed;
    const lower = file.name.toLowerCase();
    if (lower.endsWith('.json')) {
      parsed = JSON.parse(text);
    } else {
      // Default: try YAML (covers .yaml, .yml and also valid JSON)
      parsed = jsyaml.load(text);
    }

    state.config = parsed;
    state.configText = text;
    state.dirty = false;
    setFilename(file.name);
    setStatus(`Loaded ${file.name}`, 'ready');

    renderAll();
    setButtonStates();
    appendConsole('stdout', `Loaded configuration: ${file.name}`);
  } catch (err) {
    setStatus(`Failed to load file: ${err.message}`, 'error');
    showToast(`Failed to parse ${file.name}: ${err.message}`, 'error', 6000);
    appendConsole('errors', `Load error: ${err.message}`, 'error');
  }
}

/** Save the current configuration to disk */
function saveConfigFile() {
  if (!state.config) return;
  const name = state.configFilename || 'configuration.dsc.yaml';
  const isJson = name.toLowerCase().endsWith('.json');
  const text = isJson
    ? JSON.stringify(state.config, null, 2)
    : jsyaml.dump(state.config, { lineWidth: 120, noRefs: true });

  const blob = new Blob([text], { type: 'text/plain' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = name;
  a.click();
  URL.revokeObjectURL(a.href);
  state.dirty = false;
  appendConsole('stdout', `Saved: ${name}`);
}

/** Serialize current config to YAML for sending to DSC */
function configToYaml() {
  if (!state.config) return '';
  return jsyaml.dump(state.config, { lineWidth: 120, noRefs: true });
}

/** Enable / disable toolbar buttons based on current state */
function setButtonStates() {
  const ready = state.mcpInitialized;
  const hasConfig = !!state.config;
  el('btn-save').disabled    = !hasConfig;
  el('btn-execute').disabled = !hasConfig || !ready;
}

// ── Render Pipeline ───────────────────────────────────────────────────────────

function renderAll() {
  renderTreeView();
  renderVisualization();
}

// ── Tree View ─────────────────────────────────────────────────────────────────

function renderTreeView() {
  const container = el('tree-view');
  if (!state.config) {
    container.innerHTML = '<div class="empty-state">Load a configuration file to begin.</div>';
    return;
  }

  const ul = document.createElement('ul');
  ul.className = 'tree-root';
  ul.setAttribute('role', 'tree');

  // Root config node
  const rootItem = makeTreeItem('⬡', 'Configuration', '', null, 0, false);
  ul.appendChild(rootItem);
  const rootChildren = rootItem.querySelector('.tree-children');

  // Directives (if present)
  if (state.config.directives) {
    const dirItem = makeTreeItem('⚙', 'directives', 'directives', '__directives__', 1, false);
    rootChildren.appendChild(dirItem);
  }

  // Metadata (informational, no editing)
  if (state.config.metadata) {
    const metaItem = makeTreeItem('ℹ', 'metadata', 'metadata', '__metadata__', 1, false);
    rootChildren.appendChild(metaItem);
  }

  // Parameters
  if (state.config.parameters) {
    const paramKeys = Object.keys(state.config.parameters);
    const paramItem = makeTreeItem('📝', 'parameters', `${paramKeys.length} param(s)`, '__parameters__', 1, true);
    const paramChildren = paramItem.querySelector('.tree-children');
    paramKeys.forEach(k => {
      const item = makeTreeItem('◆', k, 'parameter', `__param__${k}`, 2, false);
      paramChildren.appendChild(item);
    });
    rootChildren.appendChild(paramItem);
  }

  // Resources
  const resources = state.config.resources || [];
  const resItem = makeTreeItem('📦', 'resources', `${resources.length} resource(s)`, '__resources__', 1, true);
  resItem.querySelector('.tree-children').setAttribute('aria-label', 'Resources');
  rootChildren.appendChild(resItem);

  resources.forEach(r => {
    const nodeId = resourceNodeId(r.type, r.name);
    const icon = getResourceIcon(r.type);
    const rItem = makeTreeItem(icon, r.name, r.type, nodeId, 2, hasNestedResources(r));
    if (hasNestedResources(r)) {
      appendNestedResources(rItem.querySelector('.tree-children'), r.properties?.resources || [], 3);
    }
    resItem.querySelector('.tree-children').appendChild(rItem);
  });

  container.innerHTML = '';
  container.appendChild(ul);

  // Expand root and resources by default
  rootItem.querySelector('.tree-toggle')?.click();
  resItem.querySelector('.tree-toggle')?.click();
}

function appendNestedResources(container, resources, depth) {
  resources.forEach(r => {
    const nodeId = resourceNodeId(r.type, r.name);
    const icon = getResourceIcon(r.type);
    const item = makeTreeItem(icon, r.name, r.type, nodeId, depth, hasNestedResources(r));
    if (hasNestedResources(r)) {
      appendNestedResources(item.querySelector('.tree-children'), r.properties?.resources || [], depth + 1);
    }
    container.appendChild(item);
  });
}

function hasNestedResources(resource) {
  return Array.isArray(resource.properties?.resources) && resource.properties.resources.length > 0;
}

function getResourceIcon(type) {
  if (!type) return '📦';
  const t = type.toLowerCase();
  if (t.includes('group'))     return '📁';
  if (t.includes('assertion')) return '✅';
  if (t.includes('include'))   return '📎';
  if (t.includes('osinfo'))    return '💻';
  if (t.includes('registry'))  return '🔑';
  if (t.includes('service'))   return '⚙';
  if (t.includes('echo'))      return '🔔';
  return '📦';
}

/**
 * Build a tree list item.
 * @param {string} icon
 * @param {string} label
 * @param {string} subLabel
 * @param {string|null} nodeId
 * @param {number} depth
 * @param {boolean} expandable
 */
function makeTreeItem(icon, label, subLabel, nodeId, depth, expandable) {
  const li = document.createElement('li');
  li.className = 'tree-item';
  li.setAttribute('role', 'treeitem');

  const row = document.createElement('div');
  row.className = 'tree-node';
  row.setAttribute('tabindex', '0');
  row.style.setProperty('--depth', depth);
  if (nodeId) row.dataset.nodeId = nodeId;

  if (expandable) {
    const toggle = document.createElement('button');
    toggle.className = 'tree-toggle';
    toggle.setAttribute('aria-label', `Expand ${label}`);
    toggle.setAttribute('tabindex', '-1');
    toggle.textContent = '▶';
    row.appendChild(toggle);
  } else {
    const spacer = document.createElement('span');
    spacer.style.width = '16px';
    spacer.style.flexShrink = '0';
    row.appendChild(spacer);
  }

  const iconEl = document.createElement('span');
  iconEl.className = 'tree-icon';
  iconEl.textContent = icon;
  row.appendChild(iconEl);

  const labelEl = document.createElement('span');
  labelEl.className = 'tree-label';
  labelEl.textContent = label;
  row.appendChild(labelEl);

  if (subLabel) {
    const typeEl = document.createElement('span');
    typeEl.className = 'tree-type';
    typeEl.textContent = subLabel;
    row.appendChild(typeEl);
  }

  li.appendChild(row);

  const children = document.createElement('ul');
  children.className = 'tree-children collapsed';
  children.setAttribute('role', 'group');
  li.appendChild(children);

  // Expand / collapse
  if (expandable) {
    const toggle = row.querySelector('.tree-toggle');
    row.addEventListener('click', (e) => {
      if (e.target === toggle || e.currentTarget === row) {
        const isOpen = toggle.classList.toggle('open');
        children.classList.toggle('collapsed', !isOpen);
        toggle.setAttribute('aria-label', isOpen ? `Collapse ${label}` : `Expand ${label}`);
      }
    });
    toggle.addEventListener('click', (e) => {
      e.stopPropagation();
      const isOpen = toggle.classList.toggle('open');
      children.classList.toggle('collapsed', !isOpen);
    });
  }

  // Select node
  if (nodeId) {
    row.addEventListener('click', (e) => {
      if (!e.target.classList.contains('tree-toggle')) {
        selectNode(nodeId);
      }
    });
    row.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        selectNode(nodeId);
      }
    });
  }

  return li;
}

/** Highlight a node in the tree view */
function selectTreeNode(nodeId) {
  document.querySelectorAll('.tree-node.selected').forEach(n => n.classList.remove('selected'));
  const node = document.querySelector(`.tree-node[data-node-id="${CSS.escape(nodeId)}"]`);
  if (node) {
    node.classList.add('selected');
    node.scrollIntoView({ block: 'nearest' });
  }
}

// ── Visualization (vis-network) ───────────────────────────────────────────────

const VIS_OPTIONS = {
  layout: {
    hierarchical: {
      enabled: true,
      direction: 'LR',
      sortMethod: 'directed',
      nodeSpacing: 120,
      levelSeparation: 200,
    },
  },
  physics: { enabled: false },
  nodes: {
    shape: 'box',
    font: { size: 12, face: 'monospace', multi: true },
    borderWidth: 2,
    borderWidthSelected: 3,
    margin: { top: 10, right: 14, bottom: 10, left: 14 },
  },
  edges: {
    arrows: { to: { enabled: true, scaleFactor: 0.7 } },
    smooth: { type: 'cubicBezier', forceDirection: 'horizontal', roundness: 0.4 },
    color: { color: '#6b7280', highlight: '#0078d4', hover: '#0078d4' },
    width: 1.5,
  },
  interaction: {
    hover: true,
    tooltipDelay: 300,
    navigationButtons: true,
    keyboard: { enabled: true },
  },
  autoResize: true,
};

function getNodeColor(type) {
  if (!type) return { background: '#dbeafe', border: '#3b82f6', highlight: { background: '#bfdbfe', border: '#1d4ed8' } };
  const t = type.toLowerCase();
  if (t.includes('group') || t.includes('assertion'))
    return { background: '#dcfce7', border: '#16a34a', highlight: { background: '#bbf7d0', border: '#15803d' } };
  if (t.includes('include'))
    return { background: '#fef3c7', border: '#d97706', highlight: { background: '#fde68a', border: '#b45309' } };
  return { background: '#dbeafe', border: '#3b82f6', highlight: { background: '#bfdbfe', border: '#1d4ed8' } };
}

function renderVisualization() {
  el('viz-empty').classList.toggle('hidden', !!state.config);
  if (!state.config) return;

  const nodes = [];
  const edges = [];

  function addResources(resources, parentId = null) {
    for (const r of resources) {
      const id = resourceNodeId(r.type, r.name);
      nodes.push({
        id,
        label: `<b>${sanitizeText(r.name)}</b>\n<i>${sanitizeText(r.type)}</i>`,
        color: getNodeColor(r.type),
        title: `${r.type}\n${r.name}`,
      });

      if (parentId) {
        edges.push({ from: id, to: parentId, dashes: true, color: { color: '#9ca3af' } });
      }

      // Dependency edges from dependsOn
      if (Array.isArray(r.dependsOn)) {
        for (const dep of r.dependsOn) {
          const ref = parseResourceId(dep);
          if (ref) {
            const depId = resourceNodeId(ref.type, ref.name);
            edges.push({ from: id, to: depId });
          }
        }
      }

      // Recurse into nested resources (Group / Assertion etc.)
      if (Array.isArray(r.properties?.resources)) {
        addResources(r.properties.resources, id);
      }
    }
  }

  addResources(state.config.resources || []);

  // Add directives node if present
  if (state.config.directives) {
    nodes.push({
      id: '__directives__',
      label: '<b>directives</b>',
      color: { background: '#f3e8ff', border: '#9333ea', highlight: { background: '#e9d5ff', border: '#7e22ce' } },
      shape: 'ellipse',
    });
  }

  const container = el('visualization');

  if (state.network) {
    state.network.destroy();
    state.network = null;
  }

  const dataset = {
    nodes: new vis.DataSet(nodes),
    edges: new vis.DataSet(edges),
  };

  state.network = new vis.Network(container, dataset, VIS_OPTIONS);

  state.network.on('click', (params) => {
    if (params.nodes.length > 0) {
      selectNode(params.nodes[0]);
    }
  });

  state.network.on('doubleClick', () => {
    state.network.fit();
  });
}

// ── Node Selection ────────────────────────────────────────────────────────────

function selectNode(nodeId) {
  if (state.selectedNodeId === nodeId) return;
  state.selectedNodeId = nodeId;

  // Sync tree selection
  selectTreeNode(nodeId);

  // Sync graph selection
  if (state.network) {
    state.network.selectNodes([nodeId]);
  }

  // Show properties
  showProperties(nodeId);
}

// ── Property Editor ───────────────────────────────────────────────────────────

async function showProperties(nodeId) {
  const editor = el('property-editor');

  if (nodeId === '__directives__') {
    showDirectivesEditor();
    return;
  }

  if (nodeId === '__metadata__' || nodeId === '__parameters__' || nodeId?.startsWith('__param__')) {
    showReadonlyProperties(nodeId);
    return;
  }

  // Find the resource
  const resource = findResource(state.config?.resources || [], nodeId);
  if (!resource) {
    editor.innerHTML = '<div class="empty-state">Resource not found.</div>';
    return;
  }

  editor.innerHTML = '<div class="prop-loading">Loading schema…</div>';

  let schema = null;
  if (state.mcpInitialized) {
    try {
      const response = await callTool('show_dsc_resource', { type: resource.type });
      schema = response?.schema || null;
    } catch {
      // Schema unavailable — render without it
    }
  }

  renderResourceEditor(resource, schema);
}

function findResource(resources, nodeId) {
  for (const r of resources) {
    if (resourceNodeId(r.type, r.name) === nodeId) return r;
    if (Array.isArray(r.properties?.resources)) {
      const found = findResource(r.properties.resources, nodeId);
      if (found) return found;
    }
  }
  return null;
}

/** Render a schema-driven form for a resource */
function renderResourceEditor(resource, schema) {
  const editor = el('property-editor');
  editor.innerHTML = '';

  // Header
  const header = document.createElement('div');
  header.className = 'prop-resource-header';
  header.innerHTML = `
    <div class="prop-resource-name">${sanitizeText(resource.name)}</div>
    <div class="prop-resource-type">${sanitizeText(resource.type)}</div>
  `;
  editor.appendChild(header);

  // Build a working copy of properties
  const propsCopy = deepClone(resource.properties || {});

  // Properties section
  const section = document.createElement('div');
  section.className = 'prop-section';

  const secHeader = document.createElement('div');
  secHeader.className = 'prop-section-header open';
  secHeader.setAttribute('role', 'button');
  secHeader.setAttribute('tabindex', '0');
  secHeader.innerHTML = '<span class="section-toggle">▶</span> Properties';
  section.appendChild(secHeader);

  const secBody = document.createElement('div');
  secBody.className = 'prop-section-body';

  if (schema && schema.properties) {
    buildSchemaForm(secBody, schema, propsCopy);
  } else {
    // Fallback: render existing properties as text inputs
    buildFallbackForm(secBody, propsCopy);
  }

  secHeader.addEventListener('click', () => {
    const open = secHeader.classList.toggle('open');
    secBody.classList.toggle('collapsed', !open);
  });
  secHeader.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); secHeader.click(); }
  });

  section.appendChild(secBody);
  editor.appendChild(section);

  // Action buttons
  const actions = document.createElement('div');
  actions.className = 'prop-actions';

  const saveBtn = document.createElement('button');
  saveBtn.className = 'prop-save-btn';
  saveBtn.textContent = 'Apply';
  saveBtn.setAttribute('aria-label', 'Apply property changes');
  saveBtn.addEventListener('click', () => applyPropertyChanges(resource, secBody, propsCopy, schema));

  const cancelBtn = document.createElement('button');
  cancelBtn.className = 'prop-cancel-btn';
  cancelBtn.textContent = 'Reset';
  cancelBtn.setAttribute('aria-label', 'Reset property changes');
  cancelBtn.addEventListener('click', () => showProperties(state.selectedNodeId));

  actions.appendChild(saveBtn);
  actions.appendChild(cancelBtn);
  editor.appendChild(actions);
}

/** Collect form values and apply to the config */
function applyPropertyChanges(resource, formContainer, propsCopy, schema) {
  const collected = collectFormValues(formContainer, propsCopy, schema);
  resource.properties = collected;
  state.dirty = true;
  showToast('Properties updated', 'success', 2000);
  setStatus('Modified', 'loading');
}

/** Collect current form values into an object */
function collectFormValues(container, current, schema) {
  const result = {};
  const inputs = container.querySelectorAll('[data-prop-key]');
  inputs.forEach(input => {
    const key = input.dataset.propKey;
    let value;
    if (input.type === 'checkbox') {
      value = input.checked;
    } else if (input.dataset.propType === 'number') {
      value = input.value === '' ? undefined : Number(input.value);
    } else {
      value = input.value === '' ? undefined : input.value;
    }
    if (value !== undefined) result[key] = value;
  });
  // Merge with existing values not in the form (e.g. $schema, nested objects)
  return Object.assign({}, current, result);
}

/**
 * Build a form from a JSON schema.
 * @param {HTMLElement} container
 * @param {object} schema
 * @param {object} data Current values
 * @param {string} [prefix] Key prefix for nested properties
 */
function buildSchemaForm(container, schema, data, prefix = '') {
  const props = schema.properties || {};
  const required = new Set(schema.required || []);

  for (const [key, propSchema] of Object.entries(props)) {
    if (key === '$schema' || key === '_exist') {
      // DSC internals — skip
    }
    const fullKey = prefix ? `${prefix}.${key}` : key;
    const value = data[key];
    const isRequired = required.has(key);
    appendPropRow(container, key, fullKey, propSchema, value, isRequired);
  }
}

/** Append a single property row based on its schema */
function appendPropRow(container, key, fullKey, propSchema, value, isRequired) {
  const resolvedSchema = resolveSchema(propSchema);
  const type = getSchemaType(resolvedSchema);
  const row = document.createElement('div');
  row.className = 'prop-row';

  // Label
  const labelEl = document.createElement('label');
  labelEl.className = 'prop-label';
  labelEl.htmlFor = `prop-${CSS.escape(fullKey)}`;
  labelEl.innerHTML = sanitizeText(propSchema.title || key) +
    (isRequired ? ' <span class="prop-required-badge" aria-label="required">*</span>' : '');
  row.appendChild(labelEl);

  if (propSchema.description || resolvedSchema.description) {
    const desc = document.createElement('div');
    desc.className = 'prop-description';
    desc.textContent = propSchema.description || resolvedSchema.description;
    row.appendChild(desc);
  }

  // Input based on type
  const input = buildInput(key, fullKey, resolvedSchema, type, value);
  if (input) row.appendChild(input);

  container.appendChild(row);
}

function resolveSchema(schema) {
  // Handle nullable: {oneOf: [{type: 'null'}, <real schema>]}
  if (schema.oneOf) {
    const nonNull = schema.oneOf.find(s => s.type !== 'null');
    return nonNull || schema.oneOf[0] || schema;
  }
  if (schema.anyOf) {
    const nonNull = schema.anyOf.find(s => s.type !== 'null');
    return nonNull || schema.anyOf[0] || schema;
  }
  return schema;
}

function getSchemaType(schema) {
  if (Array.isArray(schema.type)) {
    return schema.type.find(t => t !== 'null') || schema.type[0];
  }
  return schema.type || 'string';
}

/** Build an input element for a property */
function buildInput(key, fullKey, schema, type, value) {
  const id = `prop-${CSS.escape(fullKey)}`;

  // Enum → select
  if (schema.enum) {
    const sel = document.createElement('select');
    sel.className = 'prop-select';
    sel.id = id;
    sel.dataset.propKey = key;
    const emptyOpt = document.createElement('option');
    emptyOpt.value = '';
    emptyOpt.textContent = '— select —';
    sel.appendChild(emptyOpt);
    schema.enum.forEach(opt => {
      const o = document.createElement('option');
      o.value = String(opt);
      o.textContent = String(opt);
      if (opt === value) o.selected = true;
      sel.appendChild(o);
    });
    if (value != null) sel.value = String(value);
    return sel;
  }

  if (type === 'boolean') {
    const wrapper = document.createElement('div');
    wrapper.className = 'prop-checkbox-row';
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.className = 'prop-checkbox';
    cb.id = id;
    cb.dataset.propKey = key;
    cb.checked = !!value;
    const lbl = document.createElement('label');
    lbl.htmlFor = id;
    lbl.textContent = 'Enabled';
    wrapper.appendChild(cb);
    wrapper.appendChild(lbl);
    return wrapper;
  }

  if (type === 'integer' || type === 'number') {
    const inp = document.createElement('input');
    inp.type = 'number';
    inp.className = 'prop-input';
    inp.id = id;
    inp.dataset.propKey = key;
    inp.dataset.propType = 'number';
    if (value != null) inp.value = String(value);
    if (schema.minimum != null) inp.min = schema.minimum;
    if (schema.maximum != null) inp.max = schema.maximum;
    return inp;
  }

  if (type === 'array') {
    return buildArrayInput(key, fullKey, schema, value);
  }

  if (type === 'object') {
    if (schema.properties) {
      const nested = document.createElement('div');
      nested.className = 'prop-nested';
      buildSchemaForm(nested, schema, value || {}, key);
      return nested;
    }
    // Arbitrary object — textarea
    const ta = document.createElement('textarea');
    ta.className = 'prop-textarea';
    ta.id = id;
    ta.dataset.propKey = key;
    ta.value = value != null ? JSON.stringify(value, null, 2) : '';
    return ta;
  }

  // Default: string / unknown
  const inp = document.createElement('input');
  inp.type = 'text';
  inp.className = 'prop-input';
  inp.id = id;
  inp.dataset.propKey = key;
  if (value != null) inp.value = String(value);
  if (schema.pattern) inp.pattern = schema.pattern;
  return inp;
}

function buildArrayInput(key, fullKey, schema, value) {
  const wrapper = document.createElement('div');
  wrapper.className = 'prop-array';
  wrapper.dataset.propKey = key;

  const items = Array.isArray(value) ? [...value] : [];

  function renderItems() {
    wrapper.innerHTML = '';
    items.forEach((item, i) => {
      const row = document.createElement('div');
      row.className = 'prop-array-item';

      const inp = document.createElement('input');
      inp.type = 'text';
      inp.className = 'prop-input';
      inp.value = typeof item === 'string' ? item : JSON.stringify(item);
      inp.setAttribute('aria-label', `Item ${i + 1}`);
      inp.addEventListener('change', () => { items[i] = inp.value; });

      const rm = document.createElement('button');
      rm.className = 'prop-array-remove';
      rm.textContent = '✕';
      rm.setAttribute('aria-label', `Remove item ${i + 1}`);
      rm.addEventListener('click', () => { items.splice(i, 1); renderItems(); });

      row.appendChild(inp);
      row.appendChild(rm);
      wrapper.appendChild(row);
    });

    const addBtn = document.createElement('button');
    addBtn.className = 'prop-array-add';
    addBtn.textContent = '+ Add item';
    addBtn.addEventListener('click', () => { items.push(''); renderItems(); });
    wrapper.appendChild(addBtn);
  }

  renderItems();
  return wrapper;
}

/** Fallback form when no schema is available */
function buildFallbackForm(container, data) {
  if (!data || Object.keys(data).length === 0) {
    const msg = document.createElement('div');
    msg.className = 'empty-state';
    msg.textContent = 'No properties defined.';
    container.appendChild(msg);
    return;
  }
  for (const [key, value] of Object.entries(data)) {
    if (key === '$schema') continue;
    const row = document.createElement('div');
    row.className = 'prop-row';

    const label = document.createElement('label');
    label.className = 'prop-label';
    label.htmlFor = `prop-${CSS.escape(key)}`;
    label.textContent = key;
    row.appendChild(label);

    if (typeof value === 'boolean') {
      const cb = document.createElement('input');
      cb.type = 'checkbox';
      cb.className = 'prop-checkbox';
      cb.id = `prop-${CSS.escape(key)}`;
      cb.dataset.propKey = key;
      cb.checked = value;
      row.appendChild(cb);
    } else if (typeof value === 'object' && value !== null) {
      const ta = document.createElement('textarea');
      ta.className = 'prop-textarea';
      ta.id = `prop-${CSS.escape(key)}`;
      ta.dataset.propKey = key;
      ta.value = JSON.stringify(value, null, 2);
      row.appendChild(ta);
    } else {
      const inp = document.createElement('input');
      inp.type = typeof value === 'number' ? 'number' : 'text';
      inp.className = 'prop-input';
      inp.id = `prop-${CSS.escape(key)}`;
      inp.dataset.propKey = key;
      if (typeof value === 'number') inp.dataset.propType = 'number';
      inp.value = value != null ? String(value) : '';
      row.appendChild(inp);
    }

    container.appendChild(row);
  }
}

/** Show the directives editor */
async function showDirectivesEditor() {
  const editor = el('property-editor');
  editor.innerHTML = '<div class="prop-loading">Loading configuration schema…</div>';

  let directivesSchema = null;
  if (state.mcpInitialized) {
    try {
      const response = await callTool('show_dsc_schema', { type: 'Configuration' });
      directivesSchema = response?.schema?.properties?.directives || null;
    } catch {
      // ignore
    }
  }

  editor.innerHTML = '';
  const header = document.createElement('div');
  header.className = 'prop-resource-header';
  header.innerHTML = '<div class="prop-resource-name">directives</div><div class="prop-resource-type">Configuration directives</div>';
  editor.appendChild(header);

  const data = deepClone(state.config.directives || {});
  const section = document.createElement('div');
  section.className = 'prop-section';
  const secHeader = document.createElement('div');
  secHeader.className = 'prop-section-header open';
  secHeader.innerHTML = '<span class="section-toggle">▶</span> Directives';
  section.appendChild(secHeader);

  const secBody = document.createElement('div');
  secBody.className = 'prop-section-body';

  if (directivesSchema?.properties) {
    buildSchemaForm(secBody, directivesSchema, data);
  } else {
    buildFallbackForm(secBody, data);
  }

  secHeader.addEventListener('click', () => {
    const open = secHeader.classList.toggle('open');
    secBody.classList.toggle('collapsed', !open);
  });

  section.appendChild(secBody);
  editor.appendChild(section);
}

/** Show read-only properties for non-resource nodes */
function showReadonlyProperties(nodeId) {
  const editor = el('property-editor');
  editor.innerHTML = '';

  let data = null;
  let title = '';
  if (nodeId === '__metadata__') { data = state.config?.metadata; title = 'metadata'; }
  else if (nodeId === '__parameters__') { data = state.config?.parameters; title = 'parameters'; }
  else if (nodeId?.startsWith('__param__')) {
    const key = nodeId.replace('__param__', '');
    data = state.config?.parameters?.[key];
    title = key;
  }

  const header = document.createElement('div');
  header.className = 'prop-resource-header';
  header.innerHTML = `<div class="prop-resource-name">${sanitizeText(title)}</div>`;
  editor.appendChild(header);

  const pre = document.createElement('pre');
  pre.style.cssText = 'padding:10px;font-size:11px;overflow:auto;margin:0;color:var(--text-primary)';
  pre.textContent = JSON.stringify(data, null, 2);
  editor.appendChild(pre);
}

// ── Configuration Execution ───────────────────────────────────────────────────

async function executeConfig() {
  if (!state.config || !state.mcpInitialized) return;

  const operation = el('exec-operation').value;
  const yaml = configToYaml();

  clearConsole();
  appendConsole('stdout', `Executing configuration: ${operation.toUpperCase()}`, 'info');
  setStatus(`Executing ${operation}…`, 'running');
  el('btn-execute').disabled = true;

  try {
    const result = await callTool('invoke_dsc_config', {
      operation,
      configuration: yaml,
    });

    const output = JSON.stringify(result, null, 2);
    appendConsole('stdout', output);
    appendConsole('stdout', `\n✔ Execution completed successfully.`, 'success');
    setStatus('Ready', 'ready');
    showToast(`${operation} completed`, 'success');
  } catch (err) {
    appendConsole('errors', `Execution error: ${err.message}`, 'error');
    setStatus(`Execution failed: ${err.message}`, 'error');
    showToast(`Execution failed: ${err.message}`, 'error', 6000);
    // Switch to errors tab
    el('tab-btn-errors').click();
  } finally {
    el('btn-execute').disabled = false;
    setButtonStates();
  }
}

// ── Panel Resize ──────────────────────────────────────────────────────────────

function initResizeHandles() {
  setupHorizontalDivider('divider-left',  'tree-panel',    'treeWidth',    160, 480);
  setupHorizontalDivider('divider-right', 'props-panel',   'propsWidth',   200, 560);
  setupVerticalDivider('divider-console', 'console-panel', 'consoleHeight', 40, 500);

  // Keyboard-accessible resize
  document.querySelectorAll('.h-divider, .v-divider').forEach(d => {
    d.addEventListener('keydown', (e) => {
      const isH = d.classList.contains('h-divider');
      const step = 20;
      if (isH) {
        if (e.key === 'ArrowLeft')  { e.preventDefault(); nudgeDivider(d, -step); }
        if (e.key === 'ArrowRight') { e.preventDefault(); nudgeDivider(d, +step); }
      } else {
        if (e.key === 'ArrowUp')   { e.preventDefault(); nudgeDivider(d, -step); }
        if (e.key === 'ArrowDown') { e.preventDefault(); nudgeDivider(d, +step); }
      }
    });
  });
}

function nudgeDivider(divider, delta) {
  if (divider.id === 'divider-left') {
    state.treeWidth = Math.max(160, Math.min(480, state.treeWidth + delta));
    el('tree-panel').style.width = `${state.treeWidth}px`;
  } else if (divider.id === 'divider-right') {
    state.propsWidth = Math.max(200, Math.min(560, state.propsWidth + delta));
    el('props-panel').style.width = `${state.propsWidth}px`;
  } else if (divider.id === 'divider-console') {
    state.consoleHeight = Math.max(40, Math.min(500, state.consoleHeight + delta));
    el('console-panel').style.height = `${state.consoleHeight}px`;
    document.documentElement.style.setProperty('--console-h', `${state.consoleHeight}px`);
  }
}

function setupHorizontalDivider(dividerId, panelId, stateKey, minPx, maxPx) {
  const divider = el(dividerId);
  const panel = el(panelId);
  if (!divider || !panel) return;

  let startX, startW;

  divider.addEventListener('mousedown', (e) => {
    e.preventDefault();
    startX = e.clientX;
    startW = panel.offsetWidth;
    divider.classList.add('dragging');
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';

    function onMove(e) {
      const delta = dividerId === 'divider-left' ? e.clientX - startX : startX - e.clientX;
      const newW = Math.max(minPx, Math.min(maxPx, startW + delta));
      state[stateKey] = newW;
      panel.style.width = `${newW}px`;
    }

    function onUp() {
      divider.classList.remove('dragging');
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    }

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  });
}

function setupVerticalDivider(dividerId, panelId, stateKey, minPx, maxPx) {
  const divider = el(dividerId);
  const panel = el(panelId);
  if (!divider || !panel) return;

  let startY, startH;

  divider.addEventListener('mousedown', (e) => {
    e.preventDefault();
    startY = e.clientY;
    startH = panel.offsetHeight;
    divider.classList.add('dragging');
    document.body.style.cursor = 'row-resize';
    document.body.style.userSelect = 'none';

    function onMove(e) {
      const delta = startY - e.clientY;
      const newH = Math.max(minPx, Math.min(maxPx, startH + delta));
      state[stateKey] = newH;
      panel.style.height = `${newH}px`;
      document.documentElement.style.setProperty('--console-h', `${newH}px`);
    }

    function onUp() {
      divider.classList.remove('dragging');
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    }

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  });
}

// ── Panel Toggle ──────────────────────────────────────────────────────────────

function togglePanel(panelId, btnId) {
  const panel = el(panelId);
  const btn = el(btnId);
  if (!panel || !btn) return;

  const visible = !panel.classList.contains('hidden');
  panel.classList.toggle('hidden', visible);
  btn.classList.toggle('active', !visible);
  btn.setAttribute('aria-pressed', String(!visible));

  // Also hide the adjacent divider
  const dividerMap = {
    'tree-panel':    'divider-left',
    'props-panel':   'divider-right',
    'console-panel': 'divider-console',
  };
  const divider = el(dividerMap[panelId]);
  if (divider) divider.style.display = visible ? 'none' : '';
}

// ── Panel Close Buttons ───────────────────────────────────────────────────────

function initPanelCloseButtons() {
  document.querySelectorAll('.panel-close-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.dataset.target;
      const btnMap = {
        'tree-panel':    'btn-toggle-tree',
        'props-panel':   'btn-toggle-props',
        'console-panel': 'btn-toggle-console',
      };
      togglePanel(targetId, btnMap[targetId]);
    });
  });
}

// ── Theme ─────────────────────────────────────────────────────────────────────

function initTheme() {
  const saved = localStorage.getItem('dsc-visualizer-theme') || 'light';
  applyTheme(saved);
  el('btn-theme').addEventListener('click', () => {
    const current = document.documentElement.getAttribute('data-theme');
    applyTheme(current === 'dark' ? 'light' : 'dark');
  });
}

function applyTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.setItem('dsc-visualizer-theme', theme);
  el('btn-theme').textContent = theme === 'dark' ? '☀️' : '🌙';
  el('btn-theme').setAttribute('aria-label', theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode');
  // Re-render visualization to pick up new node colors
  if (state.config && state.network) renderVisualization();
}

// ── Keyboard Shortcuts ────────────────────────────────────────────────────────

function initKeyboardShortcuts() {
  document.addEventListener('keydown', (e) => {
    const ctrl = e.ctrlKey || e.metaKey;
    if (ctrl && e.key === 'o') { e.preventDefault(); el('file-input').click(); }
    if (ctrl && e.key === 's') { e.preventDefault(); saveConfigFile(); }
    if (ctrl && e.key === 'Enter') { e.preventDefault(); if (!el('btn-execute').disabled) executeConfig(); }
  });
}

// ── Version Info ──────────────────────────────────────────────────────────────

async function loadVersionInfo() {
  try {
    const res = await fetch('/api/version');
    const data = await res.json();
    el('vis-version').textContent = `v${data.visualizerVersion}`;
  } catch {
    el('vis-version').textContent = `v${VISUALIZER_VERSION}`;
  }
}

// ── Initialization ────────────────────────────────────────────────────────────

window.addEventListener('DOMContentLoaded', async () => {
  initToasts();
  initTheme();
  initConsoleTabs();
  initResizeHandles();
  initPanelCloseButtons();
  initKeyboardShortcuts();
  await loadVersionInfo();

  // File input change
  el('file-input').addEventListener('change', (e) => {
    const file = e.target.files?.[0];
    if (file) loadConfigFile(file);
    e.target.value = '';
  });

  // Toolbar buttons
  el('btn-load').addEventListener('click', () => el('file-input').click());
  el('btn-load-splash').addEventListener('click', () => el('file-input').click());
  el('btn-save').addEventListener('click', saveConfigFile);
  el('btn-execute').addEventListener('click', executeConfig);
  el('btn-fit').addEventListener('click', () => state.network?.fit());
  el('btn-restart').addEventListener('click', () => {
    state.mcpInitialized = false;
    if (state.ws) { state.ws.close(); state.ws = null; }
    setStatus('Restarting…', 'loading');
    setTimeout(connectWebSocket, 300);
  });
  el('btn-toggle-tree').addEventListener('click',    () => togglePanel('tree-panel',    'btn-toggle-tree'));
  el('btn-toggle-props').addEventListener('click',   () => togglePanel('props-panel',   'btn-toggle-props'));
  el('btn-toggle-console').addEventListener('click', () => togglePanel('console-panel', 'btn-toggle-console'));

  // Drag-and-drop onto the viz panel
  el('viz-panel').addEventListener('dragover', (e) => { e.preventDefault(); e.dataTransfer.dropEffect = 'copy'; });
  el('viz-panel').addEventListener('drop', (e) => {
    e.preventDefault();
    const file = e.dataTransfer.files?.[0];
    if (file) loadConfigFile(file);
  });

  setButtonStates();
  connectWebSocket();
});
