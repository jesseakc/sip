'use client';
import { ReactNode } from 'react';

interface ExtensionSlotProps {
  name: string;
  fallback?: ReactNode;
  context?: Record<string, unknown>;
}

export function SipExtensionSlot({ fallback }: ExtensionSlotProps) {
  return <>{fallback || null}</>;
}
