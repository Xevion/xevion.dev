import AppWrapper from "@/components/AppWrapper";
import { getPayload } from "payload";
import config from "../../payload.config";
import Link from "next/link";
import Balancer from "react-wrap-balancer";

export const dynamic = "force-dynamic"; // Don't prerender at build time

type Metadata = {
  tagline: string;
  resume: {
    id: string;
    url: string;
    filename: string;
  };
  resumeFilename?: string;
};

export default async function HomePage() {
  const payloadConfig = await config;
  const payload = await getPayload({ config: payloadConfig });

  // @ts-ignore - Globals will be typed after first database connection
  const metadata = (await payload.findGlobal({
    slug: "metadata",
  })) as Metadata;

  const title = process.env.TITLE ?? "Xevion";
  const resumeUrl = metadata.resume?.url ?? "#";

  const buttons = [
    { text: "GitHub", href: "https://github.com/Xevion" },
    { text: "Projects", href: "/projects" },
    { text: "Blog", href: "https://undefined.behavio.rs" },
    { text: "Contact", href: "/contact" },
    { text: "Resume", href: resumeUrl },
  ];

  return (
    <AppWrapper className="overflow-x-hidden" dotsClassName="animate-bg">
      <div className="flex h-screen w-screen items-center justify-center overflow-hidden">
        <div className="relative z-10 flex w-full flex-col items-center justify-start">
          <nav className="z-10 animate-fade-in">
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
          <h1 className="text-edge-outline font-display my-3.5 animate-title select-none whitespace-nowrap bg-white bg-clip-text font-hanken text-5xl uppercase text-transparent drop-shadow-extreme duration-1000 sm:text-6xl md:text-9xl lg:text-10xl">
            {title}
          </h1>
          <div className="animate-glow hidden h-px w-screen animate-fade-right bg-gradient-to-r from-zinc-300/0 via-zinc-300/50 to-zinc-300/0 md:block" />
          <div className="max-w-screen-sm animate-fade-in text-center text-sm text-zinc-500 sm:text-base">
            <Balancer>{metadata.tagline}</Balancer>
          </div>
        </div>
      </div>
    </AppWrapper>
  );
}
