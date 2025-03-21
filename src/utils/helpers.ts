/* eslint-disable @typescript-eslint/no-unused-vars */
import create from "@kodingdotninja/use-tailwind-breakpoint";
import resolveConfig from "tailwindcss/resolveConfig";
import tailwindConfig from "@/../tailwind.config.cjs";

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

const config = resolveConfig(tailwindConfig);
export const { useBreakpoint, useBreakpointValue, useBreakpointEffect } =
  create(config.theme!.screens);
