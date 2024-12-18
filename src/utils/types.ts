import type { IconType } from "react-icons";
import { AiFillGithub, AiOutlineLink } from "react-icons/ai";
import { RxOpenInNewWindow } from "react-icons/rx";

export type Project = {
  title: string;
  banner: string;
  bannerSettings?: { quality: number };
  longDescription: string;
  shortDescription: string;
  links?: LinkIcon[];
  location: string;
};

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
