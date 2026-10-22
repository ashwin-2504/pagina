import React from 'react';
import { AlertCircle, RefreshCw } from 'lucide-react';

interface ErrorBoundaryProps {
  children: React.ReactNode;
  onRetry?: () => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

/**
 * Catches render-time exceptions in the preview subtree and shows
 * a recovery UI instead of crashing the entire application.
 */
export class PreviewErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('[PreviewErrorBoundary] Render crash caught:', error, info.componentStack);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
    this.props.onRetry?.();
  };

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          padding: '32px',
          margin: '40px auto',
          maxWidth: '600px',
          background: '#1c1917',
          border: '1px solid #44403c',
          borderRadius: '12px',
          color: '#fafaf9',
          textAlign: 'center',
        }}>
          <AlertCircle size={36} color="#ef4444" style={{ marginBottom: '12px' }} />
          <h3 style={{ margin: '0 0 8px', fontSize: '16px', fontWeight: 700, color: '#fca5a5' }}>
            Preview Render Error
          </h3>
          <p style={{
            fontSize: '13px',
            color: '#a8a29e',
            lineHeight: 1.5,
            margin: '0 0 16px',
          }}>
            The preview crashed due to a rendering error. Your code is safe — fix the
            issue in the editor and click Retry.
          </p>
          <pre style={{
            background: '#292524',
            color: '#fbbf24',
            padding: '10px 14px',
            borderRadius: '6px',
            fontSize: '11px',
            textAlign: 'left',
            overflowX: 'auto',
            marginBottom: '16px',
            maxHeight: '120px',
            overflow: 'auto',
          }}>
            {this.state.error?.message || 'Unknown error'}
          </pre>
          <button
            onClick={this.handleRetry}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '6px',
              background: '#0284c7',
              color: '#fff',
              border: 'none',
              padding: '8px 20px',
              borderRadius: '6px',
              fontSize: '13px',
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            <RefreshCw size={14} /> Retry Render
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
