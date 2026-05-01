import { ReactNode, ButtonHTMLAttributes } from 'react';

interface SipButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  icon?: ReactNode;
  children: ReactNode;
}

const sizes = { sm: 'px-2.5 py-1.5 text-xs', md: 'px-3 py-2 text-sm', lg: 'px-4 py-2.5 text-base' };

export function SipButton({ variant = 'primary', size = 'md', loading, icon, children, className = '', disabled, ...props }: SipButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center space-x-2 rounded-md font-medium transition-colors
        focus:outline-none focus:ring-2 focus:ring-offset-1 disabled:opacity-50 disabled:cursor-not-allowed
        ${sizes[size]}
        ${variant === 'primary' ? 'text-white' : ''}
        ${variant === 'danger' ? 'text-white' : ''}
        ${className}`}
      style={{
        backgroundColor: variant === 'primary' ? 'var(--sip-color-primary)' :
                         variant === 'danger' ? 'var(--sip-color-danger)' :
                         variant === 'secondary' ? 'var(--sip-color-surface)' : 'transparent',
        borderColor: variant === 'secondary' ? 'var(--sip-color-border)' : 'transparent',
        borderWidth: variant === 'secondary' ? '1px' : '0',
        color: variant === 'ghost' || variant === 'secondary' ? 'var(--sip-color-text)' : undefined,
      }}
      disabled={disabled || loading}
      {...props}
    >
      {loading && (
        <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
      )}
      {!loading && icon}
      <span>{children}</span>
    </button>
  );
}
