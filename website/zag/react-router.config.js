// Every page is built to HTML ahead of time; there is no server. /404 becomes the page the host
// shows for any address it has nothing for (scripts/post-build.mjs). Each example has its own page,
// one per program in the repo's examples/.
import { readExamples } from "./scripts/examples-data.mjs";

export default {
  ssr: false,
  async prerender() {
    return ["/", "/more", "/examples", ...readExamples().map((p) => `/examples/${p.id}`), "/404"];
  },
};
