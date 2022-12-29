import {type AppType} from "next/app";

import {trpc} from "../utils/trpc";

import "../styles/globals.scss";
import {Analytics} from "@vercel/analytics/react";

const MyApp: AppType = ({Component, pageProps}) => {
    return <>
        <Component {...pageProps} />
        <Analytics/>
    </>;
};

export default trpc.withTRPC(MyApp);
