'use client';
import { ReactNode } from 'react';
import { useAuth } from '@/lib/auth-context';

interface SipPermissionGateProps {
  permission?: string;
  fallback?: ReactNode;
  children: ReactNode;
}

export function SipPermissionGate({ permission, fallback, children }: SipPermissionGateProps) {
  const { user } = useAuth();

  if (!permission) return <>{children}</>;

  const allowedRoles = ['admin', 'manager'];
  if (user && allowedRoles.includes(user.role?.toLowerCase())) {
    return <>{children}</>;
  }

  if (fallback !== undefined) return <>{fallback}</>;
  return null;
}
