#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const os = require('os');

const VERSION = require('./package.json').version;
const BIN_DIR = path.join(__dirname, 'bin');
const REPO = 'mdigitalcn/cli';

function getPlatformTarget() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'darwin') {
    return { target: 'universal-apple-darwin', ext: 'tar.gz', isWin: false };
  }
  if (platform === 'linux') {
    if (arch === 'arm64') {
      return { target: 'aarch64-unknown-linux-musl', ext: 'tar.gz', isWin: false };
    }
    return { target: 'x86_64-unknown-linux-musl', ext: 'tar.gz', isWin: false };
  }
  if (platform === 'win32') {
    return { target: 'x86_64-pc-windows-msvc', ext: 'zip', isWin: true };
  }
  throw new Error(`Unsupported platform: ${platform}/${arch}`);
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const follow = (resolvedUrl) => {
      https.get(resolvedUrl, (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          return follow(res.headers.location);
        }
        if (res.statusCode !== 200) {
          reject(new Error(`HTTP ${res.statusCode} downloading ${resolvedUrl}`));
          return;
        }
        const file = fs.createWriteStream(dest);
        res.pipe(file);
        file.on('finish', () => file.close(resolve));
        file.on('error', reject);
      }).on('error', reject);
    };
    follow(url);
  });
}

async function main() {
  const { target, ext, isWin } = getPlatformTarget();
  const name = `mdigitalcn-v${VERSION}-${target}`;
  const filename = `${name}.${ext}`;
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${filename}`;
  const tmpFile = path.join(os.tmpdir(), filename);
  const binName = isWin ? 'mdigitalcn.exe' : 'mdigitalcn';
  const binPath = path.join(BIN_DIR, binName);

  if (fs.existsSync(binPath)) {
    return;
  }

  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  console.log(`Downloading mdigitalcn v${VERSION} (${target})...`);

  try {
    await download(url, tmpFile);
  } catch (err) {
    throw new Error(`Download failed: ${err.message}\nURL: ${url}`);
  }

  if (isWin) {
    execSync(
      `powershell -command "Expand-Archive -Path '${tmpFile}' -DestinationPath '${BIN_DIR}' -Force"`,
      { stdio: 'inherit' }
    );
    fs.renameSync(path.join(BIN_DIR, `${name}.exe`), binPath);
  } else {
    execSync(`tar xzf "${tmpFile}" -C "${BIN_DIR}"`, { stdio: 'inherit' });
    fs.renameSync(path.join(BIN_DIR, name), binPath);
    fs.chmodSync(binPath, 0o755);
  }

  try { fs.unlinkSync(tmpFile); } catch (_) {}

  console.log('mdigitalcn installed successfully.');
}

main().catch((err) => {
  console.error(`\nFailed to install mdigitalcn: ${err.message}`);
  process.exit(1);
});
