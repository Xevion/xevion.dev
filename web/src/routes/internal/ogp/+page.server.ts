import type { PageServerLoad } from "./$types";
import type { OGImageSpec } from "$lib/og-types";
import { error } from "@sveltejs/kit";

export const load: PageServerLoad = async ({ url, parent }) => {
  const parentData = await parent();
  const type = url.searchParams.get("type");

  if (!type) {
    throw error(400, 'Missing "type" query parameter');
  }

  let spec: OGImageSpec;
  let title: string;

  switch (type) {
    case "index":
      spec = { type: "index" };
      title = "Index Page";
      break;
    case "projects":
      spec = { type: "projects" };
      title = "Projects Page";
      break;
    case "project": {
      const id = url.searchParams.get("id");
      if (!id) {
        throw error(400, 'Missing "id" query parameter for project type');
      }
      spec = { type: "project", id };
      title = `Project: ${id}`;
      break;
    }
    default:
      throw error(400, `Invalid "type" query parameter: ${type}`);
  }

  return {
    ...parentData,
    spec,
    title,
  };
};
