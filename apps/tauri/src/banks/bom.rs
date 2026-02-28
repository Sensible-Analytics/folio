pub const BOM_SCRIPT: &str = r#"
(async function BOM_DomScraper() {
  const BANK_KEY = 'BOM';
  const YEARS_BACK = __YEARS_BACK__;
  const RUN_ID = '__RUN_ID__';
  const cutoffDate = new Date();
  cutoffDate.setFullYear(cutoffDate.getFullYear() - YEARS_BACK);

  function log(level, message) {
    window.__TAURI__.invoke('bank_progress', { bankKey: BANK_KEY, level, message, timestamp: new Date().toISOString() });
  }

  function sendBatch(account, transactions) {
    window.__TAURI__.invoke('bank_transactions', { bankKey: BANK_KEY, runId: RUN_ID, account, transactions });
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

  try {
    log('info', 'Bank of Melbourne DOM scraper started (Westpac Group portal)');

    // Navigate to accounts list
    const accountsLink = document.querySelector("a[href*='accounts'], a[href*='transaction']")
      || [...document.querySelectorAll('a')].find(a => /accounts|transactions?/i.test(a.textContent));

    if (accountsLink) {
      log('info', 'Navigating to accounts...');
      accountsLink.click();
      await wait(2000);
    }

    // Click each account tab
    const tabs = [...document.querySelectorAll("[role='tab'], .tab-item, a.tab")];
    const targets = tabs.length ? tabs : [null];

    log('info', `Found ${targets.length} account tab(s)`);

    for (const tab of targets) {
      if (tab) {
        tab.click();
        await wait(1500);
        log('info', `Processing tab: ${tab.textContent.trim()}`);
      }

      // Scrape account metadata
      const accountName = document.querySelector('.account-name, h1, .account-title')?.textContent?.trim()
        || tab?.textContent?.trim() || 'BOM Account';
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

      // Scrape transaction rows
      const rows = [...document.querySelectorAll(
        'table tbody tr, [class*="transactionRow"]'
      )];
      log('info', `Found ${rows.length} transaction row(s)`);

      const transactions = [];

      for (const row of rows) {
        const cells = row.querySelectorAll('td');
        // Date format: "01 Jan 2024"
        const dateText = cells[0]?.textContent?.trim()
          || row.querySelector('[class*="date"]')?.textContent?.trim() || '';
        const date = parseISODate(dateText);
        if (!date) continue;
        if (new Date(date) < cutoffDate) continue;

        const description = cells[1]?.textContent?.trim()
          || row.querySelector('[class*="desc"], [class*="description"]')?.textContent?.trim() || '';

        // Check for separate debit/credit columns
        const debitText = row.querySelector('[class*="debit"]')?.textContent?.trim();
        const creditText = row.querySelector('[class*="credit"]')?.textContent?.trim();
        let amount;
        if (debitText || creditText) {
          const debit = debitText ? parseAmount(debitText) : 0;
          const credit = creditText ? parseAmount(creditText) : 0;
          amount = credit > 0 ? credit : -debit;
        } else {
          const amountText = cells[2]?.textContent?.trim()
            || row.querySelector('[class*="amount"]')?.textContent?.trim() || '0';
          amount = parseAmount(amountText);
        }

        const balanceText2 = cells[3]?.textContent?.trim()
          || row.querySelector('[class*="balance"]')?.textContent?.trim() || '';
        const balance = balanceText2 ? parseAmount(balanceText2) : null;

        transactions.push({ date, description, amount, balance, reference: null, transactionType: null });
      }

      log('info', `Sending ${transactions.length} transaction(s) for account: ${accountName}`);
      sendBatch(account, transactions);
    }

    log('success', 'BOM scraping complete');
  } catch (err) {
    log('error', `BOM scraper error: ${err.message}`);
  }
})();
"#;
