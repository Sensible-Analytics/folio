pub const BEYOND_SCRIPT: &str = r#"
(async function beyondAutomation() {
  const yearsBack = __YEARS_BACK__;
  const BANK = 'BEYOND';

  const today = new Date();
  const cutoff = new Date();
  cutoff.setFullYear(today.getFullYear() - yearsBack);

  // Format date as DD/MM/YYYY for Beyond Bank's date inputs
  function formatDate(d) {
    return `${String(d.getDate()).padStart(2,'0')}/${String(d.getMonth()+1).padStart(2,'0')}/${d.getFullYear()}`;
  }

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

  function setInput(el, value) {
    const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')?.set;
    if (nativeSetter) nativeSetter.call(el, value);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }

  log('info', 'Beyond Bank automation started');

  try {
    // ── Navigate to Statements ─────────────────────────────────────────────
    const stmtLink = document.querySelector("a[href*='statement']")
      || [...document.querySelectorAll('a')]
          .find(a => /statements?/i.test(a.textContent));

    if (!stmtLink) {
      log('warn', 'Statements link not found. Please navigate to Statements manually.');
      reportUrls([]);
      return;
    }
    log('info', 'Navigating to Statements...');
    stmtLink.click();
    await wait(2500);

    // ── Process each account ──────────────────────────────────────────────
    const accountItems = [...document.querySelectorAll(
      ".account-list li, [class*='account-item'], select[name*='account'] option"
    )];
    const accounts = accountItems.length ? accountItems : [null];

    const allUrls = [];

    for (const acct of accounts) {
      if (acct) {
        acct.click?.();
        await wait(1000);
        log('info', `Processing account: ${acct.textContent?.trim()}`);
      }

      // ── Set date range ───────────────────────────────────────────────
      const fromInput = document.querySelector(
        "input[id*='from'], input[name*='from'], input[placeholder*='From'], input[aria-label*='From date']"
      );
      const toInput = document.querySelector(
        "input[id*='to'], input[name*='to'], input[placeholder*='To'], input[aria-label*='To date']"
      );

      if (fromInput) {
        setInput(fromInput, formatDate(cutoff));
        log('info', `Set from date: ${formatDate(cutoff)}`);
      }
      if (toInput) {
        setInput(toInput, formatDate(today));
        log('info', `Set to date: ${formatDate(today)}`);
      }

      // ── Click Search ─────────────────────────────────────────────────
      const searchBtn = document.querySelector("button:not([disabled])")
        || [...document.querySelectorAll('button, input[type="submit"]')]
            .find(b => /search|find/i.test(b.textContent || b.value));
      if (searchBtn) { searchBtn.click(); await wait(2000); }

      // ── Paginate and collect PDFs ─────────────────────────────────────
      let page = 1;
      while (true) {
        const rows = [...document.querySelectorAll(
          "table tbody tr, .statement-row, [class*='statement-item']"
        )];
        log('info', `Page ${page}: found ${rows.length} row(s)`);

        for (const row of rows) {
          const dlLink = row.querySelector("a[href$='.pdf'], a[href*='download']")
            || [...row.querySelectorAll('a')].find(a => /download/i.test(a.textContent));
          if (dlLink) {
            // Extract YYYY-MM from filename if possible
            const fnMatch = dlLink.href.match(/(\d{4})-(\d{2})/);
            const period = fnMatch ? `${fnMatch[1]}-${fnMatch[2]}` : 'statement';
            allUrls.push({
              url: dlLink.href,
              filename: `BEYOND_${acct?.textContent?.trim() || 'account'}_${period}.pdf`
            });
          }
        }

        // Next page
        const nextBtn = document.querySelector(
          "a[aria-label='Next page'], button[aria-label='Next page']"
        ) || [...document.querySelectorAll('a, button')]
            .find(b => /^next$/i.test(b.textContent?.trim()));

        if (!nextBtn || nextBtn.hasAttribute('disabled') || nextBtn.getAttribute('aria-disabled') === 'true') break;
        nextBtn.click();
        await wait(2000);
        page++;
        if (page > 50) break; // safety cap
      }
    }

    log(allUrls.length ? 'success' : 'warn',
      allUrls.length ? `Found ${allUrls.length} statement(s)` : 'No statements found');
    reportUrls(allUrls);

  } catch (err) {
    log('error', `Automation error: ${err.message}`);
    reportUrls([]);
  }
})();
"#;
