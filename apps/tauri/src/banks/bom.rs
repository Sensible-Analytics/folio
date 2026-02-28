pub const BOM_SCRIPT: &str = r#"
(async function bomAutomation() {
  const yearsBack = __YEARS_BACK__;
  const BANK = 'BOM';

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

  // Parse "01 Jan 2024" → "2024-01"
  const MONTH_MAP = {
    jan:'01', feb:'02', mar:'03', apr:'04', may:'05', jun:'06',
    jul:'07', aug:'08', sep:'09', oct:'10', nov:'11', dec:'12'
  };

  function parseDDMonYYYY(text) {
    if (!text) return null;
    const m = text.toLowerCase().match(/(\d{1,2})\s+([a-z]{3})\s+(\d{4})/);
    if (!m) return null;
    const mon = MONTH_MAP[m[2]];
    return mon ? `${m[3]}-${mon}` : null;
  }

  log('info', 'Bank of Melbourne automation started');
  log('info', 'Note: BOM uses the Westpac Group portal (same as St.George, BankSA)');

  try {
    // ── Navigate to Statements ─────────────────────────────────────────────
    const stmtLink = document.querySelector(
      "a[href*='viewStatement'], a[href*='statements']"
    ) || [...document.querySelectorAll('a')]
        .find(a => /statements?/i.test(a.textContent));

    if (!stmtLink) {
      log('warn', 'Statements link not found. Please navigate to Statements manually.');
      reportUrls([]);
      return;
    }
    log('info', 'Navigating to Statements...');
    stmtLink.click();
    await wait(2500);

    // ── Click account tabs ────────────────────────────────────────────────
    const tabs = [...document.querySelectorAll(
      "[role='tab'], .tab-item, a.tab, ul.tabs li a"
    )];
    const targets = tabs.length ? tabs : [null];

    const allUrls = [];

    for (const tab of targets) {
      if (tab) {
        tab.click();
        await wait(1500);
        log('info', `Processing tab: ${tab.textContent.trim()}`);
      }

      // ── Statement table rows ──────────────────────────────────────────
      const rows = [...document.querySelectorAll(
        "table.statement-table tbody tr, table tbody tr"
      )];

      for (const row of rows) {
        const cells = row.querySelectorAll('td');
        const dateText = cells[0]?.textContent?.trim() || cells[1]?.textContent?.trim();
        const period = parseDDMonYYYY(dateText);

        if (period) {
          const year = parseInt(period.split('-')[0]);
          if (year < cutoff.getFullYear()) continue;
        }

        const dlLink = row.querySelector(
          "a[href*='downloadStatement'], a[href$='.pdf']"
        ) || [...row.querySelectorAll('a')]
            .find(a => /download pdf/i.test(a.textContent));

        if (dlLink) {
          allUrls.push({
            url: dlLink.href,
            filename: `BOM_${tab?.textContent?.trim() || 'account'}_${period || dateText || 'statement'}.pdf`
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
