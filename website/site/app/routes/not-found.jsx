import { Link } from "react-router";

export const meta = () => [{ title: "Not found · CyborgPL" }];

export default function NotFound() {
  return (
    <main className="page">
      <div className="plain">
        <h1 className="h2big">Nothing here.</h1>
        <p className="lede">That page doesn’t exist.</p>
        <Link className="btn primary" to="/">Go home</Link>
      </div>
    </main>
  );
}
