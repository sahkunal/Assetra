/** Thin concentric arcs behind the hero — a record-groove motif. */
export default function Arcs() {
  const rings = Array.from({ length: 46 }, (_, i) => 260 + i * 38);
  return (
    <svg
      aria-hidden
      className="pointer-events-none absolute inset-0 h-full w-full"
      viewBox="0 0 1600 900"
      preserveAspectRatio="xMaxYMax slice"
      fill="none"
    >
      {rings.map((r, i) => (
        <circle
          key={r}
          cx="1750"
          cy="1050"
          r={r}
          stroke="white"
          strokeOpacity={Math.max(0.03, 0.12 - i * 0.002)}
          strokeWidth="1"
        />
      ))}
    </svg>
  );
}
