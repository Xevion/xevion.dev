import { readItems, readSingleton } from "@directus/sdk";
import { GetStaticPropsResult, type NextPage } from "next";
import Head from "next/head";
import Link from "next/link";
import { useEffect } from "react";
import Balancer from "react-wrap-balancer";
import AppWrapper from "@/components/AppWrapper";
import ItemCard from "@/components/ItemCard";
import directus, { type Project } from "@/utils/directus";
import { useBreakpointValue } from "@/utils/helpers";
import Dots from "@/components/Dots";

type IndexProps = {
  tagline: string;
  projects: Project[];
  buttons: { text: string; href: string }[];
};

export async function getStaticProps(): Promise<
  GetStaticPropsResult<IndexProps>
> {
  const [metadata, projects] = await Promise.all([directus.request(readSingleton("metadata")), directus.request(readItems("project"))]);

  const resumeUrl = `${directus.url}assets/${metadata.resume}/${
    metadata.resumeFilename ?? "resume.pdf"
  }`;

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
          <Dots />
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
            <div className="hidden w-screen h-px animate-glow md:block animate-fade-left bg-gradient-to-r from-zinc-300/0 via-zinc-300/50 to-zinc-300/0" />
            <h1 className="font-hanken select-none py-3.5 px-0.5 z-10 text-transparent duration-1000 bg-white cursor-default text-edge-outline animate-title font-display text-5xl sm:text-6xl md:text-9xl lg:text-10xl whitespace-nowrap bg-clip-text ">
              XEVION
            </h1>
            <div className="hidden w-screen h-px animate-glow md:block animate-fade-right bg-gradient-to-r from-zinc-300/0 via-zinc-300/50 to-zinc-300/0" />
      {/* <div className="sm:text-9x cursor-default select-none py-10 font-hanken text-6xl font-black uppercase tracking-widest min-[300px]:text-7xl min-[500px]:text-8xl lg:text-10xl">
              Xevion
            </div> */}
            <div className="max-w-screen-sm text-center text-sm  sm:text-base animate-fade-in text-zinc-500">
              <Balancer>{tagline}</Balancer>
            </div>
            {/* <div className="max-w-screen-sm px-4 text-center text-base text-zinc-500 sm:text-sm">
              <Balancer>{tagline}</Balancer>
            </div>
          </div> */}
          </div>
        </div>
        <div
          id="projects"
          className="flex min-h-screen flex-row justify-center py-12 sm:py-8 md:items-center"
        >
          <div className="mx-auto h-full w-full max-w-[95%] lg:max-w-[85%] xl:max-w-[70%]">
            <div className="m-1 flex h-full flex-col justify-start gap-y-1">
              {projects.map((project) => {
                  return (<ItemCard key={project.id}
                                description={useLong ? project.description : project.shortDescription} banner={`${directus.url}assets/${project.bannerImage}`} title={project.name} location={`/${project.id}/`}                />
                  );
                })}
            </div>
          </div>
        </div>
      </AppWrapper>
    </>
  );
};

export default Home;
