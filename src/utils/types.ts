import { Github, ExternalLink, Link } from "lucide-react";
import type { LucideIcon } from "lucide-react";

// Promise.allSettled type guards
export const isFulfilled = <T>(
  p: PromiseSettledResult<T>,
): p is PromiseFulfilledResult<T> => p.status === "fulfilled";
export const isRejected = <T>(
  p: PromiseSettledResult<T>,
): p is PromiseRejectedResult => p.status === "rejected";

export const LinkIcons: Record<string, LucideIcon> = {
  github: Github,
  external: ExternalLink,
  link: Link,
};
export type LinkIcon = {
  icon: keyof typeof LinkIcons;
  location: string;
  newTab?: boolean;
};
