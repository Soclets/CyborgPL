import { Link } from "react-router";
import { LICENCE } from "../lib/content.js";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "Licence · CyborgPL", description: "CyborgPL’s code is Apache-2.0 with the LLVM exception; this site’s text is CC BY 4.0; the names and logo are trademarks.", path: "/licence" });

export default function Licence() {
  return (
    <div className="plain">
      <Link to="/more" className="crumb">‹ More</Link>
      <h1 className="h2big">Licence</h1>
      <div className="stack">
        {LICENCE.map((l) => (
          <article key={l.title} className="lesson"><b>{l.title}</b><p>{l.text}</p></article>
        ))}
      </div>
    </div>
  );
}
