export class ErrorCapture {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.networkFailures = [];
    this.pageErrors = [];
    this.responses = [];
    this._attached = false;
  }

  attachToPage(page) {
    if (this._attached) {
      this._detach(page);
    }

    page.on('console', (msg) => {
      const entry = {
        type: msg.type(),
        text: msg.text(),
        url: page.url(),
        timestamp: new Date().toISOString(),
      };
      if (msg.type() === 'error') {
        this.errors.push(entry);
      } else if (msg.type() === 'warning') {
        this.warnings.push(entry);
      }
    });

    page.on('pageerror', (err) => {
      this.pageErrors.push({
        message: err.message,
        stack: err.stack,
        url: page.url(),
        timestamp: new Date().toISOString(),
      });
    });

    page.on('requestfailed', (req) => {
      this.networkFailures.push({
        url: req.url(),
        method: req.method(),
        failure: req.failure()?.errorText || 'unknown',
        timestamp: new Date().toISOString(),
      });
    });

    page.on('response', (resp) => {
      const status = resp.status();
      if (status >= 400) {
        this.responses.push({
          url: resp.url(),
          status,
          statusText: resp.statusText(),
          timestamp: new Date().toISOString(),
        });
      }
    });

    this._attached = true;
  }

  _detach(page) {
    page.removeAllListeners('console');
    page.removeAllListeners('pageerror');
    page.removeAllListeners('requestfailed');
    page.removeAllListeners('response');
  }

  snapshot() {
    return {
      errors: [...this.errors],
      warnings: [...this.warnings],
      networkFailures: [...this.networkFailures],
      pageErrors: [...this.pageErrors],
      responses: [...this.responses],
    };
  }

  reset() {
    this.errors = [];
    this.warnings = [];
    this.networkFailures = [];
    this.pageErrors = [];
    this.responses = [];
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
    };
  }

  summary() {
    const lines = [];
    lines.push('=== Error Capture Summary ===');
    lines.push(`  Console Errors:     ${this.errors.length}`);
    lines.push(`  Console Warnings:   ${this.warnings.length}`);
    lines.push(`  Page Errors (JS):   ${this.pageErrors.length}`);
    lines.push(`  Network Failures:   ${this.networkFailures.length}`);
    lines.push(`  HTTP 4xx/5xx:       ${this.responses.length}`);

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

    return lines.join('\n');
  }
}
