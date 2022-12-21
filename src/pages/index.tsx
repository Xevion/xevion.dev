import {type NextPage} from "next";
import Head from "next/head";

import {trpc} from "../utils/trpc";
import Image from "next/image";
import {ArrowLongRightIcon} from "@heroicons/react/24/outline";
import SteppedSpan from "../components/SteppedSpan";

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
                tags. Using Google's advanced Vision API, Phototag can automatically generate tags for your photos,
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
                className="flex max-h-screen min-h-screen flex-row items-center justify-center bg-zinc-900 text-zinc-50">
                <div className="h-full max-w-[85%] w-[70%] 2xl:w-[70%] mx-auto">
                    <div className="flex h-full m-1 flex-col justify-start gap-y-4">
                        <SteppedSpan >Ryan Walters</SteppedSpan>
                        {
                            projects.map((project, index) =>
                                <div key={index} className="item">
                                    <Image fill src={project.banner}
                                           alt={`Banner for ${project.title}`}
                                           style={{objectFit: "cover"}}
                                    />
                                    <div className="elements grid grid-cols-5 h-full">
                                        <div className="col-span-3 z-30 drop-shadow-2xl p-3 pl-4">
                                            <div className="mt-3 text-3xl">{project.title}</div>
                                            <div className="mt-1.5 text-xl">{project.description}</div>
                                        </div>
                                        <div/>
                                        <div className="col-span-1 w-full h-full flex align-center justify-center text-zinc-50 pr-10">
                                            <ArrowLongRightIcon
                                                className="max-w-full stroke-1 text-white z-10 aspect-square w-50"/>
                                        </div>
                                    </div>
                                </div>
                            )
                        }
                    </div>
                </div>
            </main>
        </>
    );
};

export default Home;
