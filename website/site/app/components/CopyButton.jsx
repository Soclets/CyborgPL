import { useState } from "react";
import Icon from "./Icon.jsx";

// Copies a block's plain text (an example's source or output) and says so for a moment.
export default function CopyButton({ text, label = "Copy" }) {
  const [done, setDone] = useState(false);
  const copy = async () => {
    try { await navigator.clipboard.writeText(text); }
    catch {
      // older browsers, or no permission: a hidden textarea and the old command
      const t = document.createElement("textarea");
      t.value = text; t.setAttribute("readonly", ""); t.style.position = "absolute"; t.style.left = "-9999px";
      document.body.appendChild(t); t.select();
      try { document.execCommand("copy"); } finally { t.remove(); }
    }
    setDone(true);
    setTimeout(() => setDone(false), 1400);
  };
  return (
    <button type="button" className={"copy-btn" + (done ? " done" : "")} onClick={copy} aria-label={done ? "Copied" : label}>
      <Icon name={done ? "check" : "copy"} size={15} />
      <span>{done ? "Copied" : label}</span>
    </button>
  );
}
