import AppWrapper from "@/components/AppWrapper";
import directus from "@/utils/directus";
import { readSingleton } from "@directus/sdk";
import { GetStaticPropsResult, type NextPage } from "next";
import Head from "next/head";
import Link from "next/link";
import { useEffect, useState } from "react";
import Balancer from "react-wrap-balancer";

type IndexProps = {
  tagline: string;
  buttons: { text: string; href: string }[];
};

export async function getStaticProps(): Promise<
  GetStaticPropsResult<IndexProps>
> {
  const metadata = await directus.request(readSingleton("metadata"));

  const resumeUrl = `${directus.url}assets/${metadata.resume}/${
    metadata.resumeFilename ?? "resume.pdf"
  }`;

  return {
    props: {
      tagline: metadata.tagline,
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

const Home: NextPage<IndexProps> = ({ tagline, buttons }: IndexProps) => {
  const [isWalters, setIsWalters] = useState(false);
  useEffect(() => {
    // Check if URL contains "walters.to"
    if (location.href.includes("walters.to")) {
      setIsWalters(true);
    }
  }, []);

  return (
    <>
      <Head>
        <title>Xevion.dev</title>
        <meta name="description" content="My personal website." />
        <link rel="icon" href="/favicon.ico" />
      </Head>
      <AppWrapper className="overflow-x-hidden" dotsClassName="animate-bg">
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
            <div className="animate-glow hidden h-px w-screen animate-fade-left bg-gradient-to-r from-zinc-300/0 via-zinc-300/50 to-zinc-300/0 md:block" />
            <h1 className="text-edge-outline font-display z-10 my-3.5 animate-title whitespace-nowrap bg-white bg-clip-text font-hanken text-5xl uppercase text-transparent drop-shadow-extreme duration-1000 sm:text-6xl md:text-9xl lg:text-10xl">
              {isWalters ? "Walters" : "Xevion"}
            </h1>
            <div className="animate-glow hidden h-px w-screen animate-fade-right bg-gradient-to-r from-zinc-300/0 via-zinc-300/50 to-zinc-300/0 md:block" />
            <div className="max-w-screen-sm animate-fade-in text-center text-sm text-zinc-500 sm:text-base">
              <Balancer>{tagline}</Balancer>
            </div>
          </div>
        </div>
      </AppWrapper>
    </>
  );
};

export default Home;
