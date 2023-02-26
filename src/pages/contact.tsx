import {type NextPage} from "next";
import AppWrapper from "../components/AppWrapper";
import {BsDiscord, BsGithub} from "react-icons/bs";
import {AiFillMail} from "react-icons/ai";
import Link from "next/link";
import type {IconType} from "react-icons";
import Tippy from "@tippyjs/react";
import 'tippy.js/dist/tippy.css';

const socials: { icon: IconType, href?: string, hint?: string, hideHint?: boolean }[] = [
    {
        icon: BsGithub,
        href: "https://github.com/Xevion/"
    },
    {
        icon: AiFillMail,
        href: "mailto:xevion@xevion.dev",
        hint: "xevion@xevion.dev"
    },
    {
        icon: BsDiscord,
        hint: "Xevion#8506"
    }
]

const ContactPage: NextPage = () => {
    return <AppWrapper current='contact'>
        <div className="w-full my-10 flex flex-col items-center">
            <div
                className="bg-zinc-800/50 border border-zinc-800 rounded-md max-w-[23rem] sm:max-w-[25rem] lg:max-w-[30rem] mx-3 w-full p-5 flex flex-col">
                <div className="flex justify-center gap-x-5 text-center">
                    {socials.map(({icon: Icon, href, hint, hideHint}, index) => {
                        const inner = <Icon className="w-8 h-8"/>;
                        return <Tippy key={index} disabled={hideHint} content={hint ?? href}>
                            {
                                href != undefined ? <Link href={href}>{inner}</Link> : <span>{inner}</span>
                            }
                        </Tippy>
                    })}
                </div>
            </div>
        </div>
    </AppWrapper>
}

export default ContactPage;