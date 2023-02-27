import { type AppType } from "next/app";

import { trpc } from "../utils/trpc";

import "../styles/globals.scss";
import { Analytics } from "@vercel/analytics/react";
import { Provider } from "react-wrap-balancer";
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
