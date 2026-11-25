// Tracks which project is mid-navigation so the index card and the detail hero
// can share a `view-transition-name` and morph into each other. Set on card
// click (forward) and on detail mount (so back-navigation morphs in reverse).
export const morph = $state<{ slug: string | null }>({ slug: null });
