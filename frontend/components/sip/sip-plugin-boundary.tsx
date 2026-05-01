'use client';
import { ReactNode, Component } from 'react';
import { AlertTriangle } from 'lucide-react';

interface SipPluginBoundaryProps {
  pluginId: string;
  children: ReactNode;
  fallback?: ReactNode;
}

interface SipPluginBoundaryState {
  hasError: boolean;
  error?: Error;
}

export class SipPluginBoundary extends Component<SipPluginBoundaryProps, SipPluginBoundaryState> {
  constructor(props: SipPluginBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;
      if (process.env.NODE_ENV === 'development') {
        return (
          <div className="p-4 rounded-lg" style={{ backgroundColor: 'var(--sip-color-danger-light)', border: '1px solid var(--sip-color-danger)' }}>
            <div className="flex items-center space-x-2">
              <AlertTriangle className="w-5 h-5" style={{ color: 'var(--sip-color-danger)' }} />
              <span className="text-sm font-medium" style={{ color: 'var(--sip-color-danger)' }}>
                Plugin error: {this.props.pluginId}
              </span>
            </div>
            <pre className="mt-2 text-xs" style={{ color: 'var(--sip-color-text-muted)' }}>
              {this.state.error?.message}
            </pre>
          </div>
        );
      }
      return null;
    }
    return (
      <div data-sip-plugin={this.props.pluginId}>
        {this.props.children}
      </div>
    );
  }
}
