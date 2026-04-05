import { ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { AuthGate } from "@/context/auth-context";
import { LoginPage } from "@/pages/auth/login-page";

const PUBLIC_ROUTES = ["/demo", "/auth/callback"];

export function ProtectedRoutes({ children }: { children: ReactNode }) {
  const location = useLocation();

  const isPublicRoute = PUBLIC_ROUTES.some((route) => location.pathname.startsWith(route));

  if (isPublicRoute) {
    return <>{children}</>;
  }

  return <AuthGate fallback={<LoginPage />}>{children}</AuthGate>;
}
