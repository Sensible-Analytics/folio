pub const BEYOND_SCRIPT: &str = r#"
(async function BEYOND_DomScraper() {
  const BANK_KEY = 'BEYOND';
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
    // Try DD/MM/YYYY
    const dmySlash = text.match(/(\d{1,2})\/(\d{1,2})\/(\d{4})/);
    if (dmySlash) {
      return `${dmySlash[3]}-${dmySlash[2].padStart(2,'0')}-${dmySlash[1].padStart(2,'0')}`;
    }
    return null;
  }

  function wait(ms) { return new Promise(r => setTimeout(r, ms)); }

  // React-compatible input setter
  function setInput(el, value) {
    const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value')?.set;
    if (nativeSetter) nativeSetter.call(el, value);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }

  function formatDDMMYYYY(d) {
    return `${String(d.getDate()).padStart(2,'0')}/${String(d.getMonth()+1).padStart(2,'0')}/${d.getFullYear()}`;
  }

  try {
    log('info', 'Beyond Bank DOM scraper started');

    // Navigate to transaction history
    const txLink = document.querySelector("a[href*='transaction'], a[href*='history']")
      || [...document.querySelectorAll('a')].find(a => /transactions?|history/i.test(a.textContent));

    if (txLink) {
      log('info', 'Navigating to transaction history...');
      txLink.click();
      await wait(2500);
    } else {
      log('info', 'No navigation link found, proceeding on current page');
    }

    // Scrape account metadata
    const accountName = document.querySelector('.account-name, h1, .account-title')?.textContent?.trim() || 'Beyond Bank Account';
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

    // Set date range
    const today = new Date();
    const fromInput = document.querySelector(
      "input[id*='from'], input[name*='from']"
    );
    const toInput = document.querySelector(
      "input[id*='to'], input[name*='to']"
    );

    if (fromInput) {
      setInput(fromInput, formatDDMMYYYY(cutoffDate));
      log('info', `Set from date: ${formatDDMMYYYY(cutoffDate)}`);
    }
    if (toInput) {
      setInput(toInput, formatDDMMYYYY(today));
      log('info', `Set to date: ${formatDDMMYYYY(today)}`);
    }

    // Click search
    const searchBtn = [...document.querySelectorAll('button[type="submit"], .search-btn, input[type="submit"]')]
      .find(b => !b.disabled)
      || [...document.querySelectorAll('button')].find(b => /search|find/i.test(b.textContent));
    if (searchBtn) {
      searchBtn.click();
      await wait(2000);
      log('info', 'Clicked search button');
    }

    // Paginate and collect transactions
    let page = 1;
    const allTransactions = [];

    while (true) {
      const rows = [...document.querySelectorAll('table tbody tr')];
      log('info', `Page ${page}: found ${rows.length} row(s)`);

      for (const row of rows) {
        const cells = row.querySelectorAll('td');
        const dateText = cells[0]?.textContent?.trim()
          || row.querySelector('[class*="date"]')?.textContent?.trim() || '';
        const date = parseISODate(dateText);
        if (!date) continue;
        if (new Date(date) < cutoffDate) continue;

        const description = cells[1]?.textContent?.trim()
          || row.querySelector('[class*="desc"], [class*="description"]')?.textContent?.trim() || '';

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

        allTransactions.push({ date, description, amount, balance, reference: null, transactionType: null });
      }

      // Next page
      const nextBtn = document.querySelector(
        "a[aria-label='Next page'], .pagination-next, button.next"
      ) || [...document.querySelectorAll('a, button')].find(b => /^next$/i.test(b.textContent?.trim()));

      if (!nextBtn || nextBtn.hasAttribute('disabled') || nextBtn.getAttribute('aria-disabled') === 'true') break;
      nextBtn.click();
      await wait(2000);
      page++;
      if (page > 50) break; // safety cap
    }

    log('info', `Sending ${allTransactions.length} transaction(s) for account: ${accountName}`);
    sendBatch(account, allTransactions);
    log('success', 'BEYOND scraping complete');
  } catch (err) {
    log('error', `BEYOND scraper error: ${err.message}`);
  }
})();
"#;
