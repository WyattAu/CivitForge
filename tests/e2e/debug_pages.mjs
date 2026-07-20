import { chromium } from '@playwright/test';

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
page.on('pageerror', err => console.log('PAGE ERROR:', err.message));

const pages = ['/', '/repos/test/test', '/repos/test/test/issues', '/repos/test/test/pulls', '/repos/test/test/pipelines'];
for (const p of pages) {
  await page.goto('http://127.0.0.1:9091' + p);
  await page.waitForTimeout(3000);
  const h1 = await page.locator('h1').count();
  const h2 = await page.locator('h2').count();
  const h3 = await page.locator('h3').count();
  const bodyText = await page.textContent('body');
  console.log(`[${p}] h1=${h1} h2=${h2} h3=${h3} body="${bodyText?.substring(0, 200)}"`);
}
await browser.close();
