import type { NextPage } from "next";
import Head from "next/head";
import Image from "next/image";
import { BsGithub } from "react-icons/bs";
import { RxOpenInNewWindow } from "react-icons/rx";
import Link from "next/link";
import AppWrapper from "../components/AppWrapper";
import type { ReactNode } from "react";

type Screenshot = [string, null | string | ReactNode];
type ScreenshotWithQuality = [string, null | string | ReactNode, number];
const images: (Screenshot | ScreenshotWithQuality)[] = [
  ["/grain/index.jpg", null, 100],
  ["/grain/hidden.jpg", null, 100],
];

const GrainPage: NextPage = () => {
  return (
    <>
      <Head>
        <title>Grain | Xevion.dev</title>
      </Head>
      <AppWrapper>
        <div className="flex h-full min-h-screen w-full justify-center overflow-auto">
          <div className="relative my-10 w-full max-w-screen-md p-3 px-6">
            <div className="flex justify-between pb-2">
              <div className="text-3xl font-semibold">Grain</div>
              <div className="flex items-center justify-end space-x-1.5">
                <Link href="https://grain.xevion.dev" target="_blank">
                  <RxOpenInNewWindow className="h-6 w-6 hover:text-zinc-200" />
                </Link>
                <Link href="https://github.com/Xevion/grain" target="_blank">
                  <BsGithub className="h-6 w-6 hover:text-zinc-200" />
                </Link>
              </div>
            </div>
            <div className="relative">
              <Link href="https://grain.xevion.dev/">
                <Image
                  fill
                  quality={100}
                  sizes="100vw"
                  src="/grain/banner.jpeg"
                  alt=""
                  className="pointer-events-none !relative min-h-[10rem] rounded-md object-cover"
                />
              </Link>
            </div>
            <div className="prose prose-lg prose-invert mt-3 w-full">
              <p>
                After seeing an online post with beautiful noise patterns &
                gradients, I decided to try and recreate it. The result was
                Grain, a simple web app that generates beautiful noise. Under
                the hood, this app uses multiple layers of SVGs that
                automatically rescale with the browsers viewport. That way, the
                noise is always crisp and clear, no matter the screen size.
              </p>
              <ul className="md:columns-2">
                <li>Performant - SVG generation and layering is optimized</li>
                <li>Small - Builds in less than 16 seconds</li>
                <li>
                  Open Source - Want to use my gradients? Check it out on{" "}
                  <Link href="https://github.com/Xevion/grain" target="_blank">
                    GitHub
                  </Link>
                  .
                </li>
              </ul>
              <h3>Screenshots</h3>
              <div className="relative space-y-8">
                {images.map(([src, description, quality]) => {
                  return (
                    <div
                      key={src}
                      className="flex w-full flex-col justify-center"
                    >
                      <Image
                        fill
                        sizes="100vw"
                        src={src}
                        alt=""
                        quality={quality ?? 75}
                        className="pointer-events-none !relative !my-1 min-h-[10rem] rounded-md object-cover shadow-lg"
                      />
                      {description != null ? (
                        <span className="text-center text-base">
                          {description}
                        </span>
                      ) : null}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </div>
      </AppWrapper>
    </>
  );
};

export default GrainPage;
