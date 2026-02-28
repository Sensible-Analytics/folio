pub const IBKR_SCRIPT: &str = r#"
(async function IBKR_DomScraper() {
  const BANK_KEY = 'IBKR';
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
    log('info', 'IBKR DOM scraper started');

    // Navigate to reports/activity
    const reportsLink = document.querySelector("a[href*='reports'], a[href*='activity']")
      || [...document.querySelectorAll('a, button, [role="menuitem"]')]
          .find(el => /reports?|activity/i.test(el.textContent));

    if (reportsLink) {
      log('info', 'Navigating to Reports...');
      reportsLink.click();
      await wait(2000);
    } else {
      log('info', 'Reports link not found, trying hash navigation');
      window.location.hash = '#/reports/activity';
      await wait(2000);
    }

    // Try alternate hash if needed
    const activityLink = [...document.querySelectorAll('a, button, [role="tab"]')]
      .find(el => /activity/i.test(el.textContent));
    if (activityLink) {
      activityLink.click();
      await wait(1500);
    }

    // Scrape account metadata
    const accountName = document.querySelector('.account-name, h1, [class*="accountName"]')?.textContent?.trim() || 'IBKR Account';
    const accountNumber = document.querySelector('.account-number, [class*="accountId"], [class*="accountNumber"]')?.textContent?.trim() || '';
    const balanceText = document.querySelector('[class*="netAssetValue"], [class*="balance"], .nav-value')?.textContent?.trim() || '0';
    const currentBalance = parseAmount(balanceText);

    const account = {
      bankKey: BANK_KEY,
      accountName,
      accountNumber,
      bsb: null,
      currency: 'USD',
      accountType: 'TRADING',
      currentBalance
    };

    // Scrape transaction rows
    const rows = [...document.querySelectorAll(
      'table tbody tr[class*="row"], [class*="activityRow"]'
    )];
    log('info', `Found ${rows.length} row(s)`);

    const transactions = [];

    for (const row of rows) {
      const cells = row.querySelectorAll('td');
      if (cells.length < 2) continue;

      const dateText = cells[0]?.textContent?.trim() || '';
      const date = parseISODate(dateText);
      if (!date) continue;
      if (new Date(date) < cutoffDate) continue;

      const description = cells[1]?.textContent?.trim() || '';

      // Determine if trade or cash transaction
      const symbolCell = row.querySelector('[class*="symbol"], td:nth-child(2)')?.textContent?.trim() || '';
      const qtyText = row.querySelector('[class*="quantity"], td:nth-child(3)')?.textContent?.trim() || '';
      const qty = parseAmount(qtyText);

      let amount = 0;
      let transactionType = null;

      if (symbolCell && symbolCell.length < 10 && qty !== 0) {
        // It's a trade row
        const priceText = row.querySelector('[class*="price"], td:nth-child(4)')?.textContent?.trim() || '0';
        const price = parseAmount(priceText);
        amount = qty * price;
        transactionType = qty > 0 ? 'BUY' : 'SELL';
        if (transactionType === 'SELL') amount = -Math.abs(amount);
      } else {
        // Cash transaction
        const amountText = row.querySelector('[class*="amount"], td:last-child')?.textContent?.trim() || '0';
        amount = parseAmount(amountText);
        transactionType = amount >= 0 ? 'DEPOSIT' : 'WITHDRAWAL';
      }

      transactions.push({ date, description, amount, balance: null, reference: null, transactionType });
    }

    if (transactions.length === 0) {
      log('info', 'No transactions found automatically. To download statements manually, go to Reports → Activity → Custom Date Range and export as CSV');
    } else {
      log('info', `Sending ${transactions.length} transaction(s) for account: ${accountName}`);
      sendBatch(account, transactions);
    }

    log('success', 'IBKR scraping complete');
  } catch (err) {
    log('error', `IBKR scraper error: ${err.message}`);
  }
})();
"#;
