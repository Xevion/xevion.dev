import { nonEmpty, parseHexColor } from "./args.ts";
import { authedClient, loadConfig } from "./auth.ts";
import type { ApiClient } from "./client.ts";
import type { CommandNode, Context } from "./framework.ts";
import * as output from "./output.ts";
import type { ApiTag, ApiTagWithCount } from "./types.ts";

const seg = encodeURIComponent;

interface TagDetail {
  tag: ApiTag;
  projects: unknown[];
}

function clientFor(ctx: Context): ApiClient {
  return authedClient(loadConfig(ctx), ctx.api);
}

/** Empty string clears, absent keeps the current value. */
function merge(flag: string | undefined, current: string | null | undefined) {
  if (flag === undefined) return current ?? null;
  return flag === "" ? null : flag;
}

export const tagsCommand: CommandNode = {
  name: "tags",
  description: "Tag management",
  children: [
    {
      name: "list",
      description: "List all tags with project counts",
      run: async (_p, ctx) => {
        const tags = await clientFor(ctx).get<ApiTagWithCount[]>("/api/tags");
        if (ctx.json) output.printJson(tags);
        else output.printTagsTable(tags);
      },
    },
    {
      name: "get",
      description: "Get tag details with associated projects (slug or UUID)",
      parameters: ["<ref>"],
      run: async ({ _ }, ctx) => {
        const detail = await clientFor(ctx).get<TagDetail>(
          `/api/tags/${seg(String(_.ref))}`,
        );
        if (ctx.json) {
          output.printJson({ tag: detail.tag, projects: detail.projects });
          return;
        }
        output.printTag(detail.tag);
        if (detail.projects.length > 0) {
          output.info(`${detail.projects.length} associated project(s)`);
        }
      },
    },
    {
      name: "create",
      description: "Create a new tag",
      parameters: ["<name>"],
      flags: {
        slug: {
          type: String,
          description: "URL slug (auto-generated from name if omitted)",
          placeholder: "<slug>",
        },
        icon: {
          type: String,
          description: 'Icon identifier (e.g., "simple-icons:rust")',
          placeholder: "<icon>",
        },
        color: {
          type: String,
          description: 'Color hex, e.g. "3b82f6" or "#3b82f6"',
          placeholder: "<hex>",
        },
      },
      run: async ({ _, flags }, ctx) => {
        const color =
          flags.color === undefined ? undefined : parseHexColor(flags.color);
        const tag = await clientFor(ctx).post<ApiTag>("/api/tags", {
          name: String(_.name),
          slug: flags.slug ?? null,
          icon: nonEmpty(flags.icon) ?? null,
          color: nonEmpty(color) ?? null,
        });
        if (ctx.json) {
          output.printJson(tag);
        } else {
          output.success(`Created tag: ${tag.name}`);
          output.printTag(tag);
        }
      },
    },
    {
      name: "update",
      description: "Update an existing tag",
      parameters: ["<ref>"],
      flags: {
        name: {
          type: String,
          alias: "n",
          description: "Tag name",
          placeholder: "<name>",
        },
        slug: {
          type: String,
          description: "New URL slug",
          placeholder: "<slug>",
        },
        icon: {
          type: String,
          description: 'Icon identifier (use "" to clear)',
          placeholder: "<icon>",
        },
        color: {
          type: String,
          description:
            'Color hex, e.g. "3b82f6" or "#3b82f6" (use "" to clear)',
          placeholder: "<hex>",
        },
      },
      run: async ({ _, flags }, ctx) => {
        const color =
          flags.color === undefined ? undefined : parseHexColor(flags.color);
        const reference = String(_.ref);
        const client = clientFor(ctx);
        const current = await client.get<TagDetail>(
          `/api/tags/${seg(reference)}`,
        );
        const tag = await client.put<ApiTag>(`/api/tags/${seg(reference)}`, {
          name: flags.name ?? current.tag.name,
          slug: flags.slug ?? null,
          icon: merge(flags.icon, current.tag.icon),
          color: merge(color, current.tag.color),
        });
        if (ctx.json) {
          output.printJson(tag);
        } else {
          output.success(`Updated tag: ${tag.name}`);
          output.printTag(tag);
        }
      },
    },
    {
      name: "delete",
      description: "Delete a tag",
      parameters: ["<ref>"],
      run: async ({ _ }, ctx) => {
        const deleted = await clientFor(ctx).delete<ApiTag>(
          `/api/tags/${seg(String(_.ref))}`,
        );
        if (ctx.json) output.printJson(deleted);
        else output.success(`Deleted tag: ${deleted.name}`);
      },
    },
  ],
};
