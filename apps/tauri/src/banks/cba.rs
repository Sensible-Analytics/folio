pub const CBA_SCRIPT: &str = r#"
(async function cbaAutomation() {
  const yearsBack = __YEARS_BACK__;
  const BANK = 'CBA';

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

  log('info', 'CBA NetBank automation started');

  try {
    // ── Navigate to Accounts → Statements ─────────────────────────────────
    const accountsNav = document.querySelector(
      "a[href*='accounts'], a[href*='balances'], [data-testid='accounts-nav']"
    ) || [...document.querySelectorAll('a')]
        .find(a => /view accounts|accounts/i.test(a.textContent));

    if (accountsNav) {
      log('info', 'Navigating to Accounts...');
      accountsNav.click();
      await wait(2000);
    }

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

    // ── Account dropdown ──────────────────────────────────────────────────
    const accountSel = document.querySelector(
      "select[id*='account'], select[aria-label*='account'], select[name*='AccountSelect']"
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

      // ── Walk statement table rows ─────────────────────────────────────
      const rows = [...document.querySelectorAll(
        "table tbody tr, .statement-list__item, [class*='StatementRow']"
      )];
      log('info', `Found ${rows.length} statement row(s)`);

      for (const row of rows) {
        // Parse date from first cell to apply cutoff
        const dateText = row.querySelector('td:first-child, [class*="date"], [class*="period"]')?.textContent?.trim();
        if (dateText) {
          const yearMatch = dateText.match(/(\d{4})/);
          if (yearMatch && parseInt(yearMatch[1]) < cutoff.getFullYear()) continue;
        }

        const pdfLink = row.querySelector(
          "a[href$='.pdf'], a[title*='PDF'], a[aria-label*='Download']"
        ) || [...row.querySelectorAll('a')].find(a => /pdf|download/i.test(a.textContent));

        if (pdfLink) {
          allUrls.push({
            url: pdfLink.href,
            filename: `CBA_${dateText || 'statement'}.pdf`
          });
        }
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
