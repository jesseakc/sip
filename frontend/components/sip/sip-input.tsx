import { InputHTMLAttributes, SelectHTMLAttributes } from 'react';

interface SipInputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
}

export function SipInput({ label, error, hint, className = '', ...props }: SipInputProps) {
  return (
    <div className="w-full">
      {label && <label className="block text-sm font-medium mb-1" style={{ color: 'var(--sip-color-text)' }}>{label}</label>}
      <input
        className={`w-full px-3 py-2 rounded-md text-sm transition-colors focus:outline-none focus:ring-2 ${className}`}
        style={{
          backgroundColor: 'var(--sip-color-surface)',
          borderColor: error ? 'var(--sip-color-danger)' : 'var(--sip-color-border)',
          borderWidth: '1px',
          color: 'var(--sip-color-text)',
        }}
        {...props}
      />
      {error && <p className="mt-1 text-xs" style={{ color: 'var(--sip-color-danger)' }}>{error}</p>}
      {hint && !error && <p className="mt-1 text-xs" style={{ color: 'var(--sip-color-text-muted)' }}>{hint}</p>}
    </div>
  );
}

interface SipSelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  options: Array<{ value: string; label: string }>;
  error?: string;
}

export function SipSelect({ label, options, error, className = '', ...props }: SipSelectProps) {
  return (
    <div className="w-full">
      {label && <label className="block text-sm font-medium mb-1" style={{ color: 'var(--sip-color-text)' }}>{label}</label>}
      <select
        className={`w-full px-3 py-2 rounded-md text-sm transition-colors focus:outline-none focus:ring-2 ${className}`}
        style={{
          backgroundColor: 'var(--sip-color-surface)',
          borderColor: error ? 'var(--sip-color-danger)' : 'var(--sip-color-border)',
          borderWidth: '1px',
          color: 'var(--sip-color-text)',
        }}
        {...props}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
      {error && <p className="mt-1 text-xs" style={{ color: 'var(--sip-color-danger)' }}>{error}</p>}
    </div>
  );
}
