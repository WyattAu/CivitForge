import { dirname, join } from 'path';
import { fileURLToPath } from 'url';
import { mkdirSync } from 'fs';

const __dirname = dirname(fileURLToPath(import.meta.url));

export const BASE_URL = process.env.CIVITFORGE_URL || 'http://localhost:9091';
export const API_URL = `${BASE_URL}/api/v1`;
export const BRAIN_URL = process.env.CIVIT_BRAIN_URL || 'http://localhost:8082';
export const RUNNER_URL = process.env.CIVIT_RUNNER_URL || 'http://localhost:8088';
export const VFS_URL = process.env.CIVIT_VFS_URL || 'http://localhost:9090';

export const TIMEOUT = parseInt(process.env.E2E_TIMEOUT || '15000', 10);
export const ACTION_TIMEOUT = parseInt(process.env.E2E_ACTION_TIMEOUT || '5000', 10);
export const HEALTH_TIMEOUT = parseInt(process.env.E2E_HEALTH_TIMEOUT || '30000', 10);

export const HEADED = process.argv.includes('--headed');
export const DEBUG = process.argv.includes('--debug');
export const SKIP_CLEANUP = process.argv.includes('--skip-cleanup');
export const KEEP_TESTDATA = process.argv.includes('--keep-testdata');

export const TEST_USER = {
  email: process.env.E2E_USER_EMAIL || `e2e-${Date.now()}@test.example.com`,
  username: process.env.E2E_USER_USERNAME || `e2etest-${Date.now() % 100000}`,
  display_name: 'E2E Test User',
  password: process.env.E2E_USER_PASSWORD || `E2eTest${Date.now()}!`,
};

export const ADMIN_USER = {
  username: process.env.E2E_ADMIN_USERNAME || 'admin',
  password: process.env.E2E_ADMIN_PASSWORD || 'admin',
};

export const SCREENSHOTS_DIR = join(__dirname, 'screenshots');
export const REPORTS_DIR = join(__dirname, 'reports');

mkdirSync(SCREENSHOTS_DIR, { recursive: true });
mkdirSync(REPORTS_DIR, { recursive: true });
