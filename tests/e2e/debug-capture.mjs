export class ErrorCapture {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.networkFailures = [];
    this.pageErrors = [];
    this.responses = [];
    this.timings = [];
    this.performanceMetrics = [];
    this._attached = false;
  }

  attachToPage(page) {
    if (this._attached) {
      this._detach(page);
    }

    page.on('console', (msg) => {
      const entry = {
        timestamp: new Date().toISOString(),
        type: msg.type(),
        text: msg.text(),
        args: msg.args().map((a) => a.toString()),
        url: page.url(),
      };
      if (msg.type() === 'error') {
        this.errors.push(entry);
      } else if (msg.type() === 'warning') {
        this.warnings.push(entry);
      }
    });

    page.on('pageerror', (err) => {
      this.pageErrors.push({
        timestamp: new Date().toISOString(),
        message: err.message,
        stack: err.stack,
        url: page.url(),
      });
    });

    page.on('requestfailed', (req) => {
      this.networkFailures.push({
        timestamp: new Date().toISOString(),
        url: req.url(),
        method: req.method(),
        failure: req.failure()?.errorText || 'unknown',
      });
    });

    page.on('response', (resp) => {
      const timing = {
        timestamp: new Date().toISOString(),
        url: resp.url(),
        method: resp.request().method(),
        status: resp.status(),
        statusText: resp.statusText(),
        startMs: null,
        endMs: null,
        durationMs: null,
      };
      try {
        const headers = resp.headers();
        timing.startMs = parseFloat(headers['x-response-time']) || null;
        timing.endMs = Date.now();
      } catch {
        // ignore header parse failures
      }
      this.responses.push(timing);
      this.timings.push(timing);

      if (resp.status() >= 400) {
        this.errors.push({
          timestamp: new Date().toISOString(),
          source: 'http_error',
          message: `${resp.status()} ${resp.statusText()} ${resp.url()}`,
          url: page.url(),
        });
      }
    });

    page.on('metrics', (metric) => {
      this.performanceMetrics.push({
        timestamp: new Date().toISOString(),
        name: metric.name,
        value: metric.value,
      });
    });

    this._attached = true;
  }

  _detach(page) {
    page.removeAllListeners('console');
    page.removeAllListeners('pageerror');
    page.removeAllListeners('requestfailed');
    page.removeAllListeners('response');
    page.removeAllListeners('metrics');
  }

  async captureMemory(page) {
    const metrics = await page.metrics();
    return {
      timestamp: new Date().toISOString(),
      jsHeapUsedMB: parseFloat(metrics.JSHeapUsedSize) / (1024 * 1024),
      jsHeapTotalMB: parseFloat(metrics.JSHeapTotalSize) / (1024 * 1024),
    };
  }

  snapshot() {
    return {
      errors: [...this.errors],
      warnings: [...this.warnings],
      networkFailures: [...this.networkFailures],
      pageErrors: [...this.pageErrors],
      responses: [...this.responses],
      timings: [...this.timings],
      performanceMetrics: [...this.performanceMetrics],
    };
  }

  reset() {
    this.errors = [];
    this.warnings = [];
    this.networkFailures = [];
    this.pageErrors = [];
    this.responses = [];
    this.timings = [];
    this.performanceMetrics = [];
  }

  hasErrors() {
    return (
      this.errors.length > 0 ||
      this.pageErrors.length > 0 ||
      this.networkFailures.length > 0
    );
  }

  toJSON() {
    return {
      errors: this.errors,
      warnings: this.warnings,
      networkFailures: this.networkFailures,
      pageErrors: this.pageErrors,
      responses: this.responses,
      timings: this.timings,
      performanceMetrics: this.performanceMetrics,
    };
  }

  summary() {
    const lines = [];
    lines.push('=== Error Capture Summary ===');
    lines.push(`  Console Errors:      ${this.errors.length}`);
    lines.push(`  Console Warnings:    ${this.warnings.length}`);
    lines.push(`  Page Errors (JS):    ${this.pageErrors.length}`);
    lines.push(`  Network Failures:    ${this.networkFailures.length}`);
    lines.push(`  HTTP 4xx/5xx:        ${this.responses.length}`);
    lines.push(`  Network Timings:     ${this.timings.length}`);
    lines.push(`  Performance Metrics: ${this.performanceMetrics.length}`);

    if (this.errors.length > 0) {
      lines.push('\n--- Console Errors ---');
      for (const e of this.errors) {
        lines.push(`  [${e.timestamp}] ${e.url}`);
        lines.push(`    ${e.text}`);
      }
    }

    if (this.pageErrors.length > 0) {
      lines.push('\n--- Uncaught JS Errors ---');
      for (const e of this.pageErrors) {
        lines.push(`  [${e.timestamp}] ${e.url}`);
        lines.push(`    ${e.message}`);
      }
    }

    if (this.networkFailures.length > 0) {
      lines.push('\n--- Network Failures ---');
      for (const n of this.networkFailures) {
        lines.push(`  [${n.timestamp}] ${n.method} ${n.url}`);
        lines.push(`    ${n.failure}`);
      }
    }

    if (this.responses.length > 0) {
      lines.push('\n--- HTTP Error Responses ---');
      for (const r of this.responses) {
        lines.push(`  [${r.timestamp}] ${r.status} ${r.statusText} ${r.url}`);
      }
    }

    if (this.timings.length > 0) {
      lines.push('\n--- Network Timings ---');
      for (const t of this.timings) {
        const dur = t.durationMs != null ? `${t.durationMs.toFixed(0)}ms` : 'n/a';
        lines.push(`  ${t.method} ${t.status} ${t.url} (${dur})`);
      }
    }

    return lines.join('\n');
  }
}
