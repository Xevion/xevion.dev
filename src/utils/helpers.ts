import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

const isClient = (): boolean => {
  return typeof window !== "undefined";
};

const isServer = (): boolean => {
  return !isClient();
};

const hoverableQuery: MediaQueryList | null = isClient()
  ? window.matchMedia("(hover: hover) and (pointer: fine)")
  : null;

export function isHoverable() {
  return hoverableQuery?.matches;
}
