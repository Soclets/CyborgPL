import type { Config } from "@react-router/dev/config";

// The CyborgPL site (owner, 2026-10-19: React + Motion, TypeScript, written fresh). Every page is
// built to HTML ahead of time; there is no server, so it is served as static files on Cloudflare.
export default {
  ssr: false,
  async prerender() {
    return ["/"];
  },
} satisfies Config;
