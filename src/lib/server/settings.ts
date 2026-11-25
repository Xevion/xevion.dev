import { type } from "arktype";
import { asc, eq, inArray } from "drizzle-orm";
import type { BatchItem } from "drizzle-orm/batch";
import type {
  ApiSiteIdentity,
  ApiSiteSettings,
  ApiSocialLink,
} from "#lib/bindings/index.ts";
import { apiError } from "./api.ts";
import type { Db } from "./db/index.ts";
import { siteIdentity, socialLinks } from "./db/schema.ts";

export const SettingsInputSchema = type({
  identity: {
    displayName: "string",
    occupation: "string",
    bio: "string",
    siteTitle: "string",
  },
  socialLinks: type({
    id: "string",
    platform: "string",
    label: "string",
    value: "string",
    icon: "string",
    visible: "boolean",
    displayOrder: "number.integer",
  }).array(),
});

type SettingsInput = typeof SettingsInputSchema.infer;
type IdentityRow = typeof siteIdentity.$inferSelect;

const FALLBACK_IDENTITY: Omit<IdentityRow, "id" | "updatedAt"> = {
  displayName: "Ryan Walters",
  occupation: "Software Engineer",
  bio: "Software engineer and developer",
  siteTitle: "xevion.dev",
  resumeR2Key: null,
  resumeUploadedAt: null,
};

export function toApiIdentity(
  row: Pick<
    IdentityRow,
    | "displayName"
    | "occupation"
    | "bio"
    | "siteTitle"
    | "resumeR2Key"
    | "resumeUploadedAt"
  >,
  mediaBaseUrl: string,
): ApiSiteIdentity {
  return {
    displayName: row.displayName,
    occupation: row.occupation,
    bio: row.bio,
    siteTitle: row.siteTitle,
    resumeUrl: row.resumeR2Key ? `${mediaBaseUrl}/${row.resumeR2Key}` : null,
    resumeUploadedAt: row.resumeUploadedAt?.toISOString() ?? null,
  };
}

function toApiSocialLink(row: typeof socialLinks.$inferSelect): ApiSocialLink {
  return {
    id: row.id,
    platform: row.platform,
    label: row.label,
    value: row.value,
    icon: row.icon,
    visible: row.visible,
    displayOrder: row.displayOrder,
  };
}

export async function getSettings(
  db: Db,
  mediaBaseUrl: string,
): Promise<ApiSiteSettings> {
  const [identity, links] = await Promise.all([
    db.select().from(siteIdentity).where(eq(siteIdentity.id, 1)).get(),
    db.select().from(socialLinks).orderBy(asc(socialLinks.displayOrder)),
  ]);
  return {
    identity: toApiIdentity(identity ?? FALLBACK_IDENTITY, mediaBaseUrl),
    socialLinks: links.map(toApiSocialLink),
  };
}

/** Replaces the identity fields and updates each listed social link by id, atomically. */
export async function updateSettings(
  db: Db,
  input: SettingsInput,
  mediaBaseUrl: string,
): Promise<ApiSiteSettings> {
  const ids = input.socialLinks.map((l) => l.id);
  if (ids.length > 0) {
    const known = new Set(
      (
        await db
          .select({ id: socialLinks.id })
          .from(socialLinks)
          .where(inArray(socialLinks.id, ids))
      ).map((r) => r.id),
    );
    if (ids.some((id) => !known.has(id)))
      apiError(404, "NOT_FOUND", "Social link not found");
  }

  const identity = {
    ...input.identity,
    updatedAt: new Date(),
  };
  const statements: BatchItem<"sqlite">[] = [
    db
      .insert(siteIdentity)
      .values({ id: 1, ...identity })
      .onConflictDoUpdate({ target: siteIdentity.id, set: identity }),
    ...input.socialLinks.map(({ id, ...link }) =>
      db.update(socialLinks).set(link).where(eq(socialLinks.id, id)),
    ),
  ];
  await db.batch(statements as [BatchItem<"sqlite">, ...BatchItem<"sqlite">[]]);
  return getSettings(db, mediaBaseUrl);
}

export async function getResumeKey(db: Db): Promise<string | null> {
  const row = await db
    .select({ key: siteIdentity.resumeR2Key })
    .from(siteIdentity)
    .where(eq(siteIdentity.id, 1))
    .get();
  return row?.key ?? null;
}

/** Points the site at a newly uploaded resume and returns the updated identity. */
export async function setResumeKey(
  db: Db,
  key: string,
  mediaBaseUrl: string,
): Promise<ApiSiteIdentity> {
  const row = await db
    .update(siteIdentity)
    .set({ resumeR2Key: key, resumeUploadedAt: new Date() })
    .where(eq(siteIdentity.id, 1))
    .returning()
    .get();
  if (!row) apiError(404, "NOT_FOUND", "Site settings not found");
  return toApiIdentity(row, mediaBaseUrl);
}
