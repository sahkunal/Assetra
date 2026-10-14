/** Round mark: the circle is the record, the bar is the split (tokenized slice). */
export default function Logo({ className = "h-14 w-14" }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" className={className} fill="none" aria-hidden>
      <circle cx="32" cy="32" r="29" stroke="#1d1a22" strokeWidth="2" />
      <line x1="3" y1="32" x2="61" y2="32" stroke="#1d1a22" strokeWidth="2" />
      <text x="32" y="25" textAnchor="middle" fontSize="19" fontFamily="inherit" fill="#1d1a22">
        A
      </text>
      <text x="32" y="51" textAnchor="middle" fontSize="19" fontFamily="inherit" fill="#1d1a22">
        $
      </text>
    </svg>
  );
}
