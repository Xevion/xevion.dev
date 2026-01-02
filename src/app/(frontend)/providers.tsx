"use client";

import { Analytics } from "@vercel/analytics/react";
import { Provider as BalancerProvider } from "react-wrap-balancer";
import { Theme } from "@radix-ui/themes";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    // @ts-expect-error - Radix UI Themes has React 19 type compatibility issues
    <Theme appearance="dark">
      <BalancerProvider>
        {children}
        <Analytics />
      </BalancerProvider>
    </Theme>
  );
}
