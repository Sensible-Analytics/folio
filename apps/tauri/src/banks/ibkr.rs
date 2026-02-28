pub const IBKR_SCRIPT: &str = r#"
(async function ibkrAutomation() {
  const yearsBack = __YEARS_BACK__;
  const BANK = 'IBKR';

  const today = new Date();
  const cutoff = new Date();
  cutoff.setFullYear(today.getFullYear() - yearsBack);

  function log(level, msg) {
    console.log(`[${BANK}][${level}] ${msg}`);
    if (window.__TAURI_INTERNALS__?.ipc) {
      window.__TAURI_INTERNALS__.ipc.postMessage(JSON.stringify({
        cmd: 'bank_progress', callback: 0, error: 0,
        payload: { bankKey: BANK, level, message: msg, timestamp: new Date().toISOString() }
      }));
    }
  }

  function reportUrls(urls) {
    if (window.__TAURI_INTERNALS__?.ipc) {
      window.__TAURI_INTERNALS__.ipc.postMessage(JSON.stringify({
        cmd: 'bank_urls', callback: 0, error: 0,
        payload: { bankKey: BANK, urls }
      }));
    }
  }

  function wait(ms) { return new Promise(r => setTimeout(r, ms)); }

  log('info', 'Interactive Brokers automation started');
  log('info', `Looking for Activity Statements going back ${yearsBack} year(s)`);

  try {
    // ── Navigate to Reports / Statements ──────────────────────────────────
    // IBKR portal uses a SPA with hash routing
    const reportsLink = document.querySelector(
      "a[href*='reports'], a[href*='statement'], [data-nav='reports']"
    ) || [...document.querySelectorAll('a, button, [role="menuitem"]')]
        .find(el => /reports?|statements?|activity/i.test(el.textContent));

    if (reportsLink) {
      log('info', 'Navigating to Reports...');
      reportsLink.click();
      await wait(2000);
    } else {
      log('info', 'Reports menu not found via click — trying hash navigation');
      // IBKR portal hash nav fallback
      window.location.hash = '#/reports/flex-queries';
      await wait(2000);
    }

    // ── Find Activity Statement section ───────────────────────────────────
    const activityLink = [...document.querySelectorAll('a, button, [role="tab"]')]
      .find(el => /activity statements?/i.test(el.textContent));
    if (activityLink) {
      activityLink.click();
      await wait(1500);
    }

    // ── Collect available download links ──────────────────────────────────
    const allUrls = [];

    // Look for PDF / CSV download links in statement tables
    const dlLinks = [...document.querySelectorAll(
      "a[href*='download'], a[href*='getStatement'], a[href*='ActivityStatement'], a[href$='.pdf'], a[href$='.csv']"
    )];

    log('info', `Found ${dlLinks.length} download link(s)`);

    for (const link of dlLinks) {
      const rowText = link.closest('tr, [class*="row"], li')?.textContent || '';
      const yearMatch = rowText.match(/20(\d{2})/);
      const year = yearMatch ? 2000 + parseInt(yearMatch[1]) : today.getFullYear();

      if (year >= cutoff.getFullYear()) {
        const monthMatch = rowText.match(/(January|February|March|April|May|June|July|August|September|October|November|December)/i);
        const period = monthMatch
          ? `${year}-${String(['january','february','march','april','may','june','july','august','september','october','november','december'].indexOf(monthMatch[1].toLowerCase()) + 1).padStart(2,'0')}`
          : `${year}`;

        allUrls.push({
          url: link.href,
          filename: `IBKR_${period}_activity.${link.href.includes('.csv') ? 'csv' : 'pdf'}`
        });
      }
    }

    if (allUrls.length === 0) {
      log('info', 'No direct links found — checking for statement generation form...');

      // Some IBKR users need to generate a Flex Query report
      const generateBtn = [...document.querySelectorAll('button')]
        .find(b => /generate|run|create/i.test(b.textContent));
      if (generateBtn) {
        log('info', 'Found Generate button — you may need to configure and run a Flex Query manually');
      }

      log('warn', [
        'No downloadable statements found automatically.',
        'In IBKR Portal: go to Reports → Activity → select period → click Download.',
        'Or use Reports → Flex Queries to generate a custom activity report.'
      ].join(' '));
    }

    log(allUrls.length ? 'success' : 'info',
      allUrls.length ? `Found ${allUrls.length} statement(s) to download` : 'Manual download required');
    reportUrls(allUrls);

  } catch (err) {
    log('error', `Automation error: ${err.message}`);
    reportUrls([]);
  }
})();
"#;
