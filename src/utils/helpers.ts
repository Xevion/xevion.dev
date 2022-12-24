export function classNames(...classes: (string | null | undefined)[]) {
    return classes.filter(Boolean).join(" ");
}

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