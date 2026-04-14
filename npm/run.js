#!/usr/bin/env node

const path = require('path');
const { spawnSync } = require('child_process');

const isWin = process.platform === 'win32';
const bin = path.join(__dirname, 'bin', isWin ? 'mdigitalcn.exe' : 'mdigitalcn');

const result = spawnSync(bin, process.argv.slice(2), { stdio: 'inherit' });

if (result.error) {
  if (result.error.code === 'ENOENT') {
    console.error('mdigitalcn binary not found. Try reinstalling: npm install -g @mdigitalcn/cli');
  } else {
    console.error(result.error.message);
  }
  process.exit(1);
}

process.exit(result.status ?? 0);
