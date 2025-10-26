"use client";

import AppWrapper from "@/components/AppWrapper";
import { BsDiscord, BsGithub } from "react-icons/bs";
import { AiFillMail } from "react-icons/ai";
import Link from "next/link";
import type { IconType } from "react-icons";
import {
  useFloating,
  autoUpdate,
  offset,
  flip,
  shift,
  useHover,
  useFocus,
  useDismiss,
  useRole,
  useInteractions,
  FloatingPortal,
} from "@floating-ui/react";
import { useState } from "react";

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

function SocialTooltip({
  icon: Icon,
  href,
  hint,
  hideHint,
}: {
  icon: IconType;
  href?: string;
  hint?: string;
  hideHint?: boolean;
}) {
  const [isOpen, setIsOpen] = useState(false);

  const { refs, floatingStyles, context } = useFloating({
    open: isOpen,
    onOpenChange: setIsOpen,
    placement: "top",
    whileElementsMounted: autoUpdate,
    middleware: [offset(10), flip(), shift()],
  });

  const hover = useHover(context);
  const focus = useFocus(context);
  const dismiss = useDismiss(context);
  const role = useRole(context, { role: "tooltip" });

  const { getReferenceProps, getFloatingProps } = useInteractions([
    hover,
    focus,
    dismiss,
    role,
  ]);

  const inner = <Icon className="h-8 w-8" />;
  const tooltipContent = hint ?? href;

  return (
    <>
      {href != undefined ? (
        <Link
          href={href}
          ref={refs.setReference}
          {...getReferenceProps()}
        >
          {inner}
        </Link>
      ) : (
        <span
          ref={refs.setReference}
          {...getReferenceProps()}
        >
          {inner}
        </span>
      )}
      {!hideHint && isOpen && tooltipContent && (
        <FloatingPortal>
          <div
            ref={refs.setFloating}
            style={floatingStyles}
            {...getFloatingProps()}
            className="z-50 rounded bg-zinc-900 px-3 py-2 text-sm text-zinc-100 shadow-lg"
          >
            {tooltipContent}
          </div>
        </FloatingPortal>
      )}
    </>
  );
}

export default function ContactPage() {
  return (
    <AppWrapper>
      <div className="my-10 flex w-full flex-col items-center">
        <div className="mx-3 flex w-full max-w-[23rem] flex-col rounded-md border border-zinc-800 bg-zinc-800/50 p-5 sm:max-w-[25rem] lg:max-w-[30rem]">
          <div className="flex justify-center gap-x-5 text-center">
            {socials.map((social, index) => (
              <SocialTooltip key={index} {...social} />
            ))}
          </div>
        </div>
      </div>
    </AppWrapper>
  );
}
