import { ReactNode } from 'react';

type BadgeVariant = 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info';

interface SipBadgeProps {
  children: ReactNode;
  variant?: BadgeVariant;
  className?: string;
}

const variantStyles: Record<BadgeVariant, { bg: string; text: string }> = {
  default: { bg: 'var(--sip-color-surface-muted)', text: 'var(--sip-color-text-muted)' },
  primary: { bg: 'var(--sip-color-primary-light)', text: 'var(--sip-color-primary-text)' },
  success: { bg: 'var(--sip-color-success-light)', text: 'var(--sip-color-success)' },
  warning: { bg: 'var(--sip-color-warning-light)', text: 'var(--sip-color-warning)' },
  danger: { bg: 'var(--sip-color-danger-light)', text: 'var(--sip-color-danger)' },
  info: { bg: 'var(--sip-color-info-light)', text: 'var(--sip-color-info)' },
};

export function SipBadge({ children, variant = 'default', className = '' }: SipBadgeProps) {
  const s = variantStyles[variant];
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${className}`}
      style={{ backgroundColor: s.bg, color: s.text }}
    >
      {children}
    </span>
  );
}

interface SipStatusBadgeProps {
  status: string;
  className?: string;
}

const STATUS_MAP: Record<string, BadgeVariant> = {
  operational: 'success', active: 'success', completed: 'success',
  open: 'primary', in_progress: 'primary', importing: 'warning',
  degraded: 'warning', on_hold: 'warning', maintenance: 'warning',
  down: 'danger', failed: 'danger', cancelled: 'danger',
  draft: 'default', closed: 'default', archived: 'default',
};

export function SipStatusBadge({ status, className = '' }: SipStatusBadgeProps) {
  const variant = STATUS_MAP[status.replace(/ /g, '_').toLowerCase()] || 'default';
  return (
    <SipBadge variant={variant} className={className}>
      {status.replace(/_/g, ' ')}
    </SipBadge>
  );
}
