/**
 * Svelte action that moves an element to a target container (default: document.body).
 * This allows elements to escape their parent's stacking context, which is essential
 * for modals that need to appear above all other content regardless of z-index.
 *
 * @example
 * ```svelte
 * <div use:portal class="fixed inset-0 z-50">
 *   <!-- Modal content renders at document.body -->
 * </div>
 * ```
 *
 * @example
 * ```svelte
 * <!-- Portal to a specific container -->
 * <div use:portal={"#modal-container"}>
 *   ...
 * </div>
 * ```
 */
export function portal(
  node: HTMLElement,
  target: HTMLElement | string = document.body,
): { destroy: () => void } | void {
  const targetEl =
    typeof target === "string" ? document.querySelector(target) : target;

  if (!targetEl) {
    console.warn(`Portal target "${target}" not found`);
    return;
  }

  targetEl.appendChild(node);

  return {
    destroy() {
      node.remove();
    },
  };
}
