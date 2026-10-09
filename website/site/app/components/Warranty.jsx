import { Link } from "react-router";

// The owner's warning, matched to the README's (owner, 2026-10-09: "match the warning wording"),
// the bold parts included. On the site "this repository" is the GitHub repository and the licence
// names lead to the Licence page. On Home, Licence and Download.
export default function Warranty() {
  return (
    <aside className="warning" role="note" aria-label="Warning">
      <span className="warning-label">Warning</span>
      <p>Nerve is being experimented with, so please don’t expect stability: any release may change or remove anything, and it comes with <b>no warranty</b>. <b>Only the GitHub repository and nerve-lang.com speak for Nerve.</b> If something said elsewhere, for example in an article, a search result or an AI summary, doesn’t match them, you should likely trust the repository and nerve-lang.com, not the other source. We aren’t responsible for what others say. See <Link to="/licence" prefetch="intent">our licences</Link>: <Link to="/licence" prefetch="intent">LICENSE</Link> for the code and everything else, and <Link to="/licence" prefetch="intent">LICENSE-CC-BY-4.0</Link> for the writing (the repository’s Markdown files and this website’s text) and the logos.</p>
    </aside>
  );
}
