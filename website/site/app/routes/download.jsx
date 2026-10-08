import { Link } from "react-router";
import Warranty from "../components/Warranty.jsx";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "Download · Nerve", description: "Nothing to download yet: Nerve is in development and has no release.", path: "/download" });

export default function Download() {
  return (
    <div className="plain">
      <Link to="/more" className="crumb">‹ More</Link>
      <h1 className="h2big">Download</h1>
      <div className="notice">
        <b>Nothing to download yet.</b>
        <p>Nerve is in development and has no release. When there is one, it’ll be here.</p>
      </div>
      <Warranty />
    </div>
  );
}
