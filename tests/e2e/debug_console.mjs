import { chromium } from '@playwright/test';

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
page.on('console', msg => console.log('CONSOLE:', msg.type(), msg.text()));
page.on('pageerror', err => console.log('PAGE ERROR:', err.message));
await page.goto('http://127.0.0.1:9091/');
await page.waitForTimeout(5000);
const html = await page.content();
console.log('BODY LENGTH:', html.length);
console.log('HAS ASIDE:', html.includes('<aside'));
console.log('HAS NAV:', html.includes('<nav'));
const bodyText = await page.textContent('body');
console.log('BODY TEXT:', bodyText?.substring(0, 500));
await browser.close();
