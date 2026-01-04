import type { CollectionConfig } from "payload";

export const Projects: CollectionConfig = {
  slug: "projects",
  admin: {
    useAsTitle: "name",
    defaultColumns: ["name", "featured", "status", "updatedAt"],
  },
  fields: [
    {
      name: "name",
      type: "text",
      required: true,
    },
    {
      name: "description",
      type: "textarea",
      required: true,
    },
    {
      name: "shortDescription",
      type: "text",
      label: "Short Description",
      required: true,
    },
    {
      name: "icon",
      type: "text",
      label: "Icon (FontAwesome class)",
    },
    {
      name: "status",
      type: "select",
      options: [
        { label: "Draft", value: "draft" },
        { label: "Published", value: "published" },
        { label: "Archived", value: "archived" },
      ],
      defaultValue: "draft",
      required: true,
    },
    {
      name: "featured",
      type: "checkbox",
      label: "Featured Project",
      defaultValue: false,
    },
    {
      name: "autocheckUpdated",
      type: "checkbox",
      label: "Auto-check for GitHub updates",
      defaultValue: false,
      admin: {
        description:
          "Automatically check GitHub for latest commits and update lastUpdated field",
      },
    },
    {
      name: "lastUpdated",
      type: "date",
      label: "Last Updated",
      admin: {
        description:
          "Automatically updated by cron job based on GitHub commits",
        date: {
          displayFormat: "yyyy-MM-dd HH:mm:ss",
        },
      },
    },
    {
      name: "wakatimeOffset",
      type: "number",
      label: "WakaTime Offset",
      admin: {
        description: "Offset for WakaTime fetched data (optional)",
      },
    },
    {
      name: "bannerImage",
      type: "upload",
      relationTo: "media",
      label: "Banner Image",
    },
    {
      name: "technologies",
      type: "relationship",
      relationTo: "technologies",
      hasMany: true,
      label: "Technologies Used",
    },
  ],
};
