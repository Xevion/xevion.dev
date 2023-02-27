import { type AppType } from "next/app";

import { trpc } from "../utils/trpc";

import "../styles/globals.scss";
import { Analytics } from "@vercel/analytics/react";
import { Provider } from "react-wrap-balancer";
import Head from "next/head";
import { OGP } from "react-ogp";

const MyApp: AppType = ({ Component, pageProps }) => {
  return (
    <>
      <Head>
        <OGP
          url="https://xevion.dev"
          title="Xevion.dev"
          description="The personal website of Xevion, a full-stack software developer."
          siteName="xevion.dev"
          image="https://xevion.dev/banner.png"
        />
      </Head>
      <Provider>
        <Component {...pageProps} />
      </Provider>
      <Analytics />
    </>
  );
};

export default trpc.withTRPC(MyApp);
