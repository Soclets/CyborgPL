// The brand tile, drawn from brand/logo.svg's geometry (white mark, black rounded square).
export default function Logo({ size = 34 }) {
  return (
    <svg viewBox="0 0 32 32" width={size} height={size} role="img" aria-label="CyborgPL">
      <rect x="0.75" y="0.75" width="30.5" height="30.5" rx="7.25" fill="#000" stroke="#fff" strokeWidth="1.5" />
      <g transform="translate(16 16) scale(0.78) translate(-16 -16)">
        <g transform="translate(1.3 0)">
          <path d="M22.88 6.17 A12 12 0 1 0 22.88 25.83" fill="none" stroke="#fff" strokeWidth="4.5" strokeLinecap="round" />
          <circle cx="12.15" cy="13.34" r="1.54" fill="#fff" />
          <circle cx="12.15" cy="18.66" r="1.54" fill="#fff" />
          <rect x="14.95" y="12.29" width="5.95" height="2.1" rx="0.56" fill="#fff" />
          <rect x="14.95" y="17.61" width="5.95" height="2.1" rx="0.56" fill="#fff" />
        </g>
      </g>
    </svg>
  );
}
