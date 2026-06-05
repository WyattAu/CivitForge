import { mkdirSync, writeFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

export class DebugCapture {
  constructor(options = {}) {
    this.errors = [];
    this.maxErrors = options.maxErrors || 1000;
    this.screenshots = [];
    this.networkErrors = [];
    this.consoleMessages = [];
    this.performanceMetrics = [];
    this.domErrors = [];
    this.timing = {};
    this.screenshotDir = options.screenshotDir || join(__dirname, 'gui-screenshots');
    this.reportDir = options.reportDir || join(__dirname, 'gui-reports');
    this._cdpSession = null;
    this._attached = false;
  }

  async attachCDP(page) {
    try {
      const context = page.context();
      this._cdpSession = await context.newCDPSession(page);
      await this._cdpSession.send('Runtime.enable');
      await this._cdpSession.send('Log.enable');
      this._cdpSession.on('Log.entryAdded', (params) => {
        const entry = {
          timestamp: new Date().toISOString(),
          source: 'cdp-log',
          level: params.entry.level,
          text: params.entry.text,
          url: page.url(),
        };
        this.consoleMessages.push(entry);
        if (params.entry.level === 'error') {
          this.addError(entry);
        }
      });
    } catch {
      // CDP not available, skip deep debugging
    }
  }

  async captureConsole(page) {
    page.on('console', (msg) => {
      const entry = {
        timestamp: new Date().toISOString(),
        source: 'console',
        type: msg.type(),
        text: msg.text(),
        url: page.url(),
      };
      this.consoleMessages.push(entry);
      if (msg.type() === 'error') {
        this.addError({
          timestamp: entry.timestamp,
          page: page.url(),
          selector: 'console',
          error: msg.text(),
          screenshot: null,
        });
      }
    });
  }

  async captureNetwork(page) {
    page.on('response', (resp) => {
      const status = resp.status();
      if (status >= 400) {
        const entry = {
          timestamp: new Date().toISOString(),
          source: 'network',
          url: resp.url(),
          method: resp.request().method(),
          status,
          statusText: resp.statusText(),
        };
        this.networkErrors.push(entry);
        this.addError({
          timestamp: entry.timestamp,
          page: page.url(),
          selector: `network:${resp.url()}`,
          error: `${status} ${resp.statusText()}`,
          screenshot: null,
        });
      }
    });

    page.on('requestfailed', (req) => {
      const entry = {
        timestamp: new Date().toISOString(),
        source: 'network-failure',
        url: req.url(),
        method: req.method(),
        failure: req.failure()?.errorText || 'unknown',
      };
      this.networkErrors.push(entry);
      this.addError({
        timestamp: entry.timestamp,
        page: page.url(),
        selector: `network-failure:${req.url()}`,
        error: entry.failure,
        screenshot: null,
      });
    });
  }

  async captureExceptions(page) {
    page.on('pageerror', (err) => {
      this.domErrors.push({
        timestamp: new Date().toISOString(),
        source: 'uncaught-exception',
        message: err.message,
        stack: err.stack,
        url: page.url(),
      });
      this.addError({
        timestamp: new Date().toISOString(),
        page: page.url(),
        selector: 'pageerror',
        error: err.message,
        screenshot: null,
      });
    });

    try {
      await page.evaluate(() => {
        window.addEventListener('unhandledrejection', (event) => {
          console.error('[UnhandledRejection]', event.reason);
        });
      });
    } catch {
      // ignore
    }
  }

  async captureDOMErrors(page) {
    try {
      await page.evaluate(() => {
        const observer = new MutationObserver((mutations) => {
          for (const mutation of mutations) {
            for (const node of mutation.addedNodes) {
              if (node.nodeType === 1) {
                const violation = node.getAttribute('aria-invalid');
                if (violation === 'true') {
                  console.error('[Accessibility] Element added with aria-invalid="true":', node.tagName, node.id || '');
                }
                if (node.tagName === 'LABEL' && !node.getAttribute('for')) {
                  const input = node.querySelector('input, textarea, select');
                  if (input && !input.id) {
                    console.warn('[Accessibility] Label without matching input id:', node.textContent?.substring(0, 50));
                  }
                }
              }
            }
          }
        });
        observer.observe(document.body, { childList: true, subtree: true });
      });
    } catch {
      // ignore
    }
  }

  addError(entry) {
    if (this.errors.length < this.maxErrors) {
      this.errors.push(entry);
    }
  }

  async screenshot(page, name) {
    const filename = `${name.replace(/[^a-z0-9]/gi, '_')}-${Date.now()}.png`;
    try {
      const fullPath = join(this.screenshotDir, filename);
      await page.screenshot({ path: fullPath, fullPage: true });
      this.screenshots.push(filename);
      // Link the last error to this screenshot
      for (let i = this.errors.length - 1; i >= 0; i--) {
        if (!this.errors[i].screenshot) {
          this.errors[i].screenshot = filename;
          break;
        }
      }
      return filename;
    } catch {
      return null;
    }
  }

  startTimer(name) {
    this.timing[name] = { start: performance.now(), end: null, durationMs: null };
  }

  endTimer(name) {
    if (this.timing[name]) {
      this.timing[name].end = performance.now();
      this.timing[name].durationMs = this.timing[name].end - this.timing[name].start;
    }
  }

  getTimer(name) {
    return this.timing[name]?.durationMs || null;
  }

  async attachAll(page) {
    if (this._attached) {
      return;
    }
    mkdirSync(this.screenshotDir, { recursive: true });
    mkdirSync(this.reportDir, { recursive: true });
    await this.captureConsole(page);
    await this.captureNetwork(page);
    await this.captureExceptions(page);
    await this.captureDOMErrors(page);
    await this.attachCDP(page);
    this._attached = true;
  }

  snapshot() {
    return {
      errors: [...this.errors],
      screenshots: [...this.screenshots],
      networkErrors: [...this.networkErrors],
      consoleMessages: [...this.consoleMessages],
      performanceMetrics: [...this.performanceMetrics],
      domErrors: [...this.domErrors],
      timing: { ...this.timing },
    };
  }

  reset() {
    this.errors = [];
    this.screenshots = [];
    this.networkErrors = [];
    this.consoleMessages = [];
    this.performanceMetrics = [];
    this.domErrors = [];
    // Keep timing
  }

  getReport() {
    return {
      errors: this.errors,
      screenshots: this.screenshots,
      networkErrors: this.networkErrors,
      consoleMessages: this.consoleMessages,
      performanceMetrics: this.performanceMetrics,
      domErrors: this.domErrors,
      timing: this.timing,
      summary: {
        totalErrors: this.errors.length,
        totalScreenshots: this.screenshots.length,
        totalNetworkErrors: this.networkErrors.length,
        totalConsoleMessages: this.consoleMessages.length,
        totalDOMErrors: this.domErrors.length,
      },
    };
  }

  async saveReport(filename) {
    const report = this.getReport();
    const fullPath = join(this.reportDir, filename || `debug-report-${Date.now()}.json`);
    writeFileSync(fullPath, JSON.stringify(report, null, 2));
    return fullPath;
  }

  summary() {
    const lines = [];
    lines.push('=== Debug Capture Summary ===');
    lines.push(`  Errors:              ${this.errors.length}`);
    lines.push(`  Network Errors:      ${this.networkErrors.length}`);
    lines.push(`  Console Messages:    ${this.consoleMessages.length}`);
    lines.push(`  DOM Errors:          ${this.domErrors.length}`);
    lines.push(`  Screenshots Taken:   ${this.screenshots.length}`);

    if (this.errors.length > 0) {
      lines.push('\n--- Errors ---');
      for (const e of this.errors) {
        lines.push(`  [${e.timestamp}] ${e.page}`);
        lines.push(`    ${e.selector}: ${e.error}`);
        if (e.screenshot) lines.push(`    screenshot: ${e.screenshot}`);
      }
    }

    if (this.networkErrors.length > 0) {
      lines.push('\n--- Network Errors ---');
      for (const n of this.networkErrors) {
        lines.push(`  [${n.timestamp}] ${n.status} ${n.statusText} ${n.method} ${n.url}`);
      }
    }

    if (this.timing && Object.keys(this.timing).length > 0) {
      lines.push('\n--- Timings ---');
      for (const [name, t] of Object.entries(this.timing)) {
        const dur = t.durationMs != null ? `${t.durationMs.toFixed(0)}ms` : 'incomplete';
        lines.push(`  ${name}: ${dur}`);
      }
    }

    return lines.join('\n');
  }
}
