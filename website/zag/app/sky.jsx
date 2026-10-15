import { useEffect, useRef } from "react";
import { Link, useLocation } from "react-router";

// The sky behind every page, and the site's top bar.

// The sky (owner, 2026-10-15): "You land on the website, it has that blue sky with clouds
// background moving, and with a message 'Zag, a Zig ripoff? Hm, maybe not...'"
// Three layers of clouds drift across at different speeds, the far ones small, faint and slow.
// [layer, top %, width px, seconds to cross, how far along it starts (0–1)]
const CLOUDS = [
  ["far", 8, 150, 170, 0.05], ["far", 18, 120, 190, 0.42], ["far", 12, 170, 160, 0.71], ["far", 26, 130, 200, 0.9],
  ["mid", 34, 230, 110, 0.18], ["mid", 46, 270, 125, 0.55], ["mid", 40, 210, 100, 0.83],
  ["near", 62, 380, 70, 0.3], ["near", 74, 440, 80, 0.68], ["near", 68, 340, 65, 0.97],
];

function Cloud({ id }) {
  // Puffs on a soft base, with a pale underside, as the logo's cloud is built. The colours are
  // CSS variables, so the clouds darken as the storm comes in.
  const puffs = (
    <>
      <circle cx="70" cy="64" r="34" /><circle cx="122" cy="44" r="44" /><circle cx="176" cy="62" r="32" />
      <circle cx="48" cy="92" r="28" /><circle cx="100" cy="104" r="28" /><circle cx="152" cy="104" r="30" /><circle cx="200" cy="92" r="26" />
      <ellipse cx="124" cy="84" rx="96" ry="34" />
    </>
  );
  return (
    <svg viewBox="0 0 248 140" aria-hidden="true">
      <defs><clipPath id={`c${id}`}>{puffs}</clipPath></defs>
      <g className="shade">{puffs}</g>
      <g clipPath={`url(#c${id})`}><g className="lit" transform="translate(0,-9)">{puffs}</g></g>
    </svg>
  );
}

// The backdrop stays put while the page moves over it: the sky, the storm that fades in over it,
// the clouds, and the lightning's flash.
export function Backdrop() {
  // The sky holds still while you're away (owner, 2026-10-15: "Make it so that the background stops
  // moving, after the user exits the screen", then "number 1": leaving the tab or the window), and
  // carries on from where it was when you come back.
  useEffect(() => {
    const root = document.documentElement;
    const update = () => root.classList.toggle("away", document.hidden || !document.hasFocus());
    update();
    addEventListener("visibilitychange", update);
    addEventListener("blur", update);
    addEventListener("focus", update);
    addEventListener("pagehide", update);
    addEventListener("pageshow", update);
    return () => {
      removeEventListener("visibilitychange", update); removeEventListener("blur", update); removeEventListener("focus", update);
      removeEventListener("pagehide", update); removeEventListener("pageshow", update); root.classList.remove("away");
    };
  }, []);
  return (
    <div className="backdrop" aria-hidden="true">
      <div className="storm" />
      <div className="clouds">
        {["far", "mid", "near"].map((layer) => (
          <div key={layer} className={`layer ${layer}`}>
            {CLOUDS.map(([l, top, w, s, at], i) => l === layer && (
              <div key={i} className={`cloud ${layer}`}
                style={{ top: `${top}%`, "--w": `${w}px`, animationDuration: `${s}s`, animationDelay: `${-at * s}s` }}>
                <Cloud id={i} />
              </div>
            ))}
          </div>
        ))}
      </div>
      <div className="flash" />
    </div>
  );
}

// The site's top bar: a floating pill of frosted glass (owner: "I think it should be an in-theme
// pill", then "Glass. Remove Commits. new, exact arrangement: Home, Download, Docs, Examples, More").
// Home is the site's own screen, after the intro (owner: "Clicking home should tp back to Home, not
// the first scroll"): /#home. On the home page itself a click glides there (home.jsx, usePager).
const LINKS = [["Home", "/#home"], ["Download", "/download"], ["Docs", "/docs"], ["Examples", "/examples"], ["More", "/more"]];
export function Pill() {
  const { pathname } = useLocation();
  const here = pathname === "/" ? "/#home" : pathname;
  // One white highlight slides from the last page's button to this one's (owner: "The pill
  // buttons should slide when clicked"). It is placed without sliding the first time, so it
  // doesn't fly in from the left as the page opens.
  const links = useRef(null), thumb = useRef(null), placed = useRef(false);
  useEffect(() => {
    const place = () => {
      const on = links.current.querySelector('[aria-current="page"]'), t = thumb.current;
      t.style.opacity = on ? "1" : "0";
      if (!on) return;
      t.style.setProperty("--x", `${on.offsetLeft}px`);
      t.style.setProperty("--w", `${on.offsetWidth}px`);
      if (!placed.current) {
        placed.current = true;
        requestAnimationFrame(() => links.current?.parentElement.classList.add("ready"));
      }
    };
    place();
    document.fonts?.ready.then(place);
    addEventListener("resize", place);
    return () => removeEventListener("resize", place);
  }, [here]);
  return (
    <nav className="pill" aria-label="Site">
      {/* The logo goes to the start of the intro, the first screen (owner, 2026-10-15). */}
      <Link className="brand" to="/" aria-label="Zag, from the start"><img src="/logo.svg" alt="" width="45" height="34" /></Link>
      <div className="links" ref={links}>
        <span className="thumb" ref={thumb} aria-hidden="true" />
        {LINKS.map(([name, to]) => (
          <Link key={to} className="link" to={to} aria-current={to === here ? "page" : undefined}>{name}</Link>
        ))}
      </div>
    </nav>
  );
}

// Where the site's own screen is in the sky: the home page's third screen. Every other page sits
// at the same height, with the storm cleared and the pill shown, so the clouds line up exactly
// when you go between them.
export const SITE_SCREEN = 2;
export function useSiteSky() {
  useEffect(() => {
    const root = document.documentElement;
    root.style.setProperty("--y", String(SITE_SCREEN));
    root.style.setProperty("--storm", "0");
    root.style.setProperty("--site-fade", "1");
    root.classList.remove("pill-off");
  }, []);
}
