import {GetStaticPropsContext, type NextPage} from "next";
import Head from "next/head";
import React, {useEffect} from "react";
import ItemCard from "../components/ItemCard";
import {getPlaiceholder} from "plaiceholder";
import {useBreakpointValue} from "../utils/helpers";

interface Project {
    title: string;
    banner: string;
    longDescription: string;
    shortDescription: string;
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
            longDescription: `Using Google's Vision API and supporting metadata formats on Windows, Phototag makes it easy to quickly add rich, descriptive tags to your photos, saving you time and effort.`,
            shortDescription: "Effortlessly add rich & descriptive tags to your photos with Phototag."
        },
        {
            title: "Paths",
            banner: "/paths.png",
            shortDescription: "Discover the power of graph traversal algorithms with my interactive application.",
            longDescription: `Discover the power of graph traversal algorithms with my interactive Unity application!
             Easily generate and edit graphs, create mazes, and experiment with different algorithm configurations to find the most efficient path.`
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
            <main
                className="flex py-3 min-h-screen flex-row md:items-center justify-center bg-zinc-900 text-zinc-50">
                <div className="h-full w-full max-w-[95%] lg:max-w-[85%] xl:max-w-[70%] mx-auto">
                    <div className="flex h-full m-1 flex-col justify-start gap-y-4">
                        {
                            projects.map((project, index) =>
                                <ItemCard key={index} {...project}
                                          description={useLong ? project.longDescription : project.shortDescription}/>
                            )
                        }
                    </div>
                </div>
            </main>
        </>
    );
};

export default Home;
