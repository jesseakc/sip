import { ReactNode } from 'react';
import { AlertTriangle, CheckCircle, Info, XCircle } from 'lucide-react';

interface SipAlertProps {
  variant: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  children: ReactNode;
  className?: string;
}

const config = {
  info: { bg: 'var(--sip-color-info-light)', border: 'var(--sip-color-info)', text: '#0e7490', Icon: Info },
  success: { bg: 'var(--sip-color-success-light)', border: 'var(--sip-color-success)', text: '#15803d', Icon: CheckCircle },
  warning: { bg: 'var(--sip-color-warning-light)', border: 'var(--sip-color-warning)', text: '#92400e', Icon: AlertTriangle },
  error: { bg: 'var(--sip-color-danger-light)', border: 'var(--sip-color-danger)', text: '#991b1b', Icon: XCircle },
};

export function SipAlert({ variant, title, children, className = '' }: SipAlertProps) {
  const { bg, border, text, Icon } = config[variant];
  return (
    <div className={`rounded-lg border p-4 ${className}`} style={{ backgroundColor: bg, borderColor: border }}>
      <div className="flex items-start space-x-3">
        <Icon className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: text }} />
        <div>
          {title && <p className="text-sm font-medium" style={{ color: text }}>{title}</p>}
          <div className="text-sm" style={{ color: text }}>{children}</div>
        </div>
      </div>
    </div>
  );
}
