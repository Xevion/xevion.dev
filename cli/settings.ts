import { authedClient, loadConfig } from "./auth.ts";
import type { CommandNode, Context } from "./framework.ts";
import * as output from "./output.ts";
import type { ApiSiteSettings } from "./types.ts";

export const settingsCommand: CommandNode = {
  name: "settings",
  description: "Site settings management",
  children: [
    {
      name: "get",
      description: "Get current site settings",
      run: async (_p, ctx) => {
        const settings =
          await clientFor(ctx).get<ApiSiteSettings>("/api/settings");
        if (ctx.json) output.printJson(settings);
        else output.printSettings(settings);
      },
    },
    {
      name: "update",
      description: "Update site settings",
      flags: {
        displayName: {
          type: String,
          description: "Display name",
          placeholder: "<name>",
        },
        occupation: {
          type: String,
          description: "Occupation/title",
          placeholder: "<text>",
        },
        bio: { type: String, description: "Bio text", placeholder: "<text>" },
        siteTitle: {
          type: String,
          description: "Site title",
          placeholder: "<title>",
        },
      },
      run: async ({ flags }, ctx) => {
        const client = clientFor(ctx);
        const current = await client.get<ApiSiteSettings>("/api/settings");
        const request = {
          identity: {
            displayName: flags.displayName ?? current.identity.displayName,
            occupation: flags.occupation ?? current.identity.occupation,
            bio: flags.bio ?? current.identity.bio,
            siteTitle: flags.siteTitle ?? current.identity.siteTitle,
          },
          // Existing social links are kept unchanged.
          socialLinks: current.socialLinks.map((link) => ({
            id: link.id,
            platform: link.platform,
            label: link.label,
            value: link.value,
            icon: link.icon,
            visible: link.visible,
            displayOrder: link.displayOrder,
          })),
        };
        const settings = await client.put<ApiSiteSettings>(
          "/api/settings",
          request,
        );
        if (ctx.json) {
          output.printJson(settings);
        } else {
          output.success("Updated site settings");
          output.printSettings(settings);
        }
      },
    },
  ],
};

function clientFor(ctx: Context) {
  return authedClient(loadConfig(ctx), ctx.api);
}
