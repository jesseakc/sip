'use client';
import { ReactNode } from 'react';

interface SipPageProps {
  children: ReactNode;
  maxWidth?: string;
  className?: string;
}

export function SipPage({ children, maxWidth, className = '' }: SipPageProps) {
  return (
    <div
      className={`p-4 lg:p-8 overflow-auto ${className}`}
      style={{ maxWidth: maxWidth || 'var(--sip-layout-content-width)', margin: '0 auto' }}
    >
      {children}
    </div>
  );
}

interface SipPageHeaderProps {
  title: string;
  description?: string;
  actions?: ReactNode;
  breadcrumbs?: ReactNode;
}

export function SipPageHeader({ title, description, actions, breadcrumbs }: SipPageHeaderProps) {
  return (
    <div className="mb-6">
      {breadcrumbs && <div className="mb-2">{breadcrumbs}</div>}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold" style={{ color: 'var(--sip-color-text)' }}>{title}</h1>
          {description && (
            <p className="mt-1 text-sm" style={{ color: 'var(--sip-color-text-muted)' }}>{description}</p>
          )}
        </div>
        {actions && <div className="flex items-center space-x-3 flex-shrink-0 ml-4">{actions}</div>}
      </div>
    </div>
  );
}
