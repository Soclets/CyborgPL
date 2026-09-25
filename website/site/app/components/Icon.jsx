// Stroke icons drawn for this site.
const P = {
  home: "M4 11 12 4l8 7v8a1 1 0 0 1-1 1h-5v-6h-4v6H5a1 1 0 0 1-1-1z",
  book: "M5 4h9a3 3 0 0 1 3 3v13H8a3 3 0 0 1-3-3zm0 13a3 3 0 0 1 3-3h9",
  code: "m8 8-4 4 4 4m8-8 4 4-4 4m-2-10-4 16",
  more: "M5 12h.01M12 12h.01M19 12h.01",
  shield: "M12 3 5 6v6c0 4.5 3 7.5 7 9 4-1.5 7-4.5 7-9V6z",
  key: "M11 12h10m-3 0v3m-3-3v2M7.5 15.5a3.5 3.5 0 1 1 0-7 3.5 3.5 0 0 1 0 7z",
  type: "M6.5 9h.01M6.5 15h.01M11 9h7M11 15h7",  // the := glyph itself
  dot: "M4 5h16l-6 7v6l-4 2v-8z",  // a funnel: a dot narrows
  alert: "M12 4 3 20h18zm0 6v4m0 3h.01",
  chip: "M8 8h8v8H8zM4 10h2m-2 4h2m12-4h2m-2 4h2M10 4v2m4-2v2m-4 12v2m4-2v2",
  chev: "m9 6 6 6-6 6",
  search: "M11 18a7 7 0 1 1 0-14 7 7 0 0 1 0 14zm9 2-4-4",
  download: "M12 4v11m-5-5 5 5 5-5M5 20h14",
  x: "M6 6l12 12M18 6 6 18",
  scale: "M12 4v16M5 20h14M7 8h10M7 8l-3 6a3 3 0 0 0 6 0zm10 0-3 6a3 3 0 0 0 6 0z",
  external: "M14 4h6v6m0-6-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5",
  github: "M9 19c-4 1.5-4-2-6-2.5m12 5v-3.5c0-1 .1-1.4-.5-2 2.8-.3 5.5-1.4 5.5-6a4.6 4.6 0 0 0-1.3-3.2 4.2 4.2 0 0 0-.1-3.2s-1.1-.3-3.5 1.3a12 12 0 0 0-6.2 0C6.5 2.8 5.4 3.1 5.4 3.1a4.2 4.2 0 0 0-.1 3.2A4.6 4.6 0 0 0 4 9.5c0 4.6 2.7 5.7 5.5 6-.6.6-.6 1.2-.5 2V21",
};
export default function Icon({ name, size = 22, stroke = 2 }) {
  return (
    <svg className="icon" viewBox="0 0 24 24" width={size} height={size} fill="none" stroke="currentColor"
      strokeWidth={stroke} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={P[name]} />
    </svg>
  );
}
