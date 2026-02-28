import { Building2 } from "lucide-react";
import { Button } from "@sensible-folio/ui/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@sensible-folio/ui/components/ui/dialog";

import type { NewAccountCreatedPayload } from "@/adapters";

interface NewAccountsModalProps {
  accounts: NewAccountCreatedPayload[];
  onDismiss: () => void;
}

export function NewAccountsModal({ accounts, onDismiss }: NewAccountsModalProps) {
  if (accounts.length === 0) return null;

  return (
    <Dialog open onOpenChange={onDismiss}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New accounts detected</DialogTitle>
          <DialogDescription>
            The following accounts were found and added automatically. You can rename them in
            Settings.
          </DialogDescription>
        </DialogHeader>
        <ul className="space-y-2 py-2">
          {accounts.map((a) => (
            <li key={a.accountId} className="flex items-center gap-2 text-sm">
              <Building2 className="text-muted-foreground h-4 w-4" />
              <span className="font-medium">{a.accountName}</span>
              <span className="text-muted-foreground font-mono text-xs">
                &bull;&bull;&bull;{a.accountNumber.replace(/\s/g, "").slice(-4)}
              </span>
            </li>
          ))}
        </ul>
        <DialogFooter>
          <Button onClick={onDismiss}>Got it</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
