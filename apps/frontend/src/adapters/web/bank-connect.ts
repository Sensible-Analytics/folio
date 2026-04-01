// Web adapter - Bank Connect Commands
// These are stubs for desktop-only bank panel operations
// In web mode, bank panels are not available

// Interfaces (matching tauri adapter for compatibility)
export interface BankConnectSettings {
  downloadFolder: string;
  yearsBack: number;
  enabledBanks: string[];
  overwriteFiles: boolean;
}

export interface BankDownloadRun {
  id: string;
  bankKey: string;
  startedAt: string;
  completedAt: string | null;
  status: "running" | "completed" | "failed";
  filesDownloaded: number;
}

export interface BankLoginDetectedPayload {
  bankKey: string;
}

export interface BankProgressPayload {
  bankKey: string;
  message: string;
  progress: number;
}

export interface BankDownloadCompletePayload {
  bankKey: string;
  filesDownloaded: number;
}

export interface BankWindowClosedPayload {
  bankKey: string;
}

export interface ImportCompletePayload {
  bankKey: string;
  activitiesImported: number;
}

export interface NewAccountCreatedPayload {
  bankKey: string;
  accountId: string;
}

// Stub: Get bank connect settings
export const getBankConnectSettings = async (): Promise<BankConnectSettings> => {
  return {
    downloadFolder: "",
    yearsBack: 7,
    enabledBanks: [],
    overwriteFiles: false,
  };
};

// Stub: Save bank connect settings
export const saveBankConnectSettings = async (_settings: BankConnectSettings): Promise<void> => {
  console.warn("saveBankConnectSettings is not available in web mode");
};

// Stub: List bank download runs
export const listBankDownloadRuns = async (_bankKey?: string): Promise<BankDownloadRun[]> => {
  return [];
};

// Stub: Open bank window
export const openBankWindow = async (_bankKey: string): Promise<void> => {
  console.warn("openBankWindow is not available in web mode");
};

// Stub: Close bank window
export const closeBankWindow = async (_bankKey: string): Promise<void> => {
  console.warn("closeBankWindow is not available in web mode");
};

// Stub: Start bank download
export const startBankDownload = async (_bankKey: string): Promise<string> => {
  throw new Error("Bank download is not available in web mode");
};

// Stub: Listen for bank login detected
export async function listenBankLoginDetected(
  _callback: (payload: BankLoginDetectedPayload) => void,
): Promise<() => void> {
  console.warn("listenBankLoginDetected is not available in web mode");
  return () => {};
}

// Stub: Listen for bank progress
export async function listenBankProgress(
  _callback: (payload: BankProgressPayload) => void,
): Promise<() => void> {
  console.warn("listenBankProgress is not available in web mode");
  return () => {};
}

// Stub: Listen for bank download complete
export async function listenBankDownloadComplete(
  _callback: (payload: BankDownloadCompletePayload) => void,
): Promise<() => void> {
  console.warn("listenBankDownloadComplete is not available in web mode");
  return () => {};
}

// Stub: Listen for bank window closed
export async function listenBankWindowClosed(
  _callback: (payload: BankWindowClosedPayload) => void,
): Promise<() => void> {
  console.warn("listenBankWindowClosed is not available in web mode");
  return () => {};
}

// Stub: Open bank panel
export const openBankPanel = async (
  _bankKey: string,
  _bounds: { x: number; y: number; width: number; height: number },
): Promise<void> => {
  console.warn("openBankPanel is not available in web mode");
};

// Stub: Close bank panel
export const closeBankPanel = async (_bankKey: string): Promise<void> => {
  console.warn("closeBankPanel is not available in web mode");
};

// Stub: Resize bank panel
export const resizeBankPanel = async (
  _bankKey: string,
  _bounds: { x: number; y: number; width: number; height: number },
): Promise<void> => {
  console.warn("resizeBankPanel is not available in web mode");
};

// Stub: Listen for bank import complete
export async function listenBankImportComplete(
  _callback: (payload: ImportCompletePayload) => void,
): Promise<() => void> {
  console.warn("listenBankImportComplete is not available in web mode");
  return () => {};
}

// Stub: Listen for new account created
export async function listenBankNewAccountCreated(
  _callback: (payload: NewAccountCreatedPayload) => void,
): Promise<() => void> {
  console.warn("listenBankNewAccountCreated is not available in web mode");
  return () => {};
}
