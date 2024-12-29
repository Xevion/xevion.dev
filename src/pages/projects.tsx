import AppWrapper from "@/components/AppWrapper";
import directus from "@/utils/directus";
import { cn } from "@/utils/helpers";
import { readItems } from "@directus/sdk";
import { GetStaticPropsResult, type NextPage } from "next";
import Link from "next/link";

type Props = {
  projects: Awaited<ReturnType<typeof getProjects>>;
};

async function getProjects() {
  return await directus.request(readItems("project", {
    fields: ["id", "name", "shortDescription", "icon", {links: ["url"]}],
  }));
}

export async function getStaticProps(): Promise<GetStaticPropsResult<Props>> {;

  return {
    props: {
      projects: await getProjects(),
    }
  }
}

const ProjectsPage: NextPage<Props> = ({projects}) => {
  return (
    <AppWrapper dotsClassName="animate-bg-fast">
      <div className="max-w-500 mx-auto py-20 grid h-full w-max grid-cols-1 gap-x-20 gap-y-9 md:grid-cols-2 lg:grid-cols-3">
        <div className="md:col-span-2 text-center lg:col-span-3 w-full mb-10">
          <h1 className="text-4xl md:text-5xl pb-3 text-zinc-200 opacity-100 font-hanken">
          Projects
          </h1>
          <span className="text-lg text-zinc-400">
            created, maintained, or contributed to by me...
          </span>
        </div>
        {projects.map(({ id, name, shortDescription: description, links, icon }) =>
        {
          const useAnchor = links?.length ?? 0 > 0;
          const DynamicLink = useAnchor  ? Link : "div";
          const linkProps = useAnchor ? { href: links![0]!.url, target: "_blank", rel: "noreferrer" } : {};

            return <div className="flex" key={id}>
              {/* @ts-expect-error because div can't accept href */}
              <DynamicLink
                key={name}
                title={name}
                className="flex pl-3 pr-5 pt-1 pb-2.5 relative max-w-[30rem] items-center transition-colors rounded-lg text-zinc-400 hover:text-zinc-50 bg-black/10 hover:bg-zinc-500/10"
                {...linkProps}
              >

                <div className="w-14 pr-5 h-full flex items-center justify-center">
                  <i className={cn(icon ?? "fa-heart", "fa-solid text-3xl text-opacity-80 saturate-0")}></i>
                </div>
                <div className="flex-auto overflow-hidden">
                  <div className="text-lg">{name}</div>
                  <div className="text-base font-normal opacity-70 whitespace-nowrap">
                    {description}
                  </div>
                </div>
              </DynamicLink>
              <div className="grow" />
            </div>;
          } 
)}
      </div>
    </AppWrapper>
  );
};

export default ProjectsPage;
