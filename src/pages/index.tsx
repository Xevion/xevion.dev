import {type NextPage} from "next";
import Head from "next/head";
import React from "react";
import ItemCard from "../components/ItemCard";

interface Project {
    title: string;
    banner: string;
    description: React.ReactNode;
}

const Home: NextPage = () => {
    const projects: Project[] = [
        {
            title: "Phototag",
            banner: "/phototag.png",
            description: <>
                Phototag is a <b>powerful</b> and <b>efficient</b> tool that helps
                you <b>quickly</b> and <b>easily</b> describe your photos with
                tags. Using Google&apos;s advanced Vision API, Phototag can automatically generate tags for your photos,
                making it faster and easier to organize and search for your images.
            </>
        },
        {
            title: "Paths",
            banner: "/paths.png",
            description: ""
        }
    ]

    return (
        <>
            <Head>
                <title>Xevion.dev</title>
                <meta name="description" content="My personal website."/>
                <link rel="icon" href="/favicon.ico"/>
            </Head>
            <main
                className="flex py-3 max-h-screen min-h-screen flex-row md:items-center justify-center bg-zinc-900 text-zinc-50">
                <div className="h-full w-full max-w-[95%] 2xl:max-w-[70%] mx-auto">
                    <div className="flex h-full m-1 flex-col justify-start gap-y-4">
                        {
                            projects.map((project, index) =>
                                <ItemCard key={index} {...project} />
                            )
                        }
                    </div>
                </div>
            </main>
        </>
    );
};

export default Home;
