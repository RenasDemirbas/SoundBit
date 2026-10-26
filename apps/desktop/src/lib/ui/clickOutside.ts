// A Svelte action that calls `onClose` on a pointer press outside the element or on Escape,
// used by menus and popovers.

export function clickOutside(node: HTMLElement, onClose: () => void): { update: (next: () => void) => void; destroy: () => void } {
  let close = onClose;
  const onPointer = (event: PointerEvent): void => {
    if (event.target instanceof Node && !node.contains(event.target)) close();
  };
  const onKey = (event: KeyboardEvent): void => {
    if (event.key === 'Escape') close();
  };
  window.addEventListener('pointerdown', onPointer, true);
  window.addEventListener('keydown', onKey);
  return {
    update(next) {
      close = next;
    },
    destroy() {
      window.removeEventListener('pointerdown', onPointer, true);
      window.removeEventListener('keydown', onKey);
    },
  };
}
