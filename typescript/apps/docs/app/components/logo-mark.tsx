type LogoMarkProps = {
  className?: string;
};

export function LogoMark({ className }: LogoMarkProps) {
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      width="24"
      height="24"
      viewBox="0 0 100 100"
      className={className}
    >
      <rect width="100" height="100" rx="23" fill="#17181B" />
      <path
        d="M34 22C25 22 20 27 20 36V42C20 47 17 50 12 50C17 50 20 53 20 58V64C20 73 25 78 34 78"
        fill="none"
        stroke="#76CE54"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="9"
      />
      <path
        d="M66 22C75 22 80 27 80 36V42C80 47 83 50 88 50C83 50 80 53 80 58V64C80 73 75 78 66 78"
        fill="none"
        stroke="#76CE54"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="9"
      />
      <path
        d="M36 39L42 67L50 48L58 67L64 39"
        fill="none"
        stroke="#FFFFFF"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="8"
      />
    </svg>
  );
}
