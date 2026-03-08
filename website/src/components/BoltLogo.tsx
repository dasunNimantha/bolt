export function BoltLogo({ className = "w-8 h-8" }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 48 48"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <rect width="48" height="48" rx="10" fill="#F2BF40" />
      <path d="M27 6L15 27h7.5l-3 15 12-21H24l3-15z" fill="#1A1A1A" />
    </svg>
  );
}
