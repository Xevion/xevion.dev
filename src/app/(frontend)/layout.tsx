import React from "react";

// Fontsource imports
import "@fontsource-variable/inter";
import "@fontsource-variable/roboto";
import "@fontsource-variable/roboto-mono";
import "@fontsource/hanken-grotesk/900.css";
import "@fontsource/schibsted-grotesk/400.css";
import "@fontsource/schibsted-grotesk/500.css";
import "@fontsource/schibsted-grotesk/600.css";

import "@radix-ui/themes/styles.css";
import "@/styles/globals.css";
import type { Metadata } from "next";
import { Providers } from "./providers";

export const metadata: Metadata = {
  title: "Xevion.dev",
  description:
    "The personal website of Xevion, a full-stack software developer.",
  applicationName: "xevion.dev",
};

export default async function RootLayout(props: { children: React.ReactNode }) {
  const { children } = props;

  return (
    <html lang="en">
      <body>
        <Providers>
          <main>{children}</main>
        </Providers>
      </body>
    </html>
  );
}
