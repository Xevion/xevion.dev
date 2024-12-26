import { type AppType } from "next/app";

import { trpc } from "@/utils/trpc";

import "@/styles/globals.scss";
import { Analytics } from "@vercel/analytics/react";
import { Provider } from "react-wrap-balancer";
import { Metadata } from "next";
  
export const metadata: Metadata = {
  title: "Xevion.dev",
  description: "The personal website of Xevion, a full-stack software developer.",
  applicationName: "xevion.dev",
}

const MyApp: AppType = ({ Component, pageProps }) => {
  return (
    <>
      <Provider>
        <Component {...pageProps} />
      </Provider>
      <Analytics />
    </>
  );
};

export default trpc.withTRPC(MyApp);
