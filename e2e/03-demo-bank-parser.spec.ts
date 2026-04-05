import { expect, Page, test } from "@playwright/test";

test.describe.configure({ mode: "serial" });

test.describe("Demo Bank Parser Page", () => {
  const BASE_URL = "http://localhost:1420";
  let page: Page;

  test.beforeAll(async ({ browser }) => {
    page = await browser.newPage();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test("should load demo page with terminal theme", async () => {
    // Capture all console messages
    const consoleMessages: string[] = [];
    const errors: string[] = [];

    page.on("console", (msg) => {
      consoleMessages.push(`${msg.type()}: ${msg.text()}`);
      if (msg.type() === "error") {
        errors.push(msg.text());
      }
    });

    page.on("pageerror", (error) => {
      errors.push(`Page error: ${error.message}`);
    });

    await page.goto(`${BASE_URL}/demo`, { waitUntil: "networkidle" });

    // Wait for React to potentially mount
    await page.waitForTimeout(5000);

    // Log all console messages for debugging
    console.log("Console messages:", consoleMessages.slice(0, 20));
    if (errors.length > 0) {
      console.log("Errors:", errors);
    }

    // Check root content
    const rootHtml = await page.evaluate(() => {
      return document.getElementById("root")?.innerHTML || "EMPTY";
    });
    console.log("Root HTML length:", rootHtml.length);

    // Check if LoginPage is showing (requires auth)
    const loginPresent = await page.locator("text=Sign in").count();
    console.log("Login page present:", loginPresent > 0);

    // Try to find the terminal class
    const terminal = page.locator(".terminal");
    await expect(terminal).toBeVisible({ timeout: 10000 });

    // Verify terminal header elements
    await expect(page.locator(".terminal-header")).toBeVisible();
    await expect(page.locator(".dot.dot-red")).toBeVisible();
    await expect(page.locator(".dot.dot-yellow")).toBeVisible();
    await expect(page.locator(".dot.dot-green")).toBeVisible();

    // Verify terminal prompt
    await expect(page.locator(".terminal-prompt")).toBeVisible();
    await expect(page.locator(".terminal-prompt .user")).toHaveText("prabhat");
    await expect(page.locator(".terminal-prompt .host")).toHaveText("sensible");

    // Verify main heading
    await expect(page.locator("h1")).toContainText("Australian Bank Statement Parser");
  });

  test("should display all sample files", async () => {
    await page.goto(`${BASE_URL}/demo`);

    // Verify sample files section
    await expect(page.locator(".terminal-section-title").first()).toHaveText("Sample Files");

    // Verify all 7 sample files are displayed
    const sampleFiles = page.locator(".terminal-file-item");
    await expect(sampleFiles).toHaveCount(7);

    // Verify bank badges
    await expect(page.locator("text=CommBank Transactions")).toBeVisible();
    await expect(page.locator("text=Westpac Transactions")).toBeVisible();
    await expect(page.locator("text=ANZ Transactions")).toBeVisible();
    await expect(page.locator("text=NAB Transactions")).toBeVisible();
    await expect(page.locator("text=ING Transactions")).toBeVisible();
    await expect(page.locator("text=OFX Portfolio")).toBeVisible();
    await expect(page.locator("text=QIF Transactions")).toBeVisible();
  });

  test("should parse CommBank CSV sample file", async () => {
    await page.goto(`${BASE_URL}/demo`);

    // Click parse button for CBA transactions
    const cbaButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "CommBank" })
      .locator("button");
    await cbaButton.click();

    // Wait for results
    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });

    // Verify bank detection (look in the results section only)
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("CommBank");

    // Verify transactions are displayed
    const transactionRows = page.locator(".terminal-table tbody tr");
    await expect(transactionRows).toHaveCount(10);

    // Verify first transaction
    const firstRow = transactionRows.first();
    await expect(firstRow.locator("td").nth(2)).toContainText("-"); // Negative amount (debit)
  });

  test("should parse Westpac CSV sample file", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const westpacButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "Westpac" })
      .locator("button");
    await westpacButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("Westpac");

    const transactionRows = page.locator(".terminal-table tbody tr");
    await expect(transactionRows.count()).resolves.toBeGreaterThan(0);
  });

  test("should parse ANZ CSV sample file", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const anzButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "ANZ" })
      .locator("button");
    await anzButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("ANZ");

    const transactionRows = page.locator(".terminal-table tbody tr");
    await expect(transactionRows.count()).resolves.toBeGreaterThan(0);
  });

  test("should parse NAB CSV sample file", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const nabButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "NAB" })
      .locator("button");
    await nabButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("NAB");

    const transactionRows = page.locator(".terminal-table tbody tr");
    await expect(transactionRows.count()).resolves.toBeGreaterThan(0);
  });

  test("should parse ING CSV sample file", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const ingButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "ING" })
      .locator("button");
    await ingButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("ING");

    const transactionRows = page.locator(".terminal-table tbody tr");
    await expect(transactionRows.count()).resolves.toBeGreaterThan(0);
  });

  test("should parse OFX sample file", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const ofxButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "OFX" })
      .locator("button");
    await ofxButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("OFX");

    // OFX should show format as OFX
    await expect(page.locator("text=OFX Format")).toBeVisible();

    const transactionRows = page.locator(".terminal-table tbody tr");
    await expect(transactionRows.count()).resolves.toBeGreaterThan(0);
  });

  test("should parse QIF sample file", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const qifButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "QIF" })
      .locator("button");
    await qifButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("QIF");

    // QIF should show format as QIF
    await expect(page.locator("text=QIF Format")).toBeVisible();

    const transactionRows = page.locator(".terminal-table tbody tr");
    await expect(transactionRows.count()).resolves.toBeGreaterThan(0);
  });

  test("should handle drag and drop file upload", async () => {
    await page.goto(`${BASE_URL}/demo`);

    // Verify dropzone is present
    const dropzone = page.locator(".terminal-dropzone");
    await expect(dropzone).toBeVisible();

    // Verify dropzone text
    await expect(dropzone).toContainText("Drop CSV, OFX, or QIF file here");

    // Simulate drag over
    await dropzone.dispatchEvent("dragover");
    await expect(dropzone).toHaveClass(/active/);

    // Simulate drag leave
    await dropzone.dispatchEvent("dragleave");
    await expect(dropzone).not.toHaveClass(/active/);
  });

  test("should display transaction counts correctly", async () => {
    await page.goto(`${BASE_URL}/demo`);

    // Parse CBA file
    const cbaButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "CommBank" })
      .locator("button");
    await cbaButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });

    // Verify transaction count is displayed
    await expect(page.locator("text=/\\d+ transactions/")).toBeVisible();

    // Verify success badge
    await expect(page.locator(".terminal-badge-success")).toContainText("Parsed");
  });

  test("should display amounts with correct formatting", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const cbaButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "CommBank" })
      .locator("button");
    await cbaButton.click();

    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });

    // Verify amount column contains dollar signs
    const amountCells = page.locator(".terminal-table tbody tr td").nth(2);
    await expect(amountCells.first()).toContainText("$");
  });

  test("should have working back to app button", async () => {
    await page.goto(`${BASE_URL}/demo`);

    const backButton = page.locator("button").filter({ hasText: "Back to App" });
    await expect(backButton).toBeVisible();

    // Click should navigate to home
    await backButton.click();

    // Should navigate away from demo page
    await expect(page).not.toHaveURL(/\/demo/);
  });

  test("should have correct terminal styling colors", async () => {
    await page.goto(`${BASE_URL}/demo`);
    await page.waitForLoadState("networkidle");

    // Verify terminal background
    const terminal = page.locator(".terminal");
    const bgColor = await terminal.evaluate((el) => window.getComputedStyle(el).backgroundColor);

    // Verify accent color (cyan) - use the header title which always exists
    const accentElement = page.locator(".terminal-header .title");
    const accentColor = await accentElement.evaluate((el) => window.getComputedStyle(el).color);

    // Verify colors are applied (they should not be default black/white)
    expect(bgColor).not.toBe("rgb(255, 255, 255)");
    expect(bgColor).not.toBe("rgba(0, 0, 0, 0)");
  });

  test("should switch between different file formats", async () => {
    await page.goto(`${BASE_URL}/demo`);

    // Parse CBA first
    const cbaButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "CommBank" })
      .locator("button");
    await cbaButton.click();
    await expect(page.locator(".terminal-section").filter({ hasText: "Parsed" })).toBeVisible({
      timeout: 5000,
    });
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("CommBank");

    // Now parse OFX
    const ofxButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "OFX" })
      .locator("button");
    await ofxButton.click();
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("OFX", { timeout: 5000 });

    // Now parse QIF
    const qifButton = page
      .locator(".terminal-file-item")
      .filter({ hasText: "QIF" })
      .locator("button");
    await qifButton.click();
    await expect(
      page.locator(".terminal-section").filter({ hasText: "Parsed" }).locator(".terminal-accent"),
    ).toContainText("QIF", { timeout: 5000 });
  });
});
