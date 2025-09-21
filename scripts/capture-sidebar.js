#!/usr/bin/env node
/*
  Capture Aigent Z sidebar screenshot for docs
  - Expects Aigent Z running on http://localhost:3000/dashboard
  - Saves to docs/operating-manual/static/img/sidebar-iqubes-colors.png
*/
const fs = require('fs');
const path = require('path');

(async () => {
  const targetUrl = process.env.AIGENTZ_URL || 'http://localhost:3000/dashboard';
  const outPath = path.resolve(__dirname, '..', 'docs', 'operating-manual', 'static', 'img', 'sidebar-iqubes-colors.png');

  // Ensure output directory exists
  fs.mkdirSync(path.dirname(outPath), { recursive: true });

  // Basic availability check
  try {
    const res = await fetch(targetUrl, { method: 'GET' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
  } catch (e) {
    console.error(`[capture-sidebar] Target not reachable at ${targetUrl}. Start Aigent Z (port 3000) and retry.`);
    process.exit(2);
  }

  const puppeteer = require('puppeteer');
  const execPath = puppeteer.executablePath();
  const browser = await puppeteer.launch({
    headless: 'new',
    executablePath: execPath,
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
    defaultViewport: { width: 1440, height: 900 },
    timeout: 60000,
  });
  const page = await browser.newPage();
  page.setDefaultNavigationTimeout(60000);

  await page.goto(targetUrl, { waitUntil: 'networkidle2' });
  // Give Next.js a moment for client hydration and sidebar render
  await new Promise((r) => setTimeout(r, 1200));

  // Capture a clipped region covering the left sidebar
  // Assumes sidebar width ~280-300px; capture 320px for safety
  const clip = { x: 0, y: 0, width: 320, height: 900 };
  await page.screenshot({ path: outPath, clip });

  await browser.close();
  console.log(`[capture-sidebar] Saved screenshot to ${outPath}`);
})();
