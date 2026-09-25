import { Link } from "react-router";
import { LICENCE } from "../lib/content.js";

export const meta = () => [{ title: "Licence · CyborgPL" }];

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
