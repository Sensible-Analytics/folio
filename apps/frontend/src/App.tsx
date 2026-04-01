import { isWeb } from "@/adapters";
import { AuthProvider } from "@/context/auth-context";
import { WealthfolioConnectProvider } from "@/features/wealthfolio-connect";
import { DeviceSyncProvider } from "@/features/devices-sync";
import { SettingsProvider } from "@/lib/settings-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@sensible-folio/ui";
import { useState } from "react";
import { BrowserRouter } from "react-router-dom";
import { PrivacyProvider } from "./context/privacy-context";
import { AppRoutes } from "./routes";
import { ProtectedRoutes } from "./protected-routes";

function App() {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            refetchOnWindowFocus: false,
            staleTime: 5 * 60 * 1000,
            retry: false,
          },
        },
      }),
  );

  // Make QueryClient available globally for addons
  window.__wealthfolio_query_client__ = queryClient;

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <WealthfolioConnectProvider>
          <DeviceSyncProvider>
            <PrivacyProvider>
              <SettingsProvider>
                <TooltipProvider>
                  <BrowserRouter>
                    {isWeb ? (
                      <ProtectedRoutes>
                        <AppRoutes />
                      </ProtectedRoutes>
                    ) : (
                      <AppRoutes />
                    )}
                  </BrowserRouter>
                </TooltipProvider>
              </SettingsProvider>
            </PrivacyProvider>
          </DeviceSyncProvider>
        </WealthfolioConnectProvider>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
