import React, {useRef} from "react";
import {useOnClickOutside, useToggle} from "usehooks-ts";
import {classNames, isHoverable, stopPropagation} from "../utils/helpers";
import DependentImage from "./DependentImage";
import ReactMarkdown from 'react-markdown'

import Link from "next/link";
import {LinkIcon, LinkIcons} from "../pages";
import {useRouter} from "next/router";

type ItemCardProps = {
    banner: string;
    bannerBlur: string;
    title: string;
    description: string;
    links?: LinkIcon[];
    location: string;
}

const ItemCard = ({banner, bannerBlur, title, description, links, location}: ItemCardProps) => {
    const itemRef = useRef<HTMLDivElement>(null);
    const mobileButtonRef = useRef<HTMLAnchorElement>(null);
    const [active, toggleActive, setActive] = useToggle()
    const router = useRouter();

    useOnClickOutside(itemRef, (event) => {
        if (mobileButtonRef.current != null && mobileButtonRef.current?.contains(event.target as Node))
            return;
        else
            setActive(false);
    })

    const navigate = () => {
        if (!isHoverable()) toggleActive();
        else {
            router.push(location);
        }
    }

    return <>
        <div ref={itemRef}
             className={classNames("item [&:not(:first-child)]:mt-3", active ? "active" : null)}
             onClick={navigate}>
            <DependentImage fill src={banner} blurDataURL={bannerBlur}
                            className={(loaded) => classNames("object-cover", loaded ? null : "blur-xl")}
                            placeholder="blur"
                            alt={`Banner for ${title}`}
            />
            <div className="elements grid grid-cols-12 h-full m-2 px-1 sm:px-4">
                <div
                    className="col-span-12 sm:col-span-9 lg:col-span-8 max-h-full overflow-hidden drop-shadow-2xl pb-2 md:p-1 pl-2">
                    <Link href={{pathname: location}}
                          className="text-lg sm:text-2xl md:text-3xl font-semibold">{title}</Link>
                    <div className="mt-0 md:mt-1.5 text-base sm:text-xl md:text-xl overflow-hidden"
                         onClick={navigate}
                         style={{hyphens: "auto"}}>
                        <ReactMarkdown>{description}</ReactMarkdown>
                    </div>
                </div>
                {(links?.length ?? 0) > 0 ?
                    <div
                        className="hidden sm:block col-span-3 lg:col-span-4 w-full flex justify-end max-h-full md:py-5">
                        <div className="grid grid-cols-2 grid-rows-2 p-2 md:gap-3 aspect-square icon-grid">
                            {links!.map(({icon, location, newTab}) =>
                                <Link key={location} href={location} target={(newTab ?? true) ? "_blank" : "_self"}
                                      onClick={stopPropagation}>
                                    {LinkIcons[icon]!({})}
                                </Link>)}
                        </div>
                    </div> : null}
            </div>

        </div>
        <Link ref={mobileButtonRef} href={{pathname: location}}
              className={classNames(
                  "transition-all bg-zinc-800 rounded border border-zinc-900 shadow w-full flex items-center justify-center",
                  active ? "opacity-100 h-9 p-2" : "opacity-0 h-0 p-0"
              )}>
            Learn More
        </Link>
    </>
}

export default ItemCard;