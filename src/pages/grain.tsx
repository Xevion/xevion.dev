import type {NextPage} from "next";
import Head from "next/head";
import Image from "next/image";
import {BsGithub} from "react-icons/bs";
import {RxOpenInNewWindow} from "react-icons/rx";
import Link from "next/link";
import AppWrapper from "../components/AppWrapper";
import type {ReactNode} from "react";

type Screenshot = [string, null | string | ReactNode];
type ScreenshotWithQuality = [string, null | string | ReactNode, number];
const images: (Screenshot | ScreenshotWithQuality)[] = [
    ["/grain/index.jpg", null, 100],
    ["/grain/hidden.jpg", null, 100]
]

const GrainPage: NextPage = () => {
    return <>
        <Head>
            <title>Grain | Xevion.dev</title>
        </Head>
        <AppWrapper>
            <div className="w-full overflow-auto h-full min-h-screen flex justify-center">
                <div className="relative my-10 p-3 px-6 w-full max-w-screen-md">
                    <div className="pb-2 flex justify-between">
                        <div className="text-3xl font-semibold">
                            Grain
                        </div>
                        <div className="flex items-center justify-end space-x-1.5">
                            <Link href="https://grain.xevion.dev" target="_blank">
                                <RxOpenInNewWindow className="w-6 h-6 hover:text-zinc-200"/>
                            </Link>
                            <Link href="https://github.com/Xevion/grain" target="_blank">
                                <BsGithub className="w-6 h-6 hover:text-zinc-200"/>
                            </Link>
                        </div>
                    </div>
                    <div className="relative">
                        <Link href="https://grain.xevion.dev/">
                            <Image fill quality={100} sizes="100vw" src="/grain/banner.jpeg" alt=""
                                   className="!relative pointer-events-none min-h-[10rem] rounded-md object-cover"/>
                        </Link>
                    </div>
                    <div className="mt-3 w-full prose prose-invert prose-lg">
                        <p>
                            After seeing an online post with beautiful noise patterns & gradients, I decided to
                            try and recreate it. The result was Grain, a simple web app that generates beautiful noise.
                            Under the hood, this app uses multiple layers of SVGs that automatically rescale with the browsers viewport.
                            That way, the noise is always crisp and clear, no matter the screen size.
                        </p>
                        <ul className="md:columns-2">
                            <li>Performant - SVG generation and layering is optimized</li>
                            <li>Small - Builds in less than 16 seconds</li>
                            <li>Open Source - Want to use my gradients? Check it out on <Link href="https://github.com/Xevion/grain" target="_blank">GitHub</Link>.</li>
                        </ul>
                        <h3>Screenshots</h3>
                        <div className="relative space-y-8">
                            {images.map(([src, description, quality]) => {
                                    return <div key={src} className="flex flex-col justify-center w-full">
                                        <Image fill sizes="100vw" src={src} alt="" quality={quality ?? 75}
                                               className="shadow-lg !my-1 !relative pointer-events-none min-h-[10rem] rounded-md object-cover"/>
                                        {description != null ?
                                        <span
                                        className="text-center text-base">{description}</span> : null}
                                    </div>
                                }
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </AppWrapper>

    </>
}

export default GrainPage;