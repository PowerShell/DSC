// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

'use strict';

const express = require('express');
const WebSocket = require('ws');
const { spawn, execFileSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const VISUALIZER_VERSION = '1.0.0';
const PORT = parseInt(process.env.PORT || '3000', 10);

// Locate the dsc binary: prefer PATH, fall back to repo build output
function findDscBinary() {
  // Check environment variable override
  if (process.env.DSC_PATH) {
    return process.env.DSC_PATH;
  }
  // Try PATH first
  try {
    execFileSync('dsc', ['--version'], { timeout: 2000, stdio: 'pipe' });
    return 'dsc';
  } catch {
    // Not in PATH, try build output relative to this file
    const repoRoot = path.resolve(__dirname, '..', '..');
    const candidates = [
      path.join(repoRoot, 'bin', 'dsc'),
      path.join(repoRoot, 'bin', 'debug', 'dsc'),
      path.join(repoRoot, 'target', 'debug', 'dsc'),
      path.join(repoRoot, 'target', 'release', 'dsc'),
    ];
    for (const candidate of candidates) {
      if (fs.existsSync(candidate)) {
        return candidate;
      }
    }
    return 'dsc'; // Let it fail naturally with a clear error
  }
}

const DSC_BIN = findDscBinary();
console.log(`Using dsc binary: ${DSC_BIN}`);

const app = express();
app.use(express.static(path.join(__dirname, 'public')));

// Expose visualizer version to the frontend
app.get('/api/version', (_req, res) => {
  res.json({ visualizerVersion: VISUALIZER_VERSION, dscBin: DSC_BIN });
});

const server = app.listen(PORT, '127.0.0.1', () => {
  console.log(`DSC Visualizer running at http://localhost:${PORT}`);
});

const wss = new WebSocket.Server({ server });

/** @type {import('child_process').ChildProcess | null} */
let dscProcess = null;
let stdoutBuffer = '';

function broadcast(message) {
  const json = JSON.stringify(message);
  for (const client of wss.clients) {
    if (client.readyState === WebSocket.OPEN) {
      client.send(json);
    }
  }
}

function spawnDscProcess() {
  if (dscProcess) {
    try { dscProcess.kill(); } catch { /* ignore */ }
    dscProcess = null;
    stdoutBuffer = '';
  }

  let proc;
  try {
    proc = spawn(DSC_BIN, ['mcp'], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: process.env,
    });
  } catch (err) {
    broadcast({ type: 'error', message: `Failed to spawn dsc: ${err.message}` });
    return;
  }

  proc.stdout.on('data', (chunk) => {
    stdoutBuffer += chunk.toString('utf8');
    let newlineIdx;
    while ((newlineIdx = stdoutBuffer.indexOf('\n')) !== -1) {
      const line = stdoutBuffer.slice(0, newlineIdx).trim();
      stdoutBuffer = stdoutBuffer.slice(newlineIdx + 1);
      if (!line) continue;
      try {
        const parsed = JSON.parse(line);
        broadcast({ type: 'mcp', data: parsed });
      } catch {
        // Non-JSON output from dsc (e.g. trace/debug lines) → forward as stderr
        broadcast({ type: 'stderr', data: line });
      }
    }
  });

  proc.stderr.on('data', (chunk) => {
    broadcast({ type: 'stderr', data: chunk.toString('utf8') });
  });

  proc.on('error', (err) => {
    broadcast({ type: 'error', message: `dsc process error: ${err.message}` });
    dscProcess = null;
    stdoutBuffer = '';
  });

  proc.on('close', (code) => {
    broadcast({ type: 'exit', code });
    dscProcess = null;
    stdoutBuffer = '';
  });

  dscProcess = proc;
}

wss.on('connection', (ws) => {
  // Start the dsc mcp process if not already running
  if (!dscProcess) {
    spawnDscProcess();
  }

  ws.on('message', (raw) => {
    try {
      const msg = JSON.parse(raw.toString('utf8'));
      if (msg.type === 'mcp') {
        if (dscProcess && dscProcess.stdin.writable) {
          dscProcess.stdin.write(JSON.stringify(msg.data) + '\n');
        } else {
          ws.send(JSON.stringify({ type: 'error', message: 'dsc process is not running' }));
        }
      } else if (msg.type === 'restart') {
        spawnDscProcess();
      }
    } catch (err) {
      console.error('Invalid WebSocket message:', err.message);
    }
  });

  ws.on('close', () => {
    // Terminate dsc when the last client disconnects
    if (wss.clients.size === 0 && dscProcess) {
      try { dscProcess.kill(); } catch { /* ignore */ }
      dscProcess = null;
      stdoutBuffer = '';
    }
  });
});

function cleanup() {
  if (dscProcess) {
    try { dscProcess.kill(); } catch { /* ignore */ }
  }
  process.exit(0);
}

process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);
