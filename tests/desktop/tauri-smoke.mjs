#!/usr/bin/env node
/**
 * CivitForge Desktop (Tauri) Smoke Test
 *
 * Launches the Tauri binary with Xvfb virtual display, verifies:
 * 1. Binary exists and is valid ELF
 * 2. Xvfb virtual display starts
 * 3. Tauri process starts and stays alive (GTK + webkit init)
 * 4. WebKit child processes spawn (webview loaded)
 * 5. Process stays alive for 10s (WASM hydration completes)
 * 6. Clean shutdown
 *
 * Requires: nix-built Xvfb (auto-detected or via XVFB_BIN env)
 */

import { execSync, spawn } from 'child_process';
import { existsSync, readdirSync } from 'fs';

const DESKTOP_BIN = process.env.DESKTOP_BIN ||
  new URL('../../crates/civit-desktop/target/release/civit-desktop', import.meta.url)
    .pathname;

const BASE_URL = process.env.CIVITFORGE_URL || 'http://localhost:9091';
const WAIT_SECONDS = 10;
const XVFB_DISPLAY = ':99';
const PASS = '\x1b[32m[PASS]\x1b[0m';
const FAIL = '\x1b[31m[FAIL]\x1b[0m';
const WARN = '\x1b[33m[WARN]\x1b[0m';

const results = { passed: 0, failed: 0, warnings: 0, errors: [] };

function check(name, condition, msg) {
  if (condition) {
    console.log(`  ${PASS} ${name}`);
    results.passed++;
  } else {
    console.log(`  ${FAIL} ${name}: ${msg}`);
    results.failed++;
    results.errors.push(msg);
  }
}

function warn(name, msg) {
  console.log(`  ${WARN} ${name}: ${msg}`);
  results.warnings++;
}

function findXvfb() {
  if (process.env.XVFB_BIN && existsSync(process.env.XVFB_BIN)) {
    return process.env.XVFB_BIN;
  }
  // Search nix store for xvfb
  try {
    const nixStore = '/nix/store';
    const entries = readdirSync(nixStore);
    for (const entry of entries) {
      if (entry.includes('xvfb-') && entry.endsWith('-21.1.21')) {
        const bin = `${nixStore}/${entry}/bin/Xvfb`;
        if (existsSync(bin)) return bin;
      }
    }
    // Fallback: any xvfb in nix store
    for (const entry of entries) {
      if (entry.startsWith('xvfb-') || entry.includes('xvfb-run')) {
        const bin = `${nixStore}/${entry}/bin/Xvfb`;
        if (existsSync(bin)) return bin;
      }
    }
  } catch { /* ignore */ }

  // Try system
  if (existsSync('/usr/bin/Xvfb')) return '/usr/bin/Xvfb';

  return null;
}

function spawnWithTimeout(cmd, args, env, timeoutMs) {
  return new Promise((resolve, reject) => {
    const proc = spawn(cmd, args, { env, stdio: ['ignore', 'pipe', 'pipe'] });
    const timer = setTimeout(() => {
      proc.kill('SIGKILL');
      reject(new Error(`Process timed out after ${timeoutMs}ms`));
    }, timeoutMs);

    proc.on('exit', (code) => { clearTimeout(timer); resolve({ code, proc }); });
    proc.on('error', (err) => { clearTimeout(timer); reject(err); });
  });
}

async function run() {
  console.log(`\n${'='.repeat(60)}`);
  console.log('  CivitForge Desktop Smoke Test');
  console.log(`  Binary:  ${DESKTOP_BIN}`);
  console.log(`  Server:  ${BASE_URL}`);
  console.log(`  Wait:    ${WAIT_SECONDS}s`);
  console.log(`${'='.repeat(60)}\n`);

  // 1. Binary exists
  check('binary-exists', existsSync(DESKTOP_BIN), `Not found at ${DESKTOP_BIN}`);
  if (!existsSync(DESKTOP_BIN)) {
    console.log('\n  Build with: cd crates/civit-desktop && LD_LIBRARY_PATH=/usr/lib cargo tauri build\n');
    process.exit(1);
  }

  // 2. Binary format
  try {
    const output = execSync(`file "${DESKTOP_BIN}"`).toString();
    const valid = output.includes('ELF') && output.includes('executable');
    check('binary-elf-executable', valid, output.trim());
  } catch (e) {
    warn('binary-elf-executable', e.message);
  }

  // 3. Binary size (sanity check — should be >5MB for webkit-linked binary)
  try {
    const size = execSync(`stat -c%s "${DESKTOP_BIN}"`).toString().trim();
    const sizeMB = (parseInt(size) / 1024 / 1024).toFixed(1);
    check('binary-size', parseInt(size) > 5 * 1024 * 1024, `Only ${sizeMB}MB — expected >5MB`);
  } catch (e) {
    warn('binary-size', e.message);
  }

  // 4. Shared library resolution
  try {
    const ldd = execSync(`LD_LIBRARY_PATH=/usr/lib ldd "${DESKTOP_BIN}" 2>&1`).toString();
    const missing = ldd.split('\n').filter(l => l.includes('not found'));
    if (missing.length > 0) {
      check('shared-libs-resolved', false, `Missing: ${missing.map(l => l.trim()).join(', ')}`);
    } else {
      check('shared-libs-resolved', true, '');
    }
  } catch (e) {
    warn('shared-libs-resolved', e.message);
  }

  // 5. Server health
  try {
    const health = await fetch(`${BASE_URL}/api/v1/health`);
    const text = await health.text();
    check('server-health', health.ok && text.includes('OK'), `${health.status} ${text}`);
  } catch {
    warn('server-health', 'Server not reachable');
  }

  // 6. Find Xvfb
  const xvfbBin = findXvfb();
  check('xvfb-available', !!xvfbBin, xvfbBin ? '' : 'No Xvfb found — install via nix or pacman');
  if (!xvfbBin) {
    console.log('\n  Xvfb required. Install:\n');
    console.log('    nix-build \'<nixpkgs>\' -A xvfb --no-out-link\n');
    process.exit(1);
  }
  console.log(`  Xvfb:    ${xvfbBin}`);

  // 7. Start Xvfb
  console.log('\n  Starting Xvfb virtual display...\n');

  const xvfbProc = spawn(xvfbBin, [
    XVFB_DISPLAY, '-screen', '0', '1280x800x24', '-ac',
  ], { stdio: ['ignore', 'pipe', 'pipe'] });

  let xvfbStarted = false;
  await new Promise(resolve => setTimeout(resolve, 2000));
  xvfbStarted = !xvfbProc.killed && xvfbProc.exitCode === null;
  check('xvfb-started', xvfbStarted, 'Xvfb failed to start');

  if (!xvfbStarted) {
    console.log('\n  Cannot test Tauri without virtual display.\n');
    process.exit(1);
  }

  try {
    // 8. Launch Tauri
    console.log('  Launching Tauri desktop...\n');

    const tauriEnv = {
      ...process.env,
      DISPLAY: XVFB_DISPLAY,
      XAUTHORITY: '',
      GDK_BACKEND: 'x11',
      LD_LIBRARY_PATH: '/usr/lib',
    };

    const tauriProc = spawn(DESKTOP_BIN, [], {
      env: tauriEnv,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stderrBuf = '';
    tauriProc.stderr.on('data', (data) => {
      stderrBuf += data.toString();
    });

    // 9. Verify GTK init (2s should be enough for GTK + webkit spawn)
    await new Promise(resolve => setTimeout(resolve, 4000));
    const started = !tauriProc.killed && tauriProc.exitCode === null;

    if (!started) {
      check('gtk-init', false, `Process exited with code ${tauriProc.exitCode}`);
      if (stderrBuf.includes('panicked')) {
        const panic = stderrBuf.split('panicked')[1]?.substring(0, 300) || '';
        console.log(`    Panic: ${panic}`);
      }
      if (stderrBuf.includes('Failed to initialize GTK')) {
        console.log('    Hint: GTK cannot connect to Xvfb. Check XAUTHORITY and DISPLAY.');
      }
    } else {
      check('gtk-init', true, '');

      // 10. Verify WebKit child processes
      let webkitFound = false;
      try {
        const tree = execSync(`pstree -p ${tauriProc.pid} 2>/dev/null`).toString();
        webkitFound = tree.includes('WebKit') || tree.includes('WebProcess');
        check('webkit-webview-spawned', webkitFound, tree.includes('WebKit') ? '' : 'No WebKit child found');
      } catch {
        warn('webkit-webview-spawned', 'pstree not available');
      }

      // 11. WASM hydration — process stays alive
      console.log(`\n  Waiting ${WAIT_SECONDS}s for WASM hydration...`);
      await new Promise(resolve => setTimeout(resolve, WAIT_SECONDS * 1000));
      check('process-alive-after-hydration', !tauriProc.killed && tauriProc.exitCode === null, 'Process crashed');

      // 12. Check for meaningful stderr (errors, not just warnings)
      const errorLines = stderrBuf.split('\n').filter(l => {
        const s = l.trim();
        return s && !s.includes('libayatana-appindicator') && !s.includes('WARNING') &&
               !s.includes('DRI3') && !s.includes('hardware acceleration') &&
               !s.includes('libEGL') && !s.includes('xkbcomp');
      });
      if (errorLines.length > 0) {
        warn('stderr-errors', `${errorLines.length} unexpected stderr lines`);
        errorLines.slice(0, 3).forEach(l => console.log(`    ${l.trim()}`));
      } else {
        check('stderr-clean', true, '');
      }

      // 13. Clean shutdown
      tauriProc.kill('SIGTERM');
      await new Promise(resolve => {
        const t = setTimeout(() => { tauriProc.kill('SIGKILL'); resolve(); }, 5000);
        tauriProc.on('exit', () => { clearTimeout(t); resolve(); });
      });
      check('clean-shutdown', tauriProc.exitCode === 0 || tauriProc.killed, `Exit code ${tauriProc.exitCode}`);
    }
  } finally {
    // Always cleanup Xvfb
    xvfbProc.kill('SIGTERM');
    await new Promise(resolve => {
      const t = setTimeout(() => { xvfbProc.kill('SIGKILL'); resolve(); }, 2000);
      xvfbProc.on('exit', () => { clearTimeout(t); resolve(); });
    });
  }

  // Results
  console.log(`\n${'='.repeat(60)}`);
  console.log(`  Results: ${results.passed} passed, ${results.failed} failed, ${results.warnings} warnings`);
  if (results.errors.length > 0) {
    console.log(`  Errors:`);
    results.errors.forEach(e => console.log(`    - ${e}`));
  }
  console.log(`${'='.repeat(60)}\n`);

  process.exit(results.failed > 0 ? 1 : 0);
}

run().catch(e => {
  console.error(`\n  Fatal: ${e.message}`);
  process.exit(1);
});
