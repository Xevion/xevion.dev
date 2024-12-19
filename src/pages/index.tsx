import { readSingleton } from "@directus/sdk";
import { GetStaticPropsResult, type NextPage } from "next";
import Head from "next/head";
import Link from "next/link";
import { useEffect } from "react";
import Balancer from "react-wrap-balancer";
import AppWrapper from "../components/AppWrapper";
import ItemCard from "../components/ItemCard";
import directus from "../utils/directus";
import { useBreakpointValue } from "../utils/helpers";
import type { Project } from "../utils/types";

type IndexProps = {
  tagline: string;
  projects: Project[];
  buttons: { text: string; href: string }[];
};

export async function getStaticProps(): Promise<
  GetStaticPropsResult<IndexProps>
> {
  const metadata = await directus.request(readSingleton("metadata"));
  const resumeUrl = `${directus.url}assets/${metadata.resume}/${
    metadata.resumeFilename ?? "resume.pdf"
  }`;

  const projects: Project[] = [
    {
      title: "Portal",
      banner: "/portal/banner.jpeg",
      location: "/portal",
      longDescription:
        "An advanced membership & event management system for my university's premier computer science organization.",
      shortDescription:
        "Advanced membership & event management system for students",
      links: [
        {
          icon: "github",
          location: "https://github.com/acmutsa/Portal",
        },
        {
          icon: "external",
          location: "https://portal.acmutsa.org/",
        },
      ],
    },
    {
      title: "Phototag",
      banner: "/phototag.png",
      location: "/phototag",
      longDescription: `Using Google's Vision API and supporting metadata formats on Windows, Phototag makes it easy to quickly add rich, descriptive tags to your photos, saving you time and effort.`,
      shortDescription:
        "Effortlessly add rich & descriptive tags to your photos with Phototag.",
      links: [
        {
          icon: "github",
          location: "https://github.com/Xevion/phototag",
        },
      ],
    },
    {
      title: "Paths",
      banner: "/paths.png",
      location: "/paths",
      shortDescription:
        "Discover the power of graph traversal algorithms with my interactive application.",
      longDescription: `Discover the power of graph traversal algorithms with my interactive Unity application!
             Easily generate and edit graphs, create mazes, and experiment with different algorithm configurations to find the most efficient path.`,
      links: [
        {
          icon: "github",
          location: "https://github.com/Xevion/Paths",
        },
      ],
    },
    {
      title: "Grain",
      banner: "/grain/banner.jpeg",
      bannerSettings: { quality: 100 },
      location: "/grain",
      shortDescription:
        "An experimental React app to generate beautiful backgrounds with noise filters.",
      longDescription:
        "Quickly generate beautiful backgrounds with noise filters. Built with React, hosted on Vercel, and rendered using simple SVG noise filters (just HTML & CSS).",
      links: [
        {
          icon: "external",
          location: "https://grain.xevion.dev",
        },
        {
          icon: "github",
          location: "https://github.com/Xevion/grain",
        },
      ],
    },
  ];

  return {
    props: {
      tagline: metadata.tagline,
      projects,
      buttons: [
        { text: "GitHub", href: "https://github.com/Xevion" },
        { text: "Projects", href: "/projects" },
        { text: "Blog", href: "https://undefined.behavio.rs" },
        { text: "Contact", href: "/contact" },
        { text: "Resume", href: resumeUrl },
      ],
    },
    revalidate: 60 * 10,
  };
}

const Home: NextPage<IndexProps> = ({
  tagline,
  projects,
  buttons,
}: IndexProps) => {
  const useLong = useBreakpointValue("sm", true, false);

  // use-tailwind-breakpoint
  useEffect(() => {
    if (typeof window !== "undefined")
      window.dispatchEvent(new Event("resize"));
  }, []);

  return (
    <>
      <Head>
        <title>Xevion.dev</title>
        <meta name="description" content="My personal website." />
        <link rel="icon" href="/favicon.ico" />
      </Head>
      <AppWrapper hideNavigation={true} className="overflow-x-hidden">
        <div className="flex h-screen w-screen items-center justify-center overflow-hidden">
          <div className="flex w-full flex-col items-center justify-start">
            <nav className="animate-fade-in">
              <ul className="flex items-center justify-center gap-4">
                {buttons.map(({ text, href }) => (
                  <Link
                    key={href}
                    className="text-sm text-zinc-500 duration-500 hover:text-zinc-300"
                    href={href}
                  >
                    {text}
                  </Link>
                ))}
              </ul>
            </nav>
            <div className="sm:text-9x cursor-default select-none py-10 font-hanken text-6xl font-black uppercase tracking-widest min-[300px]:text-7xl min-[500px]:text-8xl lg:text-10xl">
              Xevion
            </div>
            <div className="max-w-screen-sm px-4 text-center text-base text-zinc-500 sm:text-sm">
              <Balancer>{tagline}</Balancer>
            </div>
          </div>
        </div>
        <div
          id="projects"
          className="flex min-h-screen flex-row justify-center py-12 sm:py-8 md:items-center"
        >
          <div className="mx-auto h-full w-full max-w-[95%] lg:max-w-[85%] xl:max-w-[70%]">
            <div className="m-1 flex h-full flex-col justify-start gap-y-1">
              {projects.map((project, index) => (
                <ItemCard
                  key={index}
                  {...project}
                  description={
                    useLong ? project.longDescription : project.shortDescription
                  }
                />
              ))}
            </div>
          </div>
        </div>
      </AppWrapper>
    </>
  );
};

export default Home;
