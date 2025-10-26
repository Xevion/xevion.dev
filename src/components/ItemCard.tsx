import React, { useRef } from "react";
import { useOnClickOutside, useToggle } from "usehooks-ts";
import { cn, isHoverable } from "@/utils/helpers";
import ReactMarkdown from "react-markdown";
import Balancer from "react-wrap-balancer";

import Link from "next/link";
import { useRouter } from "next/router";
import { type LinkIcon, LinkIcons } from "@/utils/types";
import DependentImage from "@/components/DependentImage";

type ItemCardProps = {
  banner: string;
  bannerSettings?: { quality: number };
  title: string;
  description: string;
  links?: LinkIcon[];
  location: string;
};

const ItemCard = ({
  banner,
  title,
  description,
  links,
  location,
  bannerSettings,
}: ItemCardProps) => {
  const itemRef = useRef<HTMLDivElement>(null);
  const mobileButtonRef = useRef<HTMLAnchorElement>(null);
  const [active, toggleActive, setActive] = useToggle();
  const router = useRouter();

  // @ts-expect-error Some kind of regression in usehooks-ts causes the useOnClickOutside hook to not accept 'null' types
  useOnClickOutside(itemRef, (event) => {
    if (
      mobileButtonRef.current != null &&
      mobileButtonRef.current?.contains(event.target as Node)
    )
      return;
    else setActive(false);
  });

  const navigate = () => {
    if (!isHoverable()) toggleActive();
    else {
      router.push(location);
    }
  };

  return (
    <>
      <div
        ref={itemRef}
        className={cn(
          "item [&:not(:first-child)]:mt-3",
          active ? "active" : null,
        )}
        onClick={navigate}
      >
        <DependentImage
          fill
          src={banner}
          quality={bannerSettings?.quality ?? 75}
          className={(loaded) => cn("object-cover", loaded ? null : "blur-xl")}
          alt={`Banner for ${title}`}
        />
        <div className="elements m-2 grid h-full grid-cols-12 px-1 sm:px-4">
          <div className="col-span-12 max-h-full overflow-hidden pb-2 pl-2 drop-shadow-2xl sm:col-span-9 md:p-1 lg:col-span-8">
            <Link
              href={{ pathname: location }}
              className="font-roboto text-lg font-semibold sm:text-2xl md:text-3xl"
            >
              {title}
            </Link>
            <div
              className="description mt-0 overflow-hidden text-base font-light sm:text-xl md:mt-1.5 md:text-xl"
              onClick={(e) => {
                e.stopPropagation();
                navigate();
              }}
            >
              <Balancer>
                <ReactMarkdown>{description}</ReactMarkdown>
              </Balancer>
            </div>
          </div>
          {(links?.length ?? 0) > 0 ? (
            <div className="col-span-3 hidden max-h-full w-full justify-end sm:flex md:py-5 lg:col-span-4">
              <div className="icon-grid grid aspect-square grid-cols-2 grid-rows-2 p-2 md:gap-3">
                {links!.map(({ icon, location, newTab }) => (
                  <Link
                    key={location}
                    href={location}
                    target={(newTab ?? true) ? "_blank" : "_self"}
                    onClick={(e) => e.stopPropagation()}
                  >
                    {LinkIcons[icon]?.({})}
                  </Link>
                ))}
              </div>
            </div>
          ) : null}
        </div>
      </div>
      <Link
        aria-disabled={!active}
        ref={mobileButtonRef}
        href={active ? { pathname: location } : {}}
        className={cn(
          "flex w-full items-center justify-center rounded border border-zinc-900 bg-zinc-800 shadow transition-all",
          active ? "h-9 p-2 opacity-100" : "h-0 p-0 opacity-0",
        )}
      >
        Learn More
      </Link>
    </>
  );
};

export default ItemCard;
