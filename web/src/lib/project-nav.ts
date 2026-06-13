import { morph } from "./stores/morph.svelte";
import { telemetry } from "./telemetry";

/**
 * Open a project's detail page: arm the index↔detail morph (shared
 * `view-transition-name`) and record the interaction. Shared by the card and
 * row so the navigation side effects can't drift between them.
 */
export function navigateToProject(project: { slug: string; name: string }) {
  morph.slug = project.slug;
  telemetry.track({
    name: "project_interaction",
    properties: {
      action: "detail_view",
      projectSlug: project.slug,
      projectName: project.name,
    },
  });
}
