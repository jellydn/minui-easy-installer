import { useEffect } from "react";

/**
 * Run an effect only on mount. Use for one-time external sync:
 * DOM integration, third-party widget init, browser API subscriptions.
 *
 * For effects that depend on changing data, use event handlers or
 * a data-fetching library instead.
 */
export function useMountEffect(effect: () => void | (() => void)) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(effect, []);
}
