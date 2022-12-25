import {GetStaticPropsContext, type NextPage} from "next";
import Head from "next/head";
import React from "react";
import ItemCard from "../components/ItemCard";
import {getPlaiceholder} from "plaiceholder";

interface Project {
    title: string;
    banner: string;
    description: string;
}

type ProjectWithBlur = Project & { bannerBlur: string };


type HomeStaticProps = {
    projects: ProjectWithBlur[];
}

export async function getStaticProps(context: GetStaticPropsContext) {
    const projects = [
        {
            title: "Phototag",
            banner: "/phototag.png",
            description: `Phototag is a **powerful** and **efficient** tool that helps you **quickly** and **easily** describe your photos with
                tags. Using Google&apos;s advanced Vision API, Phototag can automatically generate tags for your photos,
                making it faster and easier to organize and search for your images.`
        },
        {
            title: "Paths",
            banner: "/paths.png",
            description: ""
        }
    ].map(async project => {
        const {base64} = await getPlaiceholder(project.banner, {size: 16});
        return {
            ...project,
            bannerBlur: base64
        };
    })

    return {
        props: {
            projects: await Promise.all(projects)
        }
    }
}

const Home: NextPage<HomeStaticProps> = ({projects}: HomeStaticProps) => {
    return (
        <>
            <Head>
                <title>Xevion.dev</title>
                <meta name="description" content="My personal website."/>
                <link rel="icon" href="/favicon.ico"/>
            </Head>
            <main
                className="flex py-3 min-h-screen flex-row md:items-center justify-center bg-zinc-900 text-zinc-50">
                <div className="h-full w-full max-w-[95%] lg:max-w-[85%] xl:max-w-[70%] mx-auto">
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
