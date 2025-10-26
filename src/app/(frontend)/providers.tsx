"use client";

import { Analytics } from "@vercel/analytics/react";
import { Provider as BalancerProvider } from "react-wrap-balancer";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <BalancerProvider>
      {children}
      <Analytics />
    </BalancerProvider>
  );
}
