import { Link } from "react-router";

// The owner's warning (2026-10-05), in the wording the owner chose ("B"): kept exactly, the bold
// "no warranty" included; "our licences" links to the Licence page. On Home, Licence and Download.
export default function Warranty() {
  return (
    <aside className="warning" role="note" aria-label="Warning">
      <span className="warning-label">Warning</span>
      <p>CyborgPL is unstable by design: any release may change or remove anything. Nothing said about its stability is a guarantee, and it comes with <b>no warranty</b>. See <Link to="/licence" prefetch="intent">our licences</Link>.</p>
    </aside>
  );
}
