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
      <div className="max-w-500 mx-auto grid h-full w-max grid-cols-1 gap-x-20 gap-y-9 py-20 md:grid-cols-2 lg:grid-cols-3">
        <div className="mb-10 w-full text-center md:col-span-2 lg:col-span-3">
          <h1 className="pb-3 font-hanken text-4xl text-zinc-200 opacity-100 md:text-5xl">
            Projects
          </h1>
          <span className="text-lg text-zinc-400">
            created, maintained, or contributed to by me...
          </span>
        </div>
        {projects.map(
          ({ id, name, shortDescription: description, links, icon }) => {
            const useAnchor = links?.length ?? 0 > 0;
            const DynamicLink = useAnchor ? Link : "div";
            const linkProps = useAnchor
              ? { href: links![0]!.url, target: "_blank", rel: "noreferrer" }
              : {};

            return (
              <div className="flex" key={id}>
                {/* @ts-expect-error because div can't accept href */}
                <DynamicLink
                  key={name}
                  title={name}
                  className="relative flex max-w-[30rem] items-center rounded-lg bg-black/10 pb-2.5 pl-3 pr-5 pt-1 text-zinc-400 transition-colors hover:bg-zinc-500/10 hover:text-zinc-50"
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
                  <div className="flex-auto overflow-hidden">
                    <div className="text-lg">{name}</div>
                    <div className="whitespace-nowrap text-base font-normal opacity-70">
                      {description}
                    </div>
                  </div>
                </DynamicLink>
                <div className="grow" />
              </div>
            );
          },
        )}
      </div>
    </AppWrapper>
  );
};

export default ProjectsPage;
