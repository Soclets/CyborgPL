// The NERVE wordmark (brand/nerve/wordmark-black.svg), in the current text colour. On the site the
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
