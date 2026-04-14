export function LogoIcon({ className, height = 20, opacity, strokeWidth = 2, width = 20 }) {
  return (
    <svg
      className={className}
      fill="none"
      height={height}
      opacity={opacity}
      stroke="currentColor"
      strokeWidth={strokeWidth}
      viewBox="0 0 24 24"
      width={width}
    >
      <path d="M9 12l2 2 4-4" />
      <path d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z" />
    </svg>
  );
}

export function PlusIcon({ height = 16, width = 16 }) {
  return (
    <svg fill="none" height={height} stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24" width={width}>
      <line x1="12" x2="12" y1="5" y2="19" />
      <line x1="5" x2="19" y1="12" y2="12" />
    </svg>
  );
}

export function SettingsIcon({ height = 14, width = 14 }) {
  return (
    <svg fill="none" height={height} stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24" width={width}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

export function BackIcon({ height = 14, width = 14 }) {
  return (
    <svg fill="none" height={height} stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24" width={width}>
      <polyline points="15 18 9 12 15 6" />
    </svg>
  );
}

export function CloseIcon({ height = 14, width = 14 }) {
  return (
    <svg fill="none" height={height} stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24" width={width}>
      <line x1="18" x2="6" y1="6" y2="18" />
      <line x1="6" x2="18" y1="6" y2="18" />
    </svg>
  );
}

export function PlayIcon({ height = 14, opacity, strokeWidth = 2, width = 14 }) {
  return (
    <svg
      fill="none"
      height={height}
      opacity={opacity}
      stroke="currentColor"
      strokeWidth={strokeWidth}
      viewBox="0 0 24 24"
      width={width}
    >
      <polygon points="5 3 19 12 5 21 5 3" />
    </svg>
  );
}

export function ExportIcon({ height = 14, width = 14 }) {
  return (
    <svg fill="none" height={height} stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24" width={width}>
      <path d="M12 3v12" />
      <path d="M7 10l5 5 5-5" />
      <path d="M5 21h14" />
    </svg>
  );
}

export function ErrorIcon({ height = 32, width = 32 }) {
  return (
    <svg fill="none" height={height} stroke="var(--color-danger)" strokeWidth="2" viewBox="0 0 24 24" width={width}>
      <circle cx="12" cy="12" r="10" />
      <line x1="15" x2="9" y1="9" y2="15" />
      <line x1="9" x2="15" y1="9" y2="15" />
    </svg>
  );
}

export function ChevronIcon({ className, height = 14, width = 14 }) {
  return (
    <svg
      className={className}
      fill="none"
      height={height}
      stroke="currentColor"
      strokeWidth="2"
      viewBox="0 0 24 24"
      width={width}
    >
      <polyline points="6 9 12 15 18 9" />
    </svg>
  );
}
