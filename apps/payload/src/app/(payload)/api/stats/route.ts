import configPromise from "@payload-config";
import { getPayload } from "payload";
import { NextRequest, NextResponse } from "next/server";

// Force this route to be dynamic
export const dynamic = "force-dynamic";

export async function GET(_request: NextRequest) {
  try {
    const payload = await getPayload({ config: configPromise });

    // Fetch statistics about the content
    const [projectsResult, technologiesResult, linksResult] = await Promise.all(
      [
        payload.count({ collection: "projects" }),
        payload.count({ collection: "technologies" }),
        payload.count({ collection: "links" }),
      ],
    );

    // Get featured projects count
    const featuredProjects = await payload.count({
      collection: "projects",
      where: { featured: { equals: true } },
    });

    return NextResponse.json({
      stats: {
        projects: {
          total: projectsResult.totalDocs,
          featured: featuredProjects.totalDocs,
        },
        technologies: technologiesResult.totalDocs,
        links: linksResult.totalDocs,
      },
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    return NextResponse.json(
      {
        error: "Failed to fetch stats",
        details: error instanceof Error ? error.message : "Unknown error",
      },
      { status: 500 },
    );
  }
}
