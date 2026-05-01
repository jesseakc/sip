import { ReactNode, ElementType } from 'react';

interface SipCardProps {
  children: ReactNode;
  className?: string;
  padding?: 'none' | 'sm' | 'md' | 'lg';
  as?: ElementType;
  href?: string;
  onClick?: () => void;
}

export function SipCard({ children, className = '', padding = 'md', as, href, onClick }: SipCardProps) {
  const paddings = { none: '', sm: 'p-3', md: 'p-4', lg: 'p-6' };
  const Comp = as || (href ? 'a' : 'div');
  return (
    <Comp
      href={href}
      onClick={onClick}
      className={`block sip-surface ${paddings[padding]} ${href ? 'hover:shadow-md transition-shadow cursor-pointer' : ''} ${className}`}
    >
      {children}
    </Comp>
  );
}
