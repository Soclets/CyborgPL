// The Nerve mark (brand/mark-black.svg) in the current text colour.
export default function Mark({ size = 26 }) {
  return (
    <svg viewBox="0 0 32 32" width={size} height={size} aria-hidden="true">
      <g transform="translate(1.3 0)" fill="currentColor">
        <path d="M22.88 6.17 A12 12 0 1 0 22.88 25.83" fill="none" stroke="currentColor" strokeWidth="4.5" strokeLinecap="round" />
        <circle cx="12.15" cy="13.34" r="1.54" /><circle cx="12.15" cy="18.66" r="1.54" />
        <rect x="14.95" y="12.29" width="5.95" height="2.1" rx="0.56" /><rect x="14.95" y="17.61" width="5.95" height="2.1" rx="0.56" />
      </g>
    </svg>
  );
}
