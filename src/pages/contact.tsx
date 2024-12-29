import { type NextPage } from "next";
import AppWrapper from "@/components/AppWrapper";
import { BsDiscord, BsGithub } from "react-icons/bs";
import { AiFillMail } from "react-icons/ai";
import Link from "next/link";
import type { IconType } from "react-icons";
import Tippy from "@tippyjs/react";
import "tippy.js/dist/tippy.css";

const socials: {
  icon: IconType;
  href?: string;
  hint?: string;
  hideHint?: boolean;
}[] = [
  {
    icon: BsGithub,
    href: "https://github.com/Xevion/",
  },
  {
    icon: AiFillMail,
    href: "mailto:xevion@xevion.dev",
    hint: "xevion@xevion.dev",
  },
  {
    icon: BsDiscord,
    hint: "Xevion#8506",
  },
];

const ContactPage: NextPage = () => {
  return (
    <AppWrapper>
      <div className="my-10 flex w-full flex-col items-center">
        <div className="mx-3 flex w-full max-w-[23rem] flex-col rounded-md border border-zinc-800 bg-zinc-800/50 p-5 sm:max-w-[25rem] lg:max-w-[30rem]">
          <div className="flex justify-center gap-x-5 text-center">
            {socials.map(({ icon: Icon, href, hint, hideHint }, index) => {
              const inner = <Icon className="h-8 w-8" />;
              return (
                <Tippy key={index} disabled={hideHint} content={hint ?? href}>
                  {href != undefined ? (
                    <Link href={href}>{inner}</Link>
                  ) : (
                    <span>{inner}</span>
                  )}
                </Tippy>
              );
            })}
          </div>
        </div>
      </div>
    </AppWrapper>
  );
};

export default ContactPage;
