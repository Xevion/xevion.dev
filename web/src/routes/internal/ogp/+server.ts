import { ImageResponse } from "@ethercorps/sveltekit-og";
import type { RequestHandler } from "./$types";
import { loadOGFonts } from "$lib/og-fonts";
import { generateOGTemplate } from "$lib/og-template";
import { apiFetch } from "$lib/api";
import type { Project } from "../../projects/+page.server";
import type { OGImageSpec } from "$lib/og-types";
import { getLogger } from "@logtape/logtape";

const logger = getLogger(["ssr", "routes", "internal", "ogp"]);

/**
 * Internal endpoint for OG image generation.
 * Called by Rust server via POST with OGImageSpec JSON body.
 *
 * IMPORTANT: This endpoint should never be accessible externally.
 * It's blocked by the Rust ISR handler's /internal/* check.
 */
export const POST: RequestHandler = async ({ request }) => {
  let spec: OGImageSpec;

  try {
    spec = await request.json();
  } catch {
    logger.warn("Invalid JSON body received");
    return new Response("Invalid JSON body", { status: 400 });
  }

  return await generateOGImage(spec);
};

/**
 * GET handler for OG image generation using query parameters.
 * Supports: ?type=index, ?type=projects, ?type=project&id=<id>
 */
export const GET: RequestHandler = async ({ url }) => {
  const type = url.searchParams.get("type");

  if (!type) {
    logger.warn("Missing 'type' query parameter");
    return new Response("Missing 'type' query parameter", { status: 400 });
  }

  let spec: OGImageSpec;

  switch (type) {
    case "index":
      spec = { type: "index" };
      break;
    case "projects":
      spec = { type: "projects" };
      break;
    case "project": {
      const id = url.searchParams.get("id");
      if (!id) {
        logger.warn("Missing 'id' query parameter for project type");
        return new Response("Missing 'id' query parameter for project type", {
          status: 400,
        });
      }
      spec = { type: "project", id };
      break;
    }
    default:
      logger.warn("Invalid 'type' query parameter", { type });
      return new Response(`Invalid 'type' query parameter: ${type}`, {
        status: 400,
      });
  }

  return await generateOGImage(spec);
};

async function generateOGImage(spec: OGImageSpec): Promise<Response> {
  logger.info("Generating OG image", { spec });

  const templateData = await getTemplateData(spec);

  try {
    const fonts = await loadOGFonts();
    const html = generateOGTemplate(templateData);

    const imageResponse = new ImageResponse(html, {
      width: 1200,
      height: 630,
      fonts,
    });

    const imageBuffer = await imageResponse.arrayBuffer();

    logger.info("OG image generated successfully", { spec });

    return new Response(imageBuffer, {
      status: 200,
      headers: { "Content-Type": "image/png" },
    });
  } catch (error) {
    logger.error("OG image generation failed", {
      spec,
      error: error instanceof Error ? error.message : String(error),
    });
    return new Response("Failed to generate image", { status: 500 });
  }
}

async function getTemplateData(spec: OGImageSpec): Promise<{
  title: string;
  subtitle?: string;
  type?: "default" | "project";
}> {
  switch (spec.type) {
    case "index":
      return {
        title: "Ryan Walters",
        subtitle: "Full-Stack Software Engineer",
        type: "default",
      };
    case "projects":
      return {
        title: "Projects",
        subtitle: "created, maintained, or contributed to by me...",
        type: "default",
      };
    case "project":
      try {
        const projects = await apiFetch<Project[]>("/api/projects");
        const project = projects.find((p) => p.id === spec.id);
        if (project) {
          return {
            title: project.name,
            subtitle: project.shortDescription,
            type: "project",
          };
        }
      } catch (error) {
        logger.error("Failed to fetch project", { id: spec.id, error });
      }
      return {
        title: "Project",
        subtitle: "View on xevion.dev",
        type: "project",
      };
  }
}
