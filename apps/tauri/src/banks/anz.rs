pub const ANZ_SCRIPT: &str = r#"
(async function ANZ_DomScraper() {
  const BANK_KEY = 'ANZ';
  const YEARS_BACK = __YEARS_BACK__;
  const RUN_ID = '__RUN_ID__';
  const cutoffDate = new Date();
  cutoffDate.setFullYear(cutoffDate.getFullYear() - YEARS_BACK);

  function log(level, message) {
    window.__TAURI_INTERNALS__.ipc.postMessage(JSON.stringify({
      cmd: 'bank_progress',
      payload: { bankKey: BANK_KEY, level, message, timestamp: new Date().toISOString() }
    }));
  }

  function sendBatch(account, transactions) {
    window.__TAURI_INTERNALS__.ipc.postMessage(JSON.stringify({
      cmd: 'bank_transactions',
      payload: { bankKey: BANK_KEY, runId: RUN_ID, account, transactions }
    }));
  }

  function parseAmount(text) {
    const clean = text.replace(/[^0-9.\-]/g, '');
    const num = parseFloat(clean);
    return isNaN(num) ? 0 : num;
  }

  function parseISODate(text) {
    const iso = text.match(/(\d{4})-(\d{2})-(\d{2})/);
    if (iso) return iso[0];
    const dmy = text.match(/(\d{1,2})\s+(\w{3})\s+(\d{4})/);
    if (dmy) {
      const m = {Jan:'01',Feb:'02',Mar:'03',Apr:'04',May:'05',Jun:'06',
                 Jul:'07',Aug:'08',Sep:'09',Oct:'10',Nov:'11',Dec:'12'};
      return `${dmy[3]}-${m[dmy[2]] || '01'}-${dmy[1].padStart(2,'0')}`;
    }
    return null;
  }

  function wait(ms) { return new Promise(r => setTimeout(r, ms)); }

  // ANZ Plus check
  if (window.location.href.includes('/plus')) {
    log('warn', 'ANZ Plus not supported');
    return;
  }

  try {
    log('info', 'ANZ DOM scraper started');

    // Navigate to accounts / transactions
    const accountsTab = document.querySelector("a[href*='accounts'], [href*='transaction']")
      || [...document.querySelectorAll('a')].find(a => /accounts|transactions?/i.test(a.textContent));

    if (accountsTab) {
      log('info', 'Navigating to accounts...');
      accountsTab.click();
      await wait(1500);
    }

    // Account dropdown (optional)
    const accountSel = document.querySelector(
      "select[id*='account'], select[class*='account-select'], [aria-label='Select account']"
    );
    const accountOptions = accountSel
      ? [...accountSel.options].filter(o => o.value)
      : [null];

    log('info', `Found ${accountOptions.length} account(s)`);

    for (const opt of accountOptions) {
      if (opt && accountSel) {
        accountSel.value = opt.value;
        accountSel.dispatchEvent(new Event('change', { bubbles: true }));
        await wait(1500);
        log('info', `Processing account: ${opt.text.trim()}`);
      }

      // Scrape account metadata
      const accountName = document.querySelector('.account-name, h1, .account-title')?.textContent?.trim()
        || opt?.text?.trim() || 'ANZ Account';
      const accountNumber = document.querySelector('.account-number, [class*="accountNumber"]')?.textContent?.trim() || '';
      const balanceText = document.querySelector('.balance, [class*="balance"]')?.textContent?.trim() || '0';
      const currentBalance = parseAmount(balanceText);

      const account = {
        bankKey: BANK_KEY,
        accountName,
        accountNumber,
        bsb: null,
        currency: 'AUD',
        accountType: 'CHECKING',
        currentBalance
      };

      // Scrape transaction rows — skip month header rows like "January 2024"
      const rows = [...document.querySelectorAll(
        '[class*="transaction"], table tbody tr'
      )];
      log('info', `Found ${rows.length} row(s)`);

      const transactions = [];

      for (const row of rows) {
        const rowText = row.textContent?.trim() || '';
        // Skip month header rows (e.g. "January 2024", "Feb 2024")
        if (/^(January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{4}$/i.test(rowText)) continue;

        const cells = row.querySelectorAll('td');
        const dateText = row.querySelector('[class*="date"], td:first-child')?.textContent?.trim() || '';
        const date = parseISODate(dateText);
        if (!date) continue;
        if (new Date(date) < cutoffDate) continue;

        const description = row.querySelector('[class*="desc"], [class*="description"]')?.textContent?.trim()
          || cells[1]?.textContent?.trim() || '';

        // Check for separate debit/credit columns
        const debitText = row.querySelector('[class*="debit"]')?.textContent?.trim();
        const creditText = row.querySelector('[class*="credit"]')?.textContent?.trim();
        let amount;
        if (debitText || creditText) {
          const debit = debitText ? parseAmount(debitText) : 0;
          const credit = creditText ? parseAmount(creditText) : 0;
          amount = credit > 0 ? credit : -debit;
        } else {
          const amountText = row.querySelector('[class*="amount"]')?.textContent?.trim()
            || cells[2]?.textContent?.trim() || '0';
          amount = parseAmount(amountText);
        }

        const balanceText2 = row.querySelector('[class*="balance"]')?.textContent?.trim()
          || cells[3]?.textContent?.trim() || '';
        const balance = balanceText2 ? parseAmount(balanceText2) : null;

        transactions.push({ date, description, amount, balance, reference: null, transactionType: null });
      }

      log('info', `Sending ${transactions.length} transaction(s) for account: ${accountName}`);
      sendBatch(account, transactions);
    }

    log('success', 'ANZ scraping complete');
  } catch (err) {
    log('error', `ANZ scraper error: ${err.message}`);
  }
})();
"#;
