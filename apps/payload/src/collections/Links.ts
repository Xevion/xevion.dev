import type { CollectionConfig } from "payload";

export const Links: CollectionConfig = {
  slug: "links",
  admin: {
    useAsTitle: "url",
  },
  fields: [
    {
      name: "url",
      type: "text",
      required: true,
      label: "URL",
    },
    {
      name: "icon",
      type: "text",
      label: "Icon (FontAwesome class)",
    },
    {
      name: "description",
      type: "text",
    },
    {
      name: "project",
      type: "relationship",
      relationTo: "projects",
      required: true,
    },
  ],
};
