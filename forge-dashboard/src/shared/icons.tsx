import React from 'react';

export function GeneralIcon({ active, ...props }: React.SVGProps<SVGSVGElement> & { active?: boolean }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke={active ? "url(#neon-orange)" : "currentColor"} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <path d="M3 12h4l3-8 4 16 3-8h4" />
    </svg>
  );
}

export function GraphIcon({ active, ...props }: React.SVGProps<SVGSVGElement> & { active?: boolean }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke={active ? "url(#neon-orange)" : "currentColor"} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <circle cx="7" cy="17" r="2.5" />
      <circle cx="17" cy="7" r="2.5" />
      <path d="M7 5v9.5" />
      <path d="M7 14.5c0-4 10-1 10-5" />
    </svg>
  );
}

export function CacheIcon({ active, ...props }: React.SVGProps<SVGSVGElement> & { active?: boolean }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke={active ? "url(#neon-orange)" : "currentColor"} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <ellipse cx="12" cy="6" rx="8" ry="3" />
      <path d="M4 6v12c0 1.66 3.58 3 8 3s8-1.34 8-3V6" />
      <path d="M4 12c0 1.66 3.58 3 8 3s8-1.34 8-3" />
    </svg>
  );
}

export function SettingsIcon({ active, ...props }: React.SVGProps<SVGSVGElement> & { active?: boolean }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke={active ? "url(#neon-orange)" : "currentColor"} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" {...props}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}
