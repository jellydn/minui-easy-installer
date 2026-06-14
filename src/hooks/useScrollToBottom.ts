import { useEffect, useRef } from "react";

/**
 * Returns a ref to attach to a scrollable container and a ref for the
 * sentinel element at the bottom. The container scrolls to the sentinel
 * whenever `items` changes.
 */
export function useScrollToBottom<T>(items: T[]) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    sentinelRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [items]);

  return { containerRef, sentinelRef };
}
