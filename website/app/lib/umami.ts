export const UMAMI_SCRIPT_URL = "https://umami.astrohacker.com/script.js";
export const UMAMI_WEBSITE_ID = "73a53659-6394-4e00-af8e-76a42bcd2a8b";

export interface Pageview {
  url: string;
  title: string;
  referrer: string;
}

export interface UmamiTracker {
  track: (
    payload: (defaults: Record<string, unknown>) => Record<string, unknown>,
  ) => unknown;
}

declare global {
  interface Window {
    umami?: UmamiTracker;
  }
}

/** One controller survives effect replay; only consecutive equal URLs dedupe. */
export function createPageviews(): {
  visit: (page: Pageview) => void;
  ready: (tracker: UmamiTracker) => void;
  fail: () => void;
} {
  let previous: string | undefined;
  let tracker: UmamiTracker | undefined;
  let pending: Pageview[] = [];
  let failed = false;

  const fail = (): void => {
    failed = true;
    pending = [];
    tracker = undefined;
  };
  const flush = (): void => {
    if (!tracker || failed) return;
    const visits = pending;
    pending = [];
    for (const page of visits) {
      try {
        void Promise.resolve(
          tracker.track((defaults) => ({ ...defaults, ...page })),
        ).catch(fail);
      } catch {
        fail();
        return;
      }
    }
  };
  return {
    visit(page): void {
      if (failed || previous === page.url) return;
      const referrer = previous ?? page.referrer;
      previous = page.url;
      // Analytics is optional: bound memory if a script never finishes loading.
      if (pending.length >= 128) {
        fail();
        return;
      }
      pending.push({ ...page, referrer });
      flush();
    },
    ready(api): void {
      if (failed) return;
      tracker = api;
      flush();
    },
    fail,
  };
}
