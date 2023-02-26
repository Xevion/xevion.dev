import {type NextPage} from "next";
import Head from "next/head";
import React, {useEffect} from "react";
import ItemCard from "../components/ItemCard";
import {getPlaiceholder} from "plaiceholder";
import {useBreakpointValue} from "../utils/helpers";
import type {Project} from "../utils/types";
import Link from "next/link";

type ProjectWithBlur = Project & { bannerBlur: string };


type HomeStaticProps = {
    projects: ProjectWithBlur[];
}

export async function getStaticProps() {
    const projects: Project[] = [
        {
            title: "Portal",
            banner: "/portal/banner.jpeg",
            location: "/portal",
            longDescription: "An advanced membership & event management system for my university's premier computer science organization.",
            shortDescription: "Advanced membership & event management system for students",
            links: [
                {
                    icon: "github",
                    location: "https://github.com/UTSA-ACM/Portal"
                },
                {
                    icon: "external",
                    location: "https://portal.acmutsa.org/"
                }
            ]
        },
        {
            title: "Phototag",
            banner: "/phototag.png",
            location: "/phototag",
            longDescription: `Using Google's Vision API and supporting metadata formats on Windows, Phototag makes it easy to quickly add rich, descriptive tags to your photos, saving you time and effort.`,
            shortDescription: "Effortlessly add rich & descriptive tags to your photos with Phototag.",
            links: [
                {
                    icon: "github",
                    location: "https://github.com/Xevion/phototag"
                },
                {
                    icon: "external",
                    location: "https://phototag.xevion.dev"
                }
            ]
        },
        {
            title: "Paths",
            banner: "/paths.png",
            location: "/paths",
            shortDescription: "Discover the power of graph traversal algorithms with my interactive application.",
            longDescription: `Discover the power of graph traversal algorithms with my interactive Unity application!
             Easily generate and edit graphs, create mazes, and experiment with different algorithm configurations to find the most efficient path.`,
            links: [
                {
                    icon: "github",
                    location: "https://github.com/Xevion/Paths",
                }
            ]
        },
        {
            title: "Grain",
            banner: "/grain/banner.jpeg",
            bannerSettings: {quality: 100},
            location: "/grain",
            shortDescription: "An experimental React app to generate beautiful backgrounds with noise filters.",
            longDescription: "Quickly generate beautiful backgrounds with noise filters. Built with React, hosted on Vercel, and rendered using simple SVG noise filters (just HTML & CSS).",
            links: [
                {
                    icon: 'external',
                    location: "https://grain.xevion.dev"
                },
                {
                    icon: "github",
                    location: "https://github.com/Xevion/grain"
                }
            ]
        }
    ];

    return {
        props: {
            projects: await Promise.all(projects.map(async project => {
                const {base64} = await getPlaiceholder(project.banner, {size: 16});
                return {
                    ...project,
                    bannerBlur: base64
                };
            }))
        }
    }
}

const buttons = [
    {text: "GitHub", href: "https://github.com/Xevion"},
    {text: "Contact", href: "/contact"},
    {text: "Resume", href: "/resume"}
]

const Home: NextPage<HomeStaticProps> = ({projects}: HomeStaticProps) => {
    const useLong = useBreakpointValue("sm", true, false);

    // use-tailwind-breakpoint
    useEffect(() => {
        typeof window !== "undefined" ? window.dispatchEvent(new Event("resize")) : null;
    }, []);

    return (
        <>
            <Head>
                <title>Xevion.dev</title>
                <meta name="description" content="My personal website."/>
                <link rel="icon" href="/favicon.ico"/>
            </Head>
            <main className="bg-zinc-900 text-zinc-50 overflow-x-hidden">
                <div className="flex justify-center items-center bg-zinc-850 h-screen w-screen overflow-hidden">
                    <div className="top-0 p-3 absolute w-full flex justify-end">
                        <span
                            className="leading-3 bg-yellow-300 rounded-md text-black font-bold font-inter p-2">WIP</span>
                    </div>
                    <div className="w-full flex flex-col items-center h-40">
                        <div className="text-4xl sm:text-5xl pb-3">Hi. I&apos;m Ryan Walters.</div>
                        <div className="text-lg text-zinc-200">Full Stack Software Developer</div>
                        <div className="w-full flex justify-center py-2 space-x-2">
                            {buttons.map(({text, href}) =>
                                <Link href={href} key={href}>
                                    <div className="p-2 rounded-sm bg-zinc-900 hover:bg-zinc-800">
                                        {text}
                                    </div>
                                </Link>
                            )}
                        </div>
                    </div>
                </div>
                <div id="projects"
                    className="flex py-12 sm:py-8 min-h-screen flex-row md:items-center justify-center">
                    <div className="h-full w-full max-w-[95%] lg:max-w-[85%] xl:max-w-[70%] mx-auto">
                        <div className="flex h-full m-1 flex-col justify-start gap-y-1">
                            {
                                projects.map((project, index) =>
                                    <ItemCard key={index} {...project}
                                              description={useLong ? project.longDescription : project.shortDescription}/>
                                )
                            }
                        </div>
                    </div>
                </div>
            </main>
        </>
    );
};

export default Home;
