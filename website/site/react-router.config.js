import { readdirSync } from "node:fs";
import path from "node:path";

// Every screen is built to HTML ahead of time; there is no server. Loaders run at
// build time and read the repo directly (examples/, the grammar, brand/).
const examples = path.resolve(process.cwd(), "../../examples");
const ids = readdirSync(examples).filter((f) => f.endsWith(".nrv")).map((f) => f.slice(0, -4)).sort();

export default {
  ssr: false,
  async prerender() {
    return ["/", "/learn", "/examples", "/more", "/download", "/licence", "/good-practice", "/commits", "/404", ...ids.map((id) => `/examples/${id}`)];
  },
};
