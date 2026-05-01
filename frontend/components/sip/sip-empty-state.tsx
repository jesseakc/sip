import { ReactNode } from 'react';
import { FileText } from 'lucide-react';

interface SipEmptyStateProps {
  icon?: ReactNode;
  title: string;
  description?: string;
  action?: ReactNode;
}

export function SipEmptyState({ icon, title, description, action }: SipEmptyStateProps) {
  return (
    <div className="text-center py-12 rounded-lg" style={{ backgroundColor: 'var(--sip-color-surface)', border: '1px solid var(--sip-color-border)' }}>
      <div className="mb-3" style={{ color: 'var(--sip-color-text-muted)' }}>
        {icon || <FileText className="w-12 h-12 mx-auto" />}
      </div>
      <h3 className="text-lg font-medium" style={{ color: 'var(--sip-color-text)' }}>{title}</h3>
      {description && (
        <p className="mt-1 text-sm" style={{ color: 'var(--sip-color-text-muted)' }}>{description}</p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
