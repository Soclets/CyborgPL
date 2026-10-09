// The Nerve icon (brand/nerve/mark-black.svg: a cell body with nerves bursting from it) and the
// NERVE wordmark (brand/nerve/wordmark-black.svg), both in the current text colour. On the site the
// impulse is a see-through cut, so it shows whatever is behind it in either theme. The brand files
// paint it in instead, for places that draw the image without the page behind it.
const LETTERS = [
  "M44 200 V100 H82 L136 166 V100 H164 V200 H128 L74 134 V200 Z",
  "M176 100 H272 V124 H206 V138 H262 V162 H206 V176 H272 V200 H176 Z",
  "M284 200 V100 H360 C384 100 394 114 394 132 C394 146 386 156 374 160 L396 200 H364 L346 164 H314 V200 Z",
  "M408 100 H442 L468 164 L494 100 H528 L486 200 H450 Z",
  "M540 100 H636 V124 H570 V138 H626 V162 H570 V176 H636 V200 H540 Z",
];
const CUT = '<path d="M118 150 H300 L310 150 L322 108 L333 157 L341 157 L347 150 H560" fill="none" stroke="#000" stroke-width="5" stroke-linejoin="round" stroke-linecap="round"/>'
  + '<path d="M12 150 L118 147.6 L118 152.4 Z" fill="#000"/><path d="M560 147.6 L672 149.8 L672 150.2 L560 152.4 Z" fill="#000"/>';

export const wordmarkSVG = (id, height) =>
  `<svg viewBox="44 100 592 100" height="${height}" width="${Math.round(height * 5.92)}" role="img" aria-label="Nerve">`
  + `<defs><mask id="${id}" maskUnits="userSpaceOnUse" x="0" y="0" width="700" height="320"><rect width="700" height="320" fill="#fff"/>${CUT}</mask></defs>`
  + `<g fill="currentColor" mask="url(#${id})">${LETTERS.map((d) => `<path d="${d}"/>`).join("")}</g></svg>`;

const BURST = '<circle cx="90" cy="90" r="20" fill="none" stroke="currentColor" stroke-width="8"/><path d="M90.0 60.0 L90.0 20.0" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="90.0" cy="20.0" r="6" fill="currentColor"/><path d="M107.6 65.7 L120.6 47.9" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="120.6" cy="47.9" r="4.5" fill="currentColor"/><path d="M118.5 80.7 L156.6 68.4" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="156.6" cy="68.4" r="6" fill="currentColor"/><path d="M118.5 99.3 L139.5 106.1" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="139.5" cy="106.1" r="4.5" fill="currentColor"/><path d="M107.6 114.3 L131.1 146.6" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="131.1" cy="146.6" r="6" fill="currentColor"/><path d="M90.0 120.0 L90.0 142.0" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="90.0" cy="142.0" r="4.5" fill="currentColor"/><path d="M72.4 114.3 L48.9 146.6" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="48.9" cy="146.6" r="6" fill="currentColor"/><path d="M61.5 99.3 L40.5 106.1" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="40.5" cy="106.1" r="4.5" fill="currentColor"/><path d="M61.5 80.7 L23.4 68.4" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="23.4" cy="68.4" r="6" fill="currentColor"/><path d="M72.4 65.7 L59.4 47.9" fill="none" stroke="currentColor" stroke-width="7" stroke-linecap="round"/><circle cx="59.4" cy="47.9" r="4.5" fill="currentColor"/><circle cx="90" cy="90" r="7" fill="currentColor"/>';
export const iconSVG = (size) =>
  `<svg viewBox="0 0 180 180" width="${size}" height="${size}" aria-hidden="true">${BURST}</svg>`;
