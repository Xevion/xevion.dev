import { NextPage } from "next";
import Head from "next/head";
import Image from "next/image";
import { BsGithub } from "react-icons/bs";
import Link from "next/link";
import AppWrapper from "@/components/AppWrapper";

const PhototagPage: NextPage = () => {
  return (
    <>
      <Head>
        <title>Phototag | Xevion.dev</title>
      </Head>
      <AppWrapper>
        <div className="flex h-full min-h-screen w-full justify-center overflow-auto">
          <div className="relative my-10 w-full max-w-screen-md p-3 px-6">
            <div className="flex justify-between pb-2">
              <div className="text-2xl font-semibold">Phototag</div>
              <div className="flex items-center justify-end space-x-1.5">
                <Link href="https://github.com/Xevion/phototag" target="_blank">
                  <BsGithub className="h-5 w-5 hover:text-zinc-200" />
                </Link>
              </div>
            </div>
            <div className="relative">
              <Image
                fill
                sizes="100vw"
                src="/phototag.png"
                alt=""
                className="pointer-events-none !relative min-h-[10rem] rounded-md object-cover"
              />
            </div>
            <div className="prose prose-lg prose-invert mt-3 w-full">
              <p>
                Phototag is a powerful tool that helps you quickly and easily
                add rich, descriptive tags to your photos. Using Google&apos;s
                Vision API, Phototag automatically generates tags based on the
                visual content of your photos, making it easier than ever to
                organize and find your photos.
              </p>
              <p>
                With support for IPTC metadata and Adobe XMP Sidecar files, you
                can easily integrate Phototag into your existing workflow on
                Windows. Whether you&apos;re a professional photographer or a
                casual snapshot taker, Phototag is the perfect tool for adding
                clarity and context to your photos.
              </p>
              <ul className="md:columns-2">
                <li>Simple, but configurable</li>
                <li>Fully automatic</li>
                <li>Leverages compression to reduce network load</li>
                <li>Supports JPEG, PNG, GIF etc.</li>
                <li>Supports IPTC metadata</li>
                <li>Supports Adobe XMP sidecar files</li>
              </ul>
            </div>
          </div>
        </div>
      </AppWrapper>
    </>
  );
};

export default PhototagPage;
