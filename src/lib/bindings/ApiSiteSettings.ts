import type { ApiSiteIdentity } from "./ApiSiteIdentity";
import type { ApiSocialLink } from "./ApiSocialLink";

export type ApiSiteSettings = {
  identity: ApiSiteIdentity;
  socialLinks: Array<ApiSocialLink>;
};
