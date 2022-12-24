export function classNames(...classes: (string | null | undefined)[]) {
    return classes.filter(Boolean).join(" ");
}

const hoverableQuery = window.matchMedia('(hover: hover) and (pointer: fine)');

export function isHoverable() {
    return hoverableQuery.matches;
}