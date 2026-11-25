/**
 * Compact view of a related project, resolved for the detail page's "Related
 * work" list. Carries just what the row renders: name, slug, the authored
 * `projectType`, and the accent (for the generated cover thumbnail).
 */
export type ApiRelatedProject = {
  id: string;
  slug: string;
  name: string;
  projectType?: string;
  accentColor?: string;
};
