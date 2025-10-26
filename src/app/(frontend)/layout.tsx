import React from "react";
import "@/styles/globals.scss";
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
