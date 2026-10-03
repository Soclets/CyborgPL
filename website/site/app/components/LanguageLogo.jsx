import { LANGUAGE_ARTWORK } from "../lib/language-artwork.js";

// Unaltered artwork, fitted proportionally on a white tile for both themes.
// The language name is adjacent, so the image is decorative for screen readers.
export default function LanguageLogo({ id, className = "" }) {
  const art = LANGUAGE_ARTWORK[id];
  return (
    <span className={`language-logo ${className}`} aria-hidden="true" data-language={id}>
      <img src={art.asset} width="24" height="24" alt="" />
      {art.trademark && <sup>™</sup>}
    </span>
  );
}
