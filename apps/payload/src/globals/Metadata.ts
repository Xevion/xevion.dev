import type { GlobalConfig } from "payload";

export const Metadata: GlobalConfig = {
  slug: "metadata",
  access: {
    read: () => true,
  },
  fields: [
    {
      name: "tagline",
      type: "textarea",
      required: true,
      label: "Site Tagline",
    },
    {
      name: "resume",
      type: "upload",
      relationTo: "media",
      required: true,
      label: "Resume File",
    },
    {
      name: "resumeFilename",
      type: "text",
      label: "Resume Filename Override",
      admin: {
        description:
          'Optional: Override the filename for the resume (e.g., "resume.pdf")',
      },
    },
  ],
};
