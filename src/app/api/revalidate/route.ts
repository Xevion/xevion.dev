import { NextResponse } from "next/server";
import { revalidatePath } from "next/cache";
import { z } from "zod";

const requestSchema = z.object({
  collection: z.string(),
  doc: z.object({
    id: z.string().or(z.number()),
  }),
});

function getPathsToRevalidate(collection: string): string[] {
  switch (collection) {
    case "projects":
      return ["/projects"];
    case "metadata":
      return ["/"];
    case "technologies":
      return ["/projects"];
    case "links":
      return ["/projects"];
    default:
      return [];
  }
}

export async function POST(req: Request) {
  const revalidateKey = process.env.PAYLOAD_REVALIDATE_KEY;
  const authHeader = req.headers.get("authorization");

  if (!authHeader || authHeader !== `Bearer ${revalidateKey}`) {
    return NextResponse.json({ message: "Invalid token" }, { status: 401 });
  }

  try {
    const body = await req.json();
    const { success, data, error } = requestSchema.safeParse(body);

    if (!success) {
      console.error({ message: "Invalid JSON body", error });
      return NextResponse.json(
        { message: "Invalid JSON body", error },
        { status: 400 },
      );
    }

    const paths = getPathsToRevalidate(data.collection);

    if (paths.length === 0) {
      return NextResponse.json(
        { revalidated: false, message: "No paths to revalidate" },
        { status: 404 },
      );
    }

    // Revalidate all paths
    try {
      for (const path of paths) {
        revalidatePath(path);
      }
    } catch (error) {
      console.error({ message: "Error while revalidating", error });
      return NextResponse.json(
        {
          revalidated: false,
          message: "Error while revalidating",
          paths,
        },
        { status: 500 },
      );
    }

    return NextResponse.json({ revalidated: true, paths }, { status: 200 });
  } catch (error) {
    console.error({
      message: "Error while preparing to revalidate",
      error,
    });
    return NextResponse.json(
      { message: "Error revalidating" },
      { status: 500 },
    );
  }
}
