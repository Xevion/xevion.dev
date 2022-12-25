import React, {useRef} from "react";
import {useOnClickOutside, useToggle} from "usehooks-ts";
import {classNames, isHoverable} from "../utils/helpers";
import DependentImage from "./DependentImage";
import ReactMarkdown from 'react-markdown'

import {AiFillGithub} from "react-icons/ai";

type ItemCardProps = {
    banner: string;
    bannerBlur: string;
    title: string;
    description: string;
}

const ItemCard = ({banner, bannerBlur, title, description}: ItemCardProps) => {
    const itemRef = useRef<HTMLDivElement>(null);
    const [active, toggleActive, setActive] = useToggle()

    useOnClickOutside(itemRef, () => {
        setActive(false);
    })

    return <div onClick={() => {
        if (!isHoverable()) toggleActive();
    }}
                ref={itemRef} className={classNames("item", active ? "active" : null)}>
        <DependentImage fill src={banner} blurDataURL={bannerBlur}
                        className={(loaded) => classNames("object-cover", loaded ? null : "blur-xl")}
                        placeholder="blur"
                        alt={`Banner for ${title}`}
        />
        <div className="elements grid grid-cols-12 h-full">
            <div className="col-span-8 max-h-full md:col-span-7 drop-shadow-2xl md:p-3 pl-2 md:ml-4">
                <div className="mt-1 text-lg md:mt-3 md:text-3xl">{title}</div>
                <div className="mt-0 md:mt-1.5 md:text-xl overflow-hidden">
                    <ReactMarkdown>{description}</ReactMarkdown>
                </div>
            </div>
            <div className="col-span-1 hidden md:block"/>
            <div className="col-span-4 w-full flex justify-end max-h-full md:py-5">
                <div
                    className="grid grid-cols-2 grid-rows-2 mr-1 md:mr-5 gap-3 aspect-square max-w-full object-contain max-h-full icon-grid">
                    <AiFillGithub/>
                </div>
            </div>
        </div>
    </div>
}

export default ItemCard;