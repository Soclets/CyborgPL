import { motion, useReducedMotion } from "motion/react";
import type { ReactNode } from "react";

// The cyborg the loading screen assembles (owner, 2026-10-19: "a Cyborg getting assembled. Each
// 10% = one Cyborg part. Black and white"). Ten parts, bottom up: each is drawn faint and dashed
// as a blueprint from the start, and snaps in solid when its tenth of the site has loaded.
export type Part = { name: string; from: { x?: number; y?: number; scale?: number }; shape: ReactNode };

export const PARTS: Part[] = [
  {
    name: "the left leg", from: { y: 40 },
    shape: <>
      <rect x="92" y="196" width="20" height="54" rx="5" />
      <circle cx="102" cy="255" r="6" />
      <rect x="94" y="262" width="16" height="38" rx="4" />
      <path d="M86 300 H112 V311 H80 Z" />
    </>,
  },
  {
    name: "the right leg", from: { y: 40 },
    shape: <>
      <rect x="128" y="196" width="20" height="54" rx="5" />
      <circle cx="138" cy="255" r="6" />
      <rect x="130" y="262" width="16" height="38" rx="4" />
      <path d="M128 300 H154 L160 311 H128 Z" />
    </>,
  },
  {
    name: "the hips", from: { y: 30 },
    shape: <>
      <path d="M86 180 H154 L146 202 H94 Z" />
      <circle cx="102" cy="194" r="3" />
      <circle cx="138" cy="194" r="3" />
    </>,
  },
  {
    name: "the torso", from: { y: -30 },
    shape: <>
      <path d="M72 96 H168 L158 128 L150 176 H90 L82 128 Z" />
      <path d="M96 146 H112 M128 146 H144 M98 158 H112 M128 158 H142 M100 168 H140" />
    </>,
  },
  {
    name: "the core", from: { scale: 0.2 },
    shape: <>
      <circle cx="120" cy="122" r="17" />
      <circle cx="111" cy="117" r="2.6" fill="#fff" stroke="none" />
      <circle cx="111" cy="127" r="2.6" fill="#fff" stroke="none" />
      <rect x="116" y="115" width="15" height="4" rx="1.5" fill="#fff" stroke="none" />
      <rect x="116" y="125" width="15" height="4" rx="1.5" fill="#fff" stroke="none" />
    </>,
  },
  {
    name: "the left arm", from: { x: -40 },
    shape: <>
      <circle cx="68" cy="104" r="10" />
      <rect x="58" y="116" width="18" height="46" rx="5" />
      <circle cx="67" cy="166" r="5" />
      <rect x="59" y="172" width="16" height="38" rx="4" />
      <path d="M56 212 H78 L76 228 H70 V220 H64 V228 H58 Z" />
    </>,
  },
  {
    name: "the right arm", from: { x: 40 },
    shape: <>
      <circle cx="172" cy="104" r="10" />
      <rect x="164" y="116" width="18" height="46" rx="5" />
      <path d="M166 128 H180 M166 140 H180 M166 152 H180" />
      <circle cx="173" cy="166" r="5" />
      <rect x="165" y="172" width="16" height="38" rx="4" />
      <path d="M162 212 H184 L182 228 H176 V220 H170 V228 H164 Z" />
    </>,
  },
  {
    name: "the neck", from: { y: -24 },
    shape: <>
      <rect x="110" y="78" width="20" height="20" rx="3" />
      <path d="M110 85 H130 M110 91 H130" />
    </>,
  },
  {
    name: "the head", from: { y: -50 },
    shape: <>
      <rect x="90" y="22" width="60" height="58" rx="16" />
      <circle cx="88" cy="52" r="5" />
      <circle cx="152" cy="52" r="5" />
      <path d="M120 22 V12 M116 10 H124" />
      <path d="M104 68 H136" />
    </>,
  },
  {
    name: "the visor", from: { scale: 0.6 },
    shape: <rect x="100" y="42" width="40" height="13" rx="6" fill="#fff" />,
  },
];

// `done` is how many parts are attached, 0 to 10.
export function Cyborg({ done }: { done: number }) {
  const still = useReducedMotion();
  return (
    <svg className="cyborg" viewBox="0 0 240 320" role="img" aria-label={`A cyborg, ${done} of 10 parts attached`}>
      <g className="cyborg-blueprint" aria-hidden="true">
        {PARTS.map((p) => <g key={p.name}>{p.shape}</g>)}
      </g>
      {PARTS.map((p, i) => (
        <motion.g
          key={p.name}
          className="cyborg-part"
          initial={{ opacity: 0, x: p.from.x ?? 0, y: p.from.y ?? 0, scale: p.from.scale ?? 1 }}
          animate={i < done ? { opacity: 1, x: 0, y: 0, scale: 1 } : undefined}
          transition={still ? { duration: 0 } : { type: "spring", stiffness: 420, damping: 24, opacity: { duration: 0.12 } }}
          style={{ transformBox: "fill-box", transformOrigin: "center" }}
        >
          {p.shape}
        </motion.g>
      ))}
    </svg>
  );
}
