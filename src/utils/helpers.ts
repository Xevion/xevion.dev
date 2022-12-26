import create from "@kodingdotninja/use-tailwind-breakpoint";
import resolveConfig from "tailwindcss/resolveConfig";
import tailwindConfig from "./../../tailwind.config.cjs";

export function classNames(...classes: (string | null | undefined)[]) {
    return classes.filter(Boolean).join(" ");
}

/**
 * A handler that simply calls the `stopPropagation` method on an event.
 * @param event The event to prevent propagation on.
 */
export const stopPropagation = (event: Event) => {
    event.stopPropagation();
};

const isClient = (): boolean => {
    return typeof window !== "undefined";
}

const isServer = (): boolean => {
    return !isClient();
}

const hoverableQuery: MediaQueryList | null = isClient() ? window.matchMedia('(hover: hover) and (pointer: fine)') : null;

export function isHoverable() {
    return hoverableQuery?.matches;
}


const config = resolveConfig(tailwindConfig);
export const {useBreakpoint, useBreakpointValue, useBreakpointEffect} = create(config.theme!.screens);