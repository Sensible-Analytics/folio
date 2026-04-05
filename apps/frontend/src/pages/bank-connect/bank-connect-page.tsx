import { useCallback, useEffect, useRef, useState } from "react";

import { Badge } from "@sensible-folio/ui/components/ui/badge";
import { Button } from "@sensible-folio/ui/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@sensible-folio/ui/components/ui/card";
import { ScrollArea } from "@sensible-folio/ui/components/ui/scroll-area";
import { cn } from "@sensible-folio/ui/lib/utils";
import { Building2, LineChart, Lock, LogIn, X } from "lucide-react";

import {
  closeBankPanel,
  getBankConnectSettings,
  listBankDownloadRuns,
  listenBankDownloadComplete,
  listenBankImportComplete,
  listenBankLoginDetected,
  listenBankNewAccountCreated,
  listenBankProgress,
  listenBankWindowClosed,
  openBankPanel,
  resizeBankPanel,
  startBankDownload,
} from "@/adapters";

import type {
  BankConnectSettings,
  BankDownloadCompletePayload,
  BankDownloadRun,
  BankLoginDetectedPayload,
  BankProgressPayload,
  BankWindowClosedPayload,
  ImportCompletePayload,
  NewAccountCreatedPayload,
} from "@/adapters";

import { NewAccountsModal } from "./new-accounts-modal";

// ============================================================================
// Types
// ============================================================================

interface LogEntry {
  id: string;
  timestamp: string;
  level: "info" | "warn" | "error" | "success";
  message: string;
  bankKey: string;
}

type ConnectorStatus = "idle" | "window-open" | "logged-in" | "downloading" | "complete" | "error";

interface ConnectorInfo {
  key: string;
  displayName: string;
  badge?: string; // e.g. "Beta"
  comingSoon?: boolean;
}

// ============================================================================
// Constants
// ============================================================================

const BANKS: ConnectorInfo[] = [
  { key: "ING", displayName: "ING" },
  { key: "CBA", displayName: "CommBank" },
  { key: "ANZ", displayName: "ANZ" },
  { key: "BOM", displayName: "Bank of Melbourne" },
  { key: "BEYOND", displayName: "Beyond Bank" },
];

const BROKERS: ConnectorInfo[] = [
  { key: "IBKR", displayName: "Interactive Brokers", badge: "Beta" },
  { key: "COMMSEC", displayName: "CommSec", comingSoon: true },
  { key: "SHARESIES", displayName: "Sharesies", comingSoon: true },
  { key: "SELFWEALTH", displayName: "SelfWealth", comingSoon: true },
  { key: "STAKE", displayName: "Stake", comingSoon: true },
];

// ============================================================================
// Helpers
// ============================================================================

function statusLabel(status: ConnectorStatus): string {
  switch (status) {
    case "idle":
      return "Idle";
    case "window-open":
      return "Awaiting Login";
    case "logged-in":
      return "Logged In";
    case "downloading":
      return "Downloading...";
    case "complete":
      return "Complete";
    case "error":
      return "Error";
  }
}

function statusVariant(
  status: ConnectorStatus,
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "idle":
      return "outline";
    case "window-open":
      return "secondary";
    case "logged-in":
    case "downloading":
    case "complete":
      return "default";
    case "error":
      return "destructive";
  }
}

const VALID_LEVELS = ["info", "warn", "error", "success"] as const;
type LogLevel = (typeof VALID_LEVELS)[number];

function toLogLevel(s: string): LogLevel {
  return (VALID_LEVELS as readonly string[]).includes(s) ? (s as LogLevel) : "info";
}

function levelColor(level: string): string {
  switch (level) {
    case "error":
      return "text-destructive";
    case "warn":
      return "text-yellow-500";
    case "success":
      return "text-green-500";
    default:
      return "text-muted-foreground";
  }
}

function latestRunsByBank(runs: BankDownloadRun[]): Record<string, BankDownloadRun> {
  const byBank: Record<string, BankDownloadRun> = {};
  for (const run of runs) {
    if (!byBank[run.bankKey] || run.startedAt > byBank[run.bankKey].startedAt) {
      byBank[run.bankKey] = run;
    }
  }
  return byBank;
}

// ============================================================================
// ConnectorCard Component
// ============================================================================

interface ConnectorCardProps {
  connector: ConnectorInfo;
  status: ConnectorStatus;
  lastRun: BankDownloadRun | null;
  icon: React.ReactNode;
  onOpenLogin: () => void;
  onClose: () => void;
}

function ConnectorCard({
  connector,
  status,
  lastRun,
  icon,
  onOpenLogin,
  onClose,
}: ConnectorCardProps) {
  if (connector.comingSoon) {
    return (
      <Card className="opacity-60">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              {icon}
              <CardTitle className="text-base">{connector.displayName}</CardTitle>
            </div>
            <Badge variant="outline" className="text-muted-foreground gap-1">
              <Lock className="h-3 w-3" />
              Coming Soon
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-xs">
            Available as a plugin in a future release.
          </p>
        </CardContent>
      </Card>
    );
  }

  const isActive = status === "window-open" || status === "logged-in" || status === "downloading";

  return (
    <Card className={cn("transition-all", isActive && "ring-primary ring-2")}>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {icon}
            <CardTitle className="text-base">{connector.displayName}</CardTitle>
            {connector.badge && (
              <Badge variant="secondary" className="text-xs">
                {connector.badge}
              </Badge>
            )}
          </div>
          <Badge variant={statusVariant(status)}>{statusLabel(status)}</Badge>
        </div>
      </CardHeader>
      <CardContent>
        {lastRun && (
          <p className="text-muted-foreground mb-3 text-xs">
            Last run: {lastRun.filesDownloaded} downloaded, {lastRun.filesSkipped} skipped
          </p>
        )}
        <div className="flex flex-wrap gap-2">
          {(status === "idle" || status === "complete" || status === "error") && (
            <Button size="sm" variant="outline" onClick={onOpenLogin}>
              <LogIn className="mr-1 h-4 w-4" />
              Open Login
            </Button>
          )}
          {(status === "window-open" || status === "logged-in") && (
            <Button size="sm" variant="ghost" onClick={onClose}>
              <X className="mr-1 h-4 w-4" />
              Close
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

// ============================================================================
// BankConnectPage
// ============================================================================

export default function BankConnectPage() {
  const [statuses, setStatuses] = useState<Record<string, ConnectorStatus>>({});
  const [lastRuns, setLastRuns] = useState<Record<string, BankDownloadRun | null>>({});
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [settings, setSettings] = useState<BankConnectSettings | null>(null);
  const [newAccounts, setNewAccounts] = useState<NewAccountCreatedPayload[]>([]);
  const [dismissedAccountIds, setDismissedAccountIds] = useState<Set<string>>(new Set());
  const [activeBankKey, setActiveBankKey] = useState<string | null>(null);
  const logEndRef = useRef<HTMLDivElement>(null);
  const logIdCounter = useRef(0);
  const panelRef = useRef<HTMLDivElement>(null);
  const activeBankRef = useRef<string | null>(null);

  const addLog = useCallback((bankKey: string, level: LogEntry["level"], message: string) => {
    const entry: LogEntry = {
      id: String(logIdCounter.current++),
      timestamp: new Date().toISOString(),
      level,
      message,
      bankKey,
    };
    setLogs((prev) => [...prev.slice(-499), entry]);
  }, []);

  // Load initial data
  useEffect(() => {
    getBankConnectSettings().then(setSettings).catch(console.error);
    listBankDownloadRuns()
      .then((runs) => setLastRuns(latestRunsByBank(runs)))
      .catch(console.error);
  }, []);

  // Subscribe to Tauri events
  useEffect(() => {
    let mounted = true;
    const unlisteners: (() => Promise<void>)[] = [];

    (async () => {
      const uls = await Promise.all([
        listenBankLoginDetected((event: { payload: BankLoginDetectedPayload }) => {
          const key = event.payload.bankKey;
          setStatuses((prev) => ({ ...prev, [key]: "logged-in" }));
          addLog(key, "success", "Login detected — starting automation...");
          // Auto-start download
          startBankDownload(key).catch((err) =>
            addLog(key, "error", `Automation failed: ${String(err)}`),
          );
        }),
        listenBankProgress((event: { payload: BankProgressPayload }) => {
          addLog(event.payload.bankKey, toLogLevel(event.payload.level), event.payload.message);
        }),
        listenBankDownloadComplete((event: { payload: BankDownloadCompletePayload }) => {
          setStatuses((prev) => ({ ...prev, [event.payload.bankKey]: "complete" }));
          addLog(
            event.payload.bankKey,
            "success",
            `Download complete: ${event.payload.downloaded} files`,
          );
          listBankDownloadRuns()
            .then((runs) => setLastRuns(latestRunsByBank(runs)))
            .catch(console.error);
        }),
        listenBankWindowClosed((event: { payload: BankWindowClosedPayload }) => {
          setStatuses((prev) => {
            const current = prev[event.payload.bankKey];
            if (current === "window-open" || current === "logged-in") {
              return { ...prev, [event.payload.bankKey]: "idle" };
            }
            return prev;
          });
          addLog(event.payload.bankKey, "info", `${event.payload.bankKey} window closed`);
        }),
        listenBankImportComplete((event: { payload: ImportCompletePayload }) => {
          const { bankKey, newCount, skippedCount } = event.payload;
          setStatuses((prev) => ({ ...prev, [bankKey]: "complete" }));
          addLog(bankKey, "success", `Import complete: ${newCount} new, ${skippedCount} skipped`);
          listBankDownloadRuns()
            .then((runs) => setLastRuns(latestRunsByBank(runs)))
            .catch(console.error);
        }),
        listenBankNewAccountCreated((event: { payload: NewAccountCreatedPayload }) => {
          setNewAccounts((prev) => {
            // Avoid duplicates
            if (prev.some((a) => a.accountId === event.payload.accountId)) return prev;
            return [...prev, event.payload];
          });
        }),
      ]);
      if (!mounted) {
        uls.forEach((ul) => ul());
        return;
      }
      unlisteners.push(...uls);
    })();

    return () => {
      mounted = false;
      unlisteners.forEach((ul) => ul());
    };
  }, [addLog]);

  // Auto-scroll logs
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  // ResizeObserver to keep panel window in sync
  useEffect(() => {
    if (!panelRef.current) return;
    const observer = new ResizeObserver(() => {
      if (!activeBankRef.current || !panelRef.current) return;
      const r = panelRef.current.getBoundingClientRect();
      resizeBankPanel(activeBankRef.current, {
        x: r.left,
        y: r.top,
        width: r.width,
        height: r.height,
      }).catch(console.error);
    });
    observer.observe(panelRef.current);
    return () => observer.disconnect();
  }, []);

  const handleOpenLogin = async (key: string) => {
    if (!panelRef.current) return;
    const r = panelRef.current.getBoundingClientRect();
    activeBankRef.current = key;
    setActiveBankKey(key);
    try {
      await openBankPanel(key, { x: r.left, y: r.top, width: r.width, height: r.height });
      setStatuses((prev) => ({ ...prev, [key]: "window-open" }));
      addLog(key, "info", `Opening ${key} — please log in...`);
    } catch (err) {
      addLog(key, "error", `Failed to open panel: ${String(err)}`);
    }
  };

  const handleClose = async (key: string) => {
    try {
      await closeBankPanel(key);
      activeBankRef.current = null;
      setActiveBankKey(null);
    } catch (err) {
      addLog(key, "error", `Failed to close ${key} panel: ${String(err)}`);
    }
  };

  const connectorCardProps = (connector: ConnectorInfo, icon: React.ReactNode) => ({
    connector,
    status: statuses[connector.key] ?? "idle",
    lastRun: lastRuns[connector.key] ?? null,
    icon,
    onOpenLogin: () => handleOpenLogin(connector.key),
    onClose: () => handleClose(connector.key),
  });

  return (
    <div className="flex h-full flex-col gap-4 p-6">
      <div>
        <h1 className="text-2xl font-bold">Connect</h1>
        <p className="text-muted-foreground text-sm">
          Download statements from Australian banks and brokers directly into Wealthfolio
        </p>
      </div>

      <div className="flex min-h-0 flex-1 gap-4">
        {/* Left: connector cards */}
        <ScrollArea className="w-80 shrink-0">
          <div className="flex flex-col gap-4 pr-2">
            {/* Banks */}
            <div>
              <p className="text-muted-foreground mb-2 text-xs font-semibold tracking-wide uppercase">
                Banks
              </p>
              <div className="flex flex-col gap-2">
                {BANKS.map((bank) => (
                  <ConnectorCard
                    key={bank.key}
                    {...connectorCardProps(
                      bank,
                      <Building2 className="text-muted-foreground h-5 w-5" />,
                    )}
                  />
                ))}
              </div>
            </div>

            {/* Brokers */}
            <div>
              <p className="text-muted-foreground mb-2 text-xs font-semibold tracking-wide uppercase">
                Brokers
              </p>
              <div className="flex flex-col gap-2">
                {BROKERS.map((broker) => (
                  <ConnectorCard
                    key={broker.key}
                    {...connectorCardProps(
                      broker,
                      <LineChart className="text-muted-foreground h-5 w-5" />,
                    )}
                  />
                ))}
              </div>
            </div>
          </div>
        </ScrollArea>

        {/* Right: panel area + log */}
        <div className="flex flex-1 flex-col gap-2 overflow-hidden">
          {/* Bank panel placeholder */}
          <div
            ref={panelRef}
            className="bg-muted/20 flex flex-[3] items-center justify-center rounded-lg border"
          >
            {!activeBankKey && (
              <p className="text-muted-foreground text-sm">
                Select a bank and log in — the bank site will open in a separate window
              </p>
            )}
          </div>

          {/* Log Panel */}
          <div className="flex flex-[2] flex-col overflow-hidden rounded-lg border">
            <div className="bg-muted/30 flex items-center justify-between border-b px-3 py-2">
              <span className="text-sm font-medium">Activity Log</span>
              <Button variant="ghost" size="sm" onClick={() => setLogs([])} className="h-7 text-xs">
                Clear
              </Button>
            </div>
            <ScrollArea className="flex-1 p-3">
              {logs.length === 0 ? (
                <p className="text-muted-foreground py-8 text-center text-xs">
                  Log messages will appear here...
                </p>
              ) : (
                <div className="space-y-1 font-mono text-xs">
                  {logs.map((entry) => (
                    <div key={entry.id} className="flex gap-2">
                      <span className="text-muted-foreground shrink-0">
                        {new Date(entry.timestamp).toLocaleTimeString()}
                      </span>
                      <span className="text-muted-foreground w-14 shrink-0">[{entry.bankKey}]</span>
                      <span className={cn("w-14 shrink-0", levelColor(entry.level))}>
                        [{entry.level}]
                      </span>
                      <span>{entry.message}</span>
                    </div>
                  ))}
                  <div ref={logEndRef} />
                </div>
              )}
            </ScrollArea>
          </div>
        </div>
      </div>

      {/* Bottom bar */}
      {settings && (
        <div className="text-muted-foreground border-t pt-3 text-xs">
          Download folder: <span className="font-mono">{settings.downloadFolder}</span>
          {" · "}
          {settings.yearsBack} years back
        </div>
      )}

      <NewAccountsModal
        accounts={newAccounts.filter((a) => !dismissedAccountIds.has(a.accountId))}
        onDismiss={() => {
          setDismissedAccountIds(new Set(newAccounts.map((a) => a.accountId)));
        }}
      />
    </div>
  );
}
