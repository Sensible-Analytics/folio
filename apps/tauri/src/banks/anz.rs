pub const ANZ_SCRIPT: &str = r#"
(async function anzAutomation() {
  const yearsBack = __YEARS_BACK__;
  const BANK = 'ANZ';

  const MONTH_MAP = {
    january:'01', february:'02', march:'03', april:'04',
    may:'05', june:'06', july:'07', august:'08',
    september:'09', october:'10', november:'11', december:'12'
  };

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

  function parseMonthYear(text) {
    if (!text) return null;
    const m = text.toLowerCase().match(/([a-z]+)\s+(\d{4})/);
    if (!m) return null;
    const month = MONTH_MAP[m[1]];
    return month ? `${m[2]}-${month}` : null;
  }

  log('info', 'ANZ automation started');

  // ANZ Plus check
  if (window.location.href.includes('/plus')) {
    log('warn', 'ANZ Plus detected — ANZ Plus uses a separate app, skipping.');
    reportUrls([]);
    return;
  }

  try {
    // ── Navigate to Accounts → Statements ─────────────────────────────────
    const accountsTab = document.querySelector("[role='tab']:not([aria-selected='false'])")
      || [...document.querySelectorAll("a, [role='tab']")]
          .find(a => /accounts/i.test(a.textContent));
    if (accountsTab) { accountsTab.click(); await wait(1500); }

    const stmtLink = document.querySelector("a[href*='statement']")
      || [...document.querySelectorAll('a')]
          .find(a => /statements?\s*(& documents)?/i.test(a.textContent));

    if (!stmtLink) {
      log('warn', 'Statements link not found. Please navigate to Statements manually.');
      reportUrls([]);
      return;
    }
    log('info', 'Navigating to Statements & Documents...');
    stmtLink.click();
    await wait(2500);

    // ── Account dropdown (optional) ───────────────────────────────────────
    const accountSel = document.querySelector(
      "select[id*='account'], select[class*='account-select'], [aria-label='Select account']"
    );
    const accountOptions = accountSel
      ? [...accountSel.options].filter(o => o.value)
      : [null];

    const allUrls = [];

    for (const opt of accountOptions) {
      if (opt && accountSel) {
        accountSel.value = opt.value;
        accountSel.dispatchEvent(new Event('change', { bubbles: true }));
        await wait(1500);
        log('info', `Processing account: ${opt.text.trim()}`);
      }

      const rows = [...document.querySelectorAll(
        ".statement-list li, table tbody tr, [class*='statement-item']"
      )];
      log('info', `Found ${rows.length} statement row(s)`);

      for (const row of rows) {
        const dateEl = row.querySelector("[class*='date'], td:first-child");
        const dateText = dateEl?.textContent?.trim();
        const period = parseMonthYear(dateText);

        if (period) {
          const year = parseInt(period.split('-')[0]);
          if (year < cutoff.getFullYear()) continue;
        }

        const dlLink = row.querySelector(
          "a[href$='.pdf'], a[aria-label*='Download statement'], a:not([href^='#'])"
        );
        if (dlLink) {
          allUrls.push({
            url: dlLink.href,
            filename: `ANZ_${opt?.text?.trim() || 'account'}_${period || dateText || 'statement'}.pdf`
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
