// The Snap Pea mark (a tilted pod with three peas) and the "snappea" wordmark.
// Colors come from --pod and --pea in ../styles.css; the favicons in the HTML pages
// are the same drawing.

export function PeaMark({ size = 34 }: { size?: number }) {
  return (
    <svg className="pea-mark" width={size} height={(size * 96) / 132} viewBox="0 -12 132 96" aria-hidden="true">
      <g transform="rotate(-18 64 36)">
        <path d="M119 34 Q129 27 126 16" fill="none" stroke="var(--pod)" strokeWidth="5" strokeLinecap="round" />
        <path d="M8 36 A72 72 0 0 1 120 36 A72 72 0 0 1 8 36 Z" fill="var(--pod)" />
        <circle cx="41" cy="36" r="9" fill="var(--pea)" />
        <circle cx="64" cy="36" r="11" fill="var(--pea)" />
        <circle cx="87" cy="36" r="9.5" fill="var(--pea)" />
        <g fill="#fff" fillOpacity="0.85">
          <circle cx="38.8" cy="31.8" r="2.7" />
          <circle cx="61.4" cy="30.8" r="3.3" />
          <circle cx="84.7" cy="31.5" r="2.85" />
        </g>
      </g>
    </svg>
  );
}

/** Mark and wordmark, for headers. */
export function Logo() {
  return (
    <span className="logo">
      <PeaMark />
      <span className="wordmark">
        snap<span>pea</span>
      </span>
    </span>
  );
}
