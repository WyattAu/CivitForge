import { chromium } from '@playwright/test';

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
page.on('console', msg => {
  if (msg.type() === 'error') console.log('CONSOLE ERROR:', msg.text());
});

const routes = ['/', '/repos', '/repos/test/test', '/repos/test/test/issues', '/search', '/settings'];
for (const route of routes) {
  await page.goto('http://127.0.0.1:9091' + route);
  await page.waitForTimeout(3000);
  const hasAside = await page.locator('aside').count();
  const hasNav = await page.locator('nav a').count();
  const bodyLen = (await page.textContent('body'))?.length || 0;
  console.log(`[${route}] aside=${hasAside} nav_a=${hasNav} body_len=${bodyLen}`);
}
await browser.close();
