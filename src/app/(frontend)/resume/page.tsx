import { getPayload } from "payload";
import config from "../../../payload.config";
import { redirect } from "next/navigation";

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

export default async function ResumePage() {
  try {
    const payloadConfig = await config;
    const payload = await getPayload({ config: payloadConfig });

    // @ts-ignore - Globals will be typed after first database connection
    const metadata = (await payload.findGlobal({
      slug: "metadata",
    })) as Metadata;

    if (!metadata.resume?.url) {
      throw new Error("Resume URL not found");
    }

    redirect(metadata.resume.url);
  } catch (error) {
    console.error("Failed to acquire resume asset URL", error);
    throw new Error(`Failed to acquire resume (${error})`);
  }
}
