import AppWrapper from "@/components/AppWrapper";
import directus from "@/utils/directus";
import { cn } from "@/utils/helpers";
import { readItems } from "@directus/sdk";
import { GetStaticPropsResult, type NextPage } from "next";
import Link from "next/link";
import Balancer from "react-wrap-balancer";

type Props = {
  projects: Awaited<ReturnType<typeof getProjects>>;
};

async function getProjects() {
  return await directus.request(
    readItems("project", {
      fields: ["id", "name", "shortDescription", "icon", { links: ["url"] }],
      sort: "-date_updated",
    }),
  );
}

export async function getStaticProps(): Promise<GetStaticPropsResult<Props>> {
  return {
    props: {
      projects: await getProjects(),
    },
  };
}

const ProjectsPage: NextPage<Props> = ({ projects }) => {
  return (
    <AppWrapper dotsClassName="animate-bg-fast">
      <div className="mx-auto grid grid-cols-1 justify-center gap-y-4 px-4 py-20 align-middle sm:grid-cols-2 md:max-w-[50rem] lg:max-w-[75rem] lg:grid-cols-3 lg:gap-y-9">
        <div className="mb-3 text-center sm:col-span-2 md:mb-5 lg:col-span-3 lg:mb-7">
          <h1 className="pb-3 font-hanken text-4xl text-zinc-200 opacity-100 md:text-5xl">
            Projects
          </h1>
          <Balancer className="text-lg text-zinc-400">
            created, maintained, or contributed to by me...
          </Balancer>
        </div>
        {projects.map(
          ({ id, name, shortDescription: description, links, icon }) => {
            const useAnchor = links?.length ?? 0 > 0;
            const DynamicLink = useAnchor ? Link : "div";
            const linkProps = useAnchor
              ? { href: links![0]!.url, target: "_blank", rel: "noreferrer" }
              : {};

            return (
              <div className="max-w-fit" key={id}>
                {/* @ts-expect-error because div can't accept href */}
                <DynamicLink
                  key={name}
                  title={name}
                  className="flex items-center justify-start overflow-hidden rounded bg-black/10 pb-2.5 pl-3 pr-5 pt-1 text-zinc-400 transition-colors hover:bg-zinc-500/10 hover:text-zinc-50"
                  {...linkProps}
                >
                  <div className="flex h-full w-14 items-center justify-center pr-5">
                    <i
                      className={cn(
                        icon ?? "fa-heart",
                        "fa-solid text-3xl text-opacity-80 saturate-0",
                      )}
                    ></i>
                  </div>
                  <div className="overflow-hidden">
                    <span className="text-sm md:text-base lg:text-lg">
                      {name}
                    </span>
                    <p
                      className="truncate text-xs opacity-70 md:text-sm lg:text-base"
                      title={description}
                    >
                      {description}
                    </p>
                  </div>
                </DynamicLink>
              </div>
            );
          },
        )}
      </div>
    </AppWrapper>
  );
};

export default ProjectsPage;
