pub const ING_SCRIPT: &str = r#"
(async function ingAutomation() {
  const yearsBack = __YEARS_BACK__;
  const BANK = 'ING';

  const cutoff = new Date();
  cutoff.setFullYear(cutoff.getFullYear() - yearsBack);

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

  async function waitForSelector(sel, timeout = 10000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(200);
    }
    return null;
  }

  log('info', 'ING automation started');

  try {
    // ── Navigate to Statements ─────────────────────────────────────────────
    const stmtLink = document.querySelector(
      "a[href*='statement'], a[href*='eStatement'], nav a"
    );
    const stmtByText = [...document.querySelectorAll('a')]
      .find(a => /statements?/i.test(a.textContent));
    const target = stmtLink || stmtByText;

    if (!target) {
      log('warn', 'Statements link not found. Please navigate to Statements manually then click Start Download again.');
      reportUrls([]);
      return;
    }
    log('info', 'Navigating to Statements page...');
    target.click();
    await wait(2500);

    // ── Collect accounts from dropdown ────────────────────────────────────
    const accountSel = document.querySelector(
      "select[id*='account'], select[name*='account'], select[aria-label*='account']"
    );
    const accountOptions = accountSel
      ? [...accountSel.options].filter(o => o.value)
      : [null];

    log('info', `Found ${accountOptions.length} account(s)`);

    const allUrls = [];

    for (const opt of accountOptions) {
      if (opt && accountSel) {
        accountSel.value = opt.value;
        accountSel.dispatchEvent(new Event('change', { bubbles: true }));
        await wait(1500);
        log('info', `Processing account: ${opt.text.trim()}`);
      }

      // ── Iterate period / year dropdown ──────────────────────────────────
      const periodSel = document.querySelector(
        "select[id*='period'], select[name*='period'], select[id*='year']"
      );
      const periods = periodSel
        ? [...periodSel.options].filter(o => o.value)
        : [null];

      for (const period of periods) {
        if (period && periodSel) {
          // Parse year from option text; skip if before cutoff
          const yearMatch = period.text.match(/(\d{4})/);
          if (yearMatch && parseInt(yearMatch[1]) < cutoff.getFullYear()) continue;

          periodSel.value = period.value;
          periodSel.dispatchEvent(new Event('change', { bubbles: true }));
          await wait(500);

          // Click Find/Search button
          const findBtn = document.querySelector(
            "button:not([disabled])[type='submit'], input[value='Find'], button"
          );
          const findByText = [...document.querySelectorAll('button')]
            .find(b => /find|search/i.test(b.textContent));
          const btn = findBtn || findByText;
          if (btn) { btn.click(); await wait(1500); }
        }

        // ── Collect PDF links ──────────────────────────────────────────────
        const links = [...document.querySelectorAll(
          "a[href$='.pdf'], a[href*='/statement'][href*='download'], a[download]"
        )].map(a => ({
          url: a.href,
          filename: (opt?.text?.trim() || 'ING') + '_' +
                    (period?.text?.trim() || '') + '.pdf'
        })).filter(item => item.url.startsWith('http'));

        allUrls.push(...links);
        if (links.length) log('info', `Found ${links.length} PDF(s) for period ${period?.text || 'all'}`);
      }
    }

    log(allUrls.length ? 'success' : 'warn',
      allUrls.length ? `Found ${allUrls.length} statement(s) total` : 'No statements found');
    reportUrls(allUrls);

  } catch (err) {
    log('error', `Automation error: ${err.message}`);
    reportUrls([]);
  }
})();
"#;
