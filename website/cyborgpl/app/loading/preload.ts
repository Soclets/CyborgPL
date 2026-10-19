// What "loaded" means for the loading screen: every file in /preload.json (written at build time
// by scripts/post-build.ts: the site's code, styles, fonts and images, each with its size) has
// been downloaded, counted byte by byte as it arrives, and then every page's code has been
// started, so no click afterwards waits on the network.

type Entry = { url: string; size: number };
type Manifest = { routes: Record<string, { module: string }> };
declare global {
  interface Window { __reactRouterManifest?: Manifest }
}

// Downloads `url`, reporting each chunk's size. A failure counts as done: the page still shows.
async function pull(url: string, onBytes: (n: number) => void): Promise<void> {
  try {
    const res = await fetch(url);
    const reader = res.body?.getReader();
    if (!reader) return;
    for (;;) {
      const { done, value } = await reader.read();
      if (done) return;
      onBytes(value.byteLength);
    }
  } catch {
    // Unreachable or blocked: carry on without it.
  }
}

// Calls `progress` with a number from 0 to 1 as the site arrives, and resolves when all of it has.
export async function preloadSite(progress: (fraction: number) => void): Promise<void> {
  let list: Entry[] = [];
  try {
    list = await (await fetch("/preload.json")).json();
  } catch {
    // No list (a development server): only the pages' code is loaded below.
  }
  const total = list.reduce((sum, e) => sum + e.size, 0) || 1;
  let got = 0, pages = 0;
  const modules = Object.values(window.__reactRouterManifest?.routes ?? {}).map((r) => r.module);
  // Bytes are 95% of the bar; starting each page's code (from the cache by then) is the last 5%.
  const report = () => progress(Math.min(1, (0.95 * Math.min(got, total)) / total + (0.05 * pages) / Math.max(1, modules.length)));
  await Promise.all(list.map((e) => pull(e.url, (n) => { got += n; report(); })));
  got = total;
  report();
  await Promise.all(modules.map((m) => import(/* @vite-ignore */ m).catch(() => {}).then(() => { pages += 1; report(); })));
  progress(1);
}
