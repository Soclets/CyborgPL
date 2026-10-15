import { Backdrop, Pill } from "../sky.jsx";

export const meta = () => [{ title: "Not here · Zag" }, { name: "robots", content: "noindex" }];

// A page that isn't there, or isn't built yet. Without it the server answered with an empty reply,
// and Safari offered to download it (owner, 2026-10-15: "why is it asking me a permission prompt").
export default function NotFound() {
  return (
    <>
      <Backdrop />
      <main className="lost">
        <Pill here="" />
        <div className="lost-body">
          <h1>Nothing here, yet.</h1>
          <p>This page isn’t built, or it never was.</p>
          <a className="home-link" href="/">Back to the sky</a>
        </div>
      </main>
    </>
  );
}
