#!/usr/bin/env node

const path = require('path');
const fs = require('fs');
const { spawnSync } = require('child_process');

const isWin = process.platform === 'win32';
const bin = path.join(__dirname, 'bin', isWin ? 'mdigitalcn.exe' : 'mdigitalcn');

if (!fs.existsSync(bin)) {
  const install = spawnSync(process.execPath, [path.join(__dirname, 'install.js')], { stdio: 'inherit' });
  if (install.status !== 0 || !fs.existsSync(bin)) {
    console.error('mdigitalcn binary not found. Try reinstalling: npm install -g @mdigitalcn/cli');
    process.exit(1);
  }
}

const result = spawnSync(bin, process.argv.slice(2), { stdio: 'inherit' });

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
