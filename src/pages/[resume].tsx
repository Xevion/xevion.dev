import directus from "@/utils/directus";
import { readSingleton } from "@directus/sdk";
import {
  GetStaticPaths,
  GetStaticPropsContext,
  GetStaticPropsResult,
} from "next";

// 'blocking' fallback, but don't provide any paths for pre-rendering; it will fail otherwise for redirect paths.
export const getStaticPaths: GetStaticPaths = async () => {
  return { paths: [], fallback: "blocking" };
};

// Handle static props for `[resume]` route.
export async function getStaticProps({
  params,
}: GetStaticPropsContext<{ resume: string }>): Promise<
  GetStaticPropsResult<never>
> {
  const { resume } = params ?? {};
  if (resume !== "resume") return { notFound: true };

  try {
    console.log("Revalidating resume redirect");
    const metadata = await directus.request(readSingleton("metadata"));
    const resumeUrl = `${directus.url}assets/${metadata.resume}/${
      metadata.resumeFilename ?? "resume.pdf"
    }`;

    return {
      redirect: { destination: resumeUrl, permanent: false },
      revalidate: 3600,
    };
  } catch (error) {
    console.error("Failed to acquire resume asset URL", error);
    throw new Error(`Failed to acquire asset (${error})`);
  }
}

// Empty component as the page redirects or returns `notFound`.
export default function Resume() {
  return <></>;
}
