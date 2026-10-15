import { Link } from "react-router";
import { useSiteSky } from "../sky.jsx";

export const meta = () => [{ title: "Not here · Zag" }, { name: "robots", content: "noindex" }];

// A page that isn't there, or isn't built yet. Without it the server answered with an empty reply,
// and Safari offered to download it (owner, 2026-10-15: "why is it asking me a permission prompt").
export default function NotFound() {
  useSiteSky();
  return (
    <>
      <main className="lost page-in">
        <div className="lost-body">
          <h1>Nothing here, yet.</h1>
          <p>This page isn’t built, or it never was.</p>
          <Link className="home-link" to="/" prefetch="render">Back to the sky</Link>
        </div>
      </main>
    </>
  );
}
