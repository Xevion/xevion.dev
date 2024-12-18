import type { NextPage } from "next";
import Head from "next/head";
import Image from "next/image";
import { BsGithub } from "react-icons/bs";
import Link from "next/link";
import AppWrapper from "../components/AppWrapper";
import type { ReactNode } from "react";

const images: [string, string | ReactNode][] = [
  ["/portal/home.jpeg", "The home page."],
  [
    "/portal/events.png",
    <>
      {" "}
      A page listing all current events. <br /> Initial data is cached for
      performance, but becomes dynamic when filtered.
    </>,
  ],
  [
    "/portal/admin.png",
    "A secure admin panel for our officers to view, filter & edit members & events.",
  ],
  ["/portal/event.png", "The view of a specific event."],
  ["/portal/checkin.png", "The check-in view."],
  [
    "/portal/filters.png",
    "Organization filtering options. Dynamic semester filtering & event sorting is also available.",
  ],
  ["/portal/login.png", "The login. Fast form validation, seamless login."],
  [
    "/portal/profile.png",
    <>
      The member profile view; fully editable on both desktop & mobile. <br />{" "}
      Seamless editing of profiles for users. Full validation available.
    </>,
  ],
  [
    "/portal/status.png",
    "Members can check their progress towards becoming full members & view what events they attended.",
  ],
];

const PortalPage: NextPage = () => {
  return (
    <>
      <Head>
        <title>Portal | Xevion.dev</title>
      </Head>
      <AppWrapper>
        <div className="flex h-full min-h-screen w-full justify-center overflow-auto">
          <div className="relative my-10 w-full max-w-screen-md p-3 px-6">
            <div className="flex justify-between pb-2">
              <div className="text-3xl font-semibold">Portal</div>
              <div className="flex items-center justify-end space-x-1.5">
                <Link href="https://github.com/acmutsa/Portal" target="_blank">
                  <BsGithub className="h-6 w-6 hover:text-zinc-200" />
                </Link>
              </div>
            </div>
            <div className="relative">
              <Link href="https://portal.acmutsa.org/">
                <Image
                  fill
                  sizes="100vw"
                  src="/portal/banner.jpeg"
                  alt=""
                  className="pointer-events-none !relative min-h-[10rem] rounded-md object-cover"
                />
              </Link>
            </div>
            <div className="prose prose-lg prose-invert mt-3 w-full">
              <p>
                Created in service of our membership, Portal was designed as a
                approachable membership portal for our users so we could{" "}
                <b>track membership</b>, <b>advertise events</b> and replace our
                existing <b>database solution</b>.
              </p>
              <ul className="md:columns-2">
                <li>Fast - built to serve thousands</li>
                <li>Cheap - minimize costs</li>
                <li>Open Source - help us improve</li>
                <li>Cutting Edge - the latest technology</li>
              </ul>
              <h3>Screenshots</h3>
              <div className="relative space-y-8">
                {images.map(([src, description]) => {
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
                        className="pointer-events-none !relative !my-1 min-h-[10rem] rounded-md object-cover shadow-lg"
                      />
                      <span className="text-center text-base">
                        {description}
                      </span>
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

export default PortalPage;
