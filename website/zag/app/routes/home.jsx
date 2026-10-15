export const meta = () => [
  { title: "Zag" },
  { name: "description", content: "Zag, a programming language: very slow at the development, fast & safe at the final product." },
];

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
  // Puffs on a soft base, with a pale underside, as the logo's cloud is built.
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
      <g fill="#DCEBFA">{puffs}</g>
      <g clipPath={`url(#c${id})`}><g fill="#FFFFFF" transform="translate(0,-9)">{puffs}</g></g>
    </svg>
  );
}

export default function Home() {
  return (
    <main className="sky">
      <div className="clouds" aria-hidden="true">
        {CLOUDS.map(([layer, top, w, s, at], i) => (
          <div key={i} className={`cloud ${layer}`}
            style={{ top: `${top}%`, "--w": `${w}px`, animationDuration: `${s}s`, animationDelay: `${-at * s}s` }}>
            <Cloud id={i} />
          </div>
        ))}
      </div>
      <section className="hero">
        <img className="logo" src="/logo.svg" alt="Zag" width="392" height="294" />
        <h1>Zag, a Zig ripoff?</h1>
        <p className="aside">Hm, maybe not...</p>
      </section>
    </main>
  );
}
