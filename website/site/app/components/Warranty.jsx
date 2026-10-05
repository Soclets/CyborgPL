import { Link } from "react-router";

// The owner's words, as in the README's warning (6f52e33, 2026-10-05): kept exactly, the bold
// "does not" included; "our licenses" links to the Licence page. On Home, Licence and Download.
export default function Warranty() {
  return (
    <aside className="warning" role="note" aria-label="Warning">
      <span className="warning-label">Warning</span>
      <p>Any marketing of CyborgPL’s stability <b>does not</b> actually guarantee anything, CyborgPL does not have any warranties. We recommend looking at <Link to="/licence" prefetch="intent">our licenses</Link>.</p>
    </aside>
  );
}
