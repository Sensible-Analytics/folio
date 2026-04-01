import { useState, useCallback } from "react";
import { Button } from "@sensible-folio/ui";
import { useNavigate } from "react-router-dom";

interface ParsedTransaction {
  date: string;
  description: string;
  amount: string;
  type: string;
}

interface ParseResult {
  bank: string;
  format: string;
  transactions: ParsedTransaction[];
  success: boolean;
}

const SAMPLE_FILES = [
  { name: "CommBank Transactions", file: "/samples/cba-transactions.csv", bank: "CBA" },
  { name: "Westpac Transactions", file: "/samples/westpac-transactions.csv", bank: "Westpac" },
  { name: "ANZ Transactions", file: "/samples/anz-transactions.csv", bank: "ANZ" },
  { name: "NAB Transactions", file: "/samples/nab-transactions.csv", bank: "NAB" },
  { name: "ING Transactions", file: "/samples/ing-transactions.csv", bank: "ING" },
  { name: "OFX Portfolio", file: "/samples/sample-portfolio.ofx", bank: "OFX" },
  { name: "QIF Transactions", file: "/samples/sample-transactions.qif", bank: "QIF" },
];

export default function DemoLandingPage() {
  const navigate = useNavigate();
  const [isDragging, setIsDragging] = useState(false);
  const [parseResults, setParseResults] = useState<ParseResult[]>([]);

  const detectBank = (content: string, filename: string): { bank: string; format: string } => {
    const ext = filename.split(".").pop()?.toLowerCase() || "";

    if (ext === "ofx" || content.includes("OFXHEADER")) {
      return { bank: "OFX", format: "OFX" };
    }
    if (ext === "qif" || content.startsWith("!Type:")) {
      return { bank: "QIF", format: "QIF" };
    }

    const lowerContent = content.toLowerCase();
    if (
      lowerContent.includes("commbank") ||
      lowerContent.includes("cba") ||
      filename.includes("cba")
    ) {
      return { bank: "CommBank (CBA)", format: "CSV" };
    }
    if (lowerContent.includes("westpac") || filename.includes("westpac")) {
      return { bank: "Westpac", format: "CSV" };
    }
    if (lowerContent.includes("anz") || filename.includes("anz")) {
      return { bank: "ANZ", format: "CSV" };
    }
    if (lowerContent.includes("nab") || filename.includes("nab")) {
      return { bank: "NAB", format: "CSV" };
    }
    if (lowerContent.includes("ing") || filename.includes("ing")) {
      return { bank: "ING", format: "CSV" };
    }

    return { bank: "Unknown", format: "CSV" };
  };

  const parseCSV = (content: string): ParsedTransaction[] => {
    const lines = content.trim().split("\n");
    const headers = lines[0].split(",").map((h) => h.trim().toLowerCase());

    const dateIdx = headers.findIndex((h) => h.includes("date") || h.includes("time"));
    const descIdx = headers.findIndex(
      (h) =>
        h.includes("desc") ||
        h.includes("memo") ||
        h.includes("narration") ||
        h.includes("particulars"),
    );
    const amountIdx = headers.findIndex(
      (h) => h.includes("amount") || h.includes("debit") || h.includes("credit"),
    );
    const typeIdx = headers.findIndex((h) => h.includes("type") || h.includes("transaction"));

    return lines
      .slice(1)
      .map((line) => {
        const cols = line.split(",").map((c) => c.trim().replace(/"/g, ""));
        return {
          date: cols[dateIdx >= 0 ? dateIdx : 0] || "",
          description: cols[descIdx >= 0 ? descIdx : 1] || "",
          amount: cols[amountIdx >= 0 ? amountIdx : 2] || "",
          type: cols[typeIdx >= 0 ? typeIdx : 3] || "DEBIT",
        };
      })
      .filter((t) => t.date || t.description);
  };

  const handleFileSelect = useCallback(async (file: File | string) => {
    let content: string;
    if (typeof file === "string") {
      const response = await fetch(file);
      content = await response.text();
    } else {
      content = await file.text();
    }

    const { bank, format } = detectBank(content, typeof file === "string" ? file : file.name);

    let transactions: ParsedTransaction[] = [];

    if (format === "CSV") {
      transactions = parseCSV(content);
    } else if (format === "OFX") {
      const stmtMatches = content.match(/<STMTTRN>([\s\S]*?)<\/STMTTRN>/gi) || [];
      transactions = stmtMatches.map((tr) => {
        const dtposted = /<DTPOSTED>(\d+)/.exec(tr)?.[1] ?? "";
        const trnamt = /<TRNAMT>([+-]?[\d.]+)/.exec(tr)?.[1] ?? "";
        const name = /<NAME>([^<]+)/.exec(tr)?.[1] ?? "";
        const trntype = /<TRNTYPE>([^<]+)/.exec(tr)?.[1] ?? "";

        const year = dtposted.slice(0, 4);
        const month = dtposted.slice(4, 6);
        const day = dtposted.slice(6, 8);

        return {
          date: `${day}/${month}/${year}`,
          description: name,
          amount: trnamt,
          type: trntype,
        };
      });
    } else if (format === "QIF") {
      const entries = content.split("^").filter((e) => e.trim());
      transactions = entries
        .map((entry) => {
          const lines = entry.trim().split("\n");
          let date = "",
            amount = "",
            payee = "",
            type = "";

          lines.forEach((line) => {
            const code = line[0];
            const value = line.slice(1);
            if (code === "D") date = value;
            if (code === "T" || code === "U") amount = value;
            if (code === "P") payee = value;
            if (code === "L") type = value;
          });

          return { date, description: payee, amount, type };
        })
        .filter((t) => t.date);
    }

    setParseResults([
      {
        bank,
        format,
        transactions,
        success: transactions.length > 0,
      },
    ]);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragging(false);

      const file = e.dataTransfer.files[0];
      if (file) {
        handleFileSelect(file);
      }
    },
    [handleFileSelect],
  );

  return (
    <div className="terminal">
      <div className="terminal-header">
        <div className="dots">
          <span className="dot dot-red" />
          <span className="dot dot-yellow" />
          <span className="dot dot-green" />
        </div>
        <span className="title">proprro.sensibleanalytics.co</span>
        <div />
      </div>

      <div className="terminal-body">
        <div className="terminal-prompt">
          <span className="user">prabhat</span>
          <span className="at">@</span>
          <span className="host">sensible</span>
          <span className="path">:~</span>
          <span className="dollar">$</span>
          <span className="cursor" />
        </div>

        <h1 className="mb-2 text-2xl font-bold">
          <span className="terminal-accent">Australian Bank Statement Parser</span>
        </h1>
        <p className="terminal-muted mb-6">Parse CSV, OFX, and QIF files from Australian banks</p>

        <div className="terminal-section">
          <div className="terminal-section-title">Sample Files</div>
          <div className="terminal-file-list mb-6">
            {SAMPLE_FILES.map((sample) => (
              <div key={sample.file} className="terminal-file-item">
                <div>
                  <span className="terminal-badge terminal-badge-info mr-2">{sample.bank}</span>
                  <span className="name">{sample.name}</span>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  className="terminal-btn"
                  onClick={() => handleFileSelect(sample.file)}
                >
                  Parse
                </Button>
              </div>
            ))}
          </div>
        </div>

        <hr className="terminal-divider" />

        <div className="terminal-section">
          <div className="terminal-section-title">Upload File</div>
          <div
            className={`terminal-dropzone ${isDragging ? "active" : ""}`}
            onDragOver={(e) => {
              e.preventDefault();
              setIsDragging(true);
            }}
            onDragLeave={() => setIsDragging(false)}
            onDrop={handleDrop}
            onClick={() => {
              const input = document.createElement("input");
              input.type = "file";
              input.accept = ".csv,.ofx,.qif";
              input.onchange = (e) => {
                const file = (e.target as HTMLInputElement).files?.[0];
                if (file) handleFileSelect(file);
              };
              input.click();
            }}
          >
            <div className="terminal-dropzone-icon">📁</div>
            <div className="terminal-muted">
              Drop CSV, OFX, or QIF file here, or click to browse
            </div>
          </div>
        </div>

        {parseResults.map((result, idx) => (
          <div key={idx} className="terminal-section">
            <hr className="terminal-divider" />

            <div className="mb-4 flex items-center gap-4">
              <span className="terminal-badge terminal-badge-success">
                {result.success ? "✓ Parsed" : "✗ Failed"}
              </span>
              <span className="terminal-accent">{result.bank}</span>
              <span className="terminal-muted">•</span>
              <span className="terminal-muted">{result.format} Format</span>
              <span className="terminal-muted">•</span>
              <span className="terminal-muted">{result.transactions.length} transactions</span>
            </div>

            {result.transactions.length > 0 && (
              <div className="terminal-output">
                <table className="terminal-table">
                  <thead>
                    <tr>
                      <th>Date</th>
                      <th>Description</th>
                      <th>Amount</th>
                      <th>Type</th>
                    </tr>
                  </thead>
                  <tbody>
                    {result.transactions.slice(0, 10).map((tx, i) => (
                      <tr key={i}>
                        <td>{tx.date}</td>
                        <td className="max-w-xs truncate">{tx.description}</td>
                        <td
                          className={
                            parseFloat(tx.amount) >= 0 ? "terminal-success" : "terminal-error"
                          }
                        >
                          ${parseFloat(tx.amount).toFixed(2)}
                        </td>
                        <td>{tx.type}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {result.transactions.length > 10 && (
                  <p className="terminal-muted mt-2 text-sm">
                    ... and {result.transactions.length - 10} more transactions
                  </p>
                )}
              </div>
            )}
          </div>
        ))}

        <hr className="terminal-divider" />

        <div className="flex gap-4">
          <Button
            variant="outline"
            size="lg"
            className="terminal-btn"
            onClick={() => navigate("/")}
          >
            ← Back to App
          </Button>
        </div>
      </div>
    </div>
  );
}
