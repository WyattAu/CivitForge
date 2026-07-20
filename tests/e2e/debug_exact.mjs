import { chromium } from '@playwright/test';

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
page.on('console', msg => console.log('CONSOLE:', msg.type(), msg.text()));
page.on('pageerror', err => console.log('PAGE ERROR:', err.message));

// Exact same sequence as the test
console.log('goto /');
await page.goto('/');
console.log('waitForLoadState networkidle');
await page.waitForLoadState('networkidle');
console.log('networkidle done, waiting for aside nav a...');
try {
  await page.waitForSelector('aside nav a, nav a', { timeout: 15000 });
  console.log('SUCCESS: nav links found');
} catch(e) {
  console.log('FAILED:', e.message.split('\n')[0]);
  const body = await page.content();
  console.log('HTML length:', body.length);
  console.log('Has script tag:', body.includes('civit_ui.js'));
  console.log('Has aside:', body.includes('<aside'));
  console.log('Body text:', (await page.textContent('body'))?.substring(0, 100));
}
await browser.close();
