import { chromium } from '@playwright/test';

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
page.on('console', msg => console.log('CONSOLE:', msg.type(), msg.text()));
page.on('pageerror', err => console.log('PAGE ERROR:', err.message));
page.on('requestfailed', req => console.log('FAILED:', req.url(), req.failure()?.errorText));

console.log('--- Navigating to / ---');
await page.goto('http://127.0.0.1:9091/');
console.log('networkidle reached');
await page.waitForTimeout(2000);
let bodyLen = (await page.textContent('body'))?.length;
console.log('Body after 2s:', bodyLen);
await page.waitForTimeout(5000);
bodyLen = (await page.textContent('body'))?.length;
console.log('Body after 7s:', bodyLen);
const hasAside = await page.locator('aside').count();
console.log('aside count:', hasAside);
const hasNav = await page.locator('nav').count();
console.log('nav count:', hasNav);
const hasNavA = await page.locator('nav a').count();
console.log('nav a count:', hasNavA);

// Now test the navigation that the test does
console.log('--- Navigating to /repos/test/test ---');
await page.goto('http://127.0.0.1:9091/repos/test/test');
await page.waitForTimeout(3000);
bodyLen = (await page.textContent('body'))?.length;
console.log('Body after /repos/test/test:', bodyLen);
const hasAside2 = await page.locator('aside').count();
console.log('aside count:', hasAside2);

await browser.close();
