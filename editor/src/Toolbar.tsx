import React from 'react';
import {
  FileText,
  RefreshCw,
  CheckCircle2,
  AlertTriangle,
} from 'lucide-react';
import type { EngineStatus } from './engineBridge';
import { ToolbarActions } from './ToolbarActions';

interface ToolbarProps {
  engineStatus: EngineStatus;
  isCompiling: boolean;
  viewMode: string;
  setViewMode: React.Dispatch<React.SetStateAction<string>>;
  onFormat: () => void;
  onOpenImagePicker: () => void;
  onOpenImportPicker: () => void;
  isImporting: boolean;
  isExporting: boolean;
  onExport: (format: 'pdf' | 'docx') => void;
  zoom: number;
  setZoom: React.Dispatch<React.SetStateAction<number>>;
  onRender: () => void;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  engineStatus,
  isCompiling,
  viewMode,
  setViewMode,
  onFormat,
  onOpenImagePicker,
  onOpenImportPicker,
  isImporting,
  isExporting,
  onExport,
  zoom,
  setZoom,
  onRender,
}) => {
  return (
    <header
      className="no-print"
      style={{
        height: '48px',
        borderBottom: '1px solid #1e293b',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 16px',
        background: '#0f172a',
        color: '#f8fafc',
        flexShrink: 0,
        zIndex: 10,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <FileText size={18} color="#38bdf8" />
        <span style={{ fontWeight: 700, fontSize: '15px', color: '#f8fafc' }}>
          Pagina Studio
        </span>
        <span
          style={{
            fontSize: '11px',
            background: '#1e293b',
            color: '#38bdf8',
            padding: '2px 8px',
            borderRadius: '6px',
            fontWeight: 600,
          }}
        >
          Modular DDL
        </span>

        {/* Rust Engine Status Badge */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '5px',
            fontSize: '11px',
            padding: '2px 8px',
            borderRadius: '6px',
            background: engineStatus.online
              ? 'rgba(34, 197, 94, 0.15)'
              : 'rgba(239, 68, 68, 0.15)',
            color: engineStatus.online ? '#4ade80' : '#f87171',
            border: engineStatus.online
              ? '1px solid rgba(34, 197, 94, 0.3)'
              : '1px solid rgba(239, 68, 68, 0.3)',
            marginLeft: '8px',
          }}
        >
          {engineStatus.online ? (
            <CheckCircle2 size={12} />
          ) : (
            <AlertTriangle size={12} />
          )}
          <span>{engineStatus.label}</span>
        </div>

        {isCompiling && (
          <span
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
              fontSize: '11px',
              color: '#38bdf8',
              marginLeft: '6px',
            }}
          >
            <RefreshCw size={11} className="animate-spin" /> Compiling...
          </span>
        )}
      </div>

      <ToolbarActions
        viewMode={viewMode}
        setViewMode={setViewMode}
        onFormat={onFormat}
        onOpenImagePicker={onOpenImagePicker}
        onOpenImportPicker={onOpenImportPicker}
        isImporting={isImporting}
        isExporting={isExporting}
        onExport={onExport}
        zoom={zoom}
        setZoom={setZoom}
        onRender={onRender}
      />
    </header>
  );
};
