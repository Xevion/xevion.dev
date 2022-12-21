import { type AppType } from "next/app";

import { trpc } from "../utils/trpc";

import "../styles/globals.scss";

const MyApp: AppType = ({ Component, pageProps }) => {
  return <Component {...pageProps} />;
};

export default trpc.withTRPC(MyApp);
