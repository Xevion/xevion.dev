import Image from "next/image";
import {ArrowLongRightIcon} from "@heroicons/react/24/outline";
import React, {useRef} from "react";
import {useOnClickOutside, useToggle} from "usehooks-ts";
import {classNames, isHoverable} from "../utils/helpers";

type ItemCardProps = {
    banner: string;
    title: React.ReactNode;
    description: React.ReactNode;
}

const ItemCard = ({banner, title, description}: ItemCardProps) => {
    const itemRef = useRef<HTMLDivElement>(null);
    const [active, toggleActive, setActive] = useToggle()

    useOnClickOutside(itemRef, () => {
        setActive(false);
    })

    return <div onClick={() => {if (!isHoverable()) toggleActive();}}
                ref={itemRef} className={classNames("item", active ? "active" : null)}>
        <Image fill src={banner}
               alt={`Banner for ${title}`}
               style={{objectFit: "cover"}}
        />
        <div className="elements grid grid-cols-5 h-full">
            <div className="col-span-4 md:col-span-3 z-30 drop-shadow-2xl p-0 md:p-3 pl-2 md:ml-4">
                <div className="mt-1 text-xl md:mt-3 md:text-3xl">{title}</div>
                <div className="mt-0 text-sm md:mt-1.5 md:text-xl ">{description}</div>
            </div>
            <div className="hidden md:block"/>
            <div className="col-span-1 w-full h-full flex align-center justify-center text-zinc-50 pr-10">
                <ArrowLongRightIcon
                    className="max-w-full stroke-1 text-white z-10 aspect-square w-50"/>
            </div>
        </div>
    </div>
}

export default ItemCard;