import AppWrapper from "@/components/AppWrapper";
import directus, { Project } from "@/utils/directus";
import { cn } from "@/utils/helpers";
import { readItems } from "@directus/sdk";
import { GetStaticPropsResult, type NextPage } from "next";
import Link from "next/link";

type Props = {
  projects: Project[];
};

export async function getStaticProps(): Promise<GetStaticPropsResult<Props>> {
  const projects = await directus.request(readItems("project"));

  return {
    props: {projects}
  }
}

const ProjectsPage: NextPage<Props> = ({projects}) => {
  return (
    <AppWrapper dotsClassName="animate-bg-fast">
      <div className="max-w-500 mx-auto mt-20 grid h-full w-max grid-cols-1 gap-x-20 gap-y-7 py-2 md:grid-cols-2 lg:grid-cols-3">
        {projects.map(({ name, shortDescription: description, links, icon }) => (
          <Link
            key={name}
            className="flex relative max-w-[30rem] flex-shrink items-center opacity-75 transition-opacity hover:opacity-100 bg-black/30 hover:bg-white/5 py-2 rounded"
            href={links[0]?.url ?? "#"}
            target="_blank"
            rel="noreferrer"
            title={name}
          >
            
            <div className="pr-5 pt-2">
              <i className={cn(icon ?? "fa-heart", "fa-solid text-3xl text-opacity-80 saturate-0")}></i>
            </div>
            <div className="flex-auto overflow-hidden">
              <div className="text-lg">{name}</div>
              <div className="text-base font-normal opacity-70 whitespace-nowrap">
                {description}
              </div>
            </div>
          </Link>
        ))}
      </div>
    </AppWrapper>
  );
};

export default ProjectsPage;
