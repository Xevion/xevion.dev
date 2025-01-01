import type { IconType } from "react-icons";
import { AiFillGithub, AiOutlineLink } from "react-icons/ai";
import { RxOpenInNewWindow } from "react-icons/rx";

// Promise.allSettled type guards
export const isFulfilled = <T>(
  p: PromiseSettledResult<T>,
): p is PromiseFulfilledResult<T> => p.status === "fulfilled";
export const isRejected = <T>(
  p: PromiseSettledResult<T>,
): p is PromiseRejectedResult => p.status === "rejected";

export const LinkIcons: Record<string, IconType> = {
  github: AiFillGithub,
  external: RxOpenInNewWindow,
  link: AiOutlineLink,
};
export type LinkIcon = {
  icon: keyof typeof LinkIcons;
  location: string;
  newTab?: boolean;
};
