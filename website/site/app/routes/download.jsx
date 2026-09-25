import { Link } from "react-router";

export const meta = () => [{ title: "Download · CyborgPL" }];

export default function Download() {
  return (
    <div className="plain">
      <Link to="/more" className="crumb">‹ More</Link>
      <h1 className="h2big">Download</h1>
      <div className="notice">
        <b>Nothing to download yet.</b>
        <p>CyborgPL is in development and has no release. When there is one, it’ll be here.</p>
      </div>
    </div>
  );
}
