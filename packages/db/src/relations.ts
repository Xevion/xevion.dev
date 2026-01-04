// Custom Drizzle relations that extend the auto-generated Payload schema
// This file adds the reverse relationships that Payload doesn't auto-generate

import { relations } from "drizzle-orm";
import { projects, links, technologies, projects_rels, media } from "./schema";

// Override the auto-generated relations_projects to add links
export const projectsRelations = relations(projects, ({ one, many }) => ({
  bannerImage: one(media, {
    fields: [projects.bannerImage],
    references: [media.id],
    relationName: "bannerImage",
  }),
  _rels: many(projects_rels, {
    relationName: "_rels",
  }),
  // Add the reverse relationship for links
  links: many(links, {
    relationName: "project",
  }),
}));

// Note: Other relations are auto-generated and exported from schema.ts
// We only override the ones that need custom reverse relationships
