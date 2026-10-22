import React from 'react';
import {
  Play,
  ZoomIn,
  ZoomOut,
  Wand2,
  BookOpen,
  Download,
  Upload,
  Printer,
  Image as ImageIcon,
} from 'lucide-react';

interface ToolbarActionsProps {
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

export const ToolbarActions: React.FC<ToolbarActionsProps> = ({
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
    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
      <button
        onClick={() => setViewMode(v => (v === 'docs' ? 'editor' : 'docs'))}
        title="Open Documentation"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: viewMode === 'docs' ? '#38bdf8' : '#1e293b',
          color: viewMode === 'docs' ? '#0f172a' : '#38bdf8',
          border: '1px solid #334155',
          padding: '6px 12px',
          borderRadius: '6px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: 'pointer',
          transition: 'all 0.15s ease',
        }}
      >
        <BookOpen size={13} /> {viewMode === 'docs' ? 'Close Docs' : 'Docs'}
      </button>

      <button
        onClick={onFormat}
        title="Format current file"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#1e293b',
          color: '#38bdf8',
          border: '1px solid #334155',
          padding: '6px 12px',
          borderRadius: '6px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: 'pointer',
        }}
      >
        <Wand2 size={13} /> Format
      </button>

      <button
        onClick={onOpenImagePicker}
        title="Insert Image (Embeds Base64 PNG/JPG/SVG directly)"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#1e293b',
          color: '#38bdf8',
          border: '1px solid #0284c7',
          padding: '6px 12px',
          borderRadius: '6px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: 'pointer',
        }}
      >
        <ImageIcon size={13} /> Insert Image
      </button>

      <button
        onClick={onOpenImportPicker}
        disabled={isImporting}
        title="Reverse-engineer DOCX or PDF into .dxml and .dcss"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#1e293b',
          color: '#a78bfa',
          border: '1px solid #4c1d95',
          padding: '6px 12px',
          borderRadius: '6px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: isImporting ? 'not-allowed' : 'pointer',
        }}
      >
        <Upload size={13} /> {isImporting ? 'Importing...' : 'Import DOCX/PDF'}
      </button>

      <div style={{ display: 'flex', alignItems: 'center' }}>
        <button
          onClick={() => window.print()}
          title="Export High-Fidelity Vector PDF (Matches live preview 100%)"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            background: '#065f46',
            color: '#34d399',
            border: '1px solid #059669',
            padding: '6px 12px',
            borderRadius: '6px 0 0 6px',
            fontSize: '12px',
            fontWeight: 600,
            cursor: 'pointer',
            transition: 'background 0.15s ease',
          }}
        >
          <Printer size={13} /> Vector PDF
        </button>
        <button
          onClick={() => onExport('pdf')}
          disabled={isExporting}
          title="Download PDF compiled directly by Rust ddl-engine"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
            background: '#1e293b',
            color: '#34d399',
            border: '1px solid #065f46',
            borderLeft: 'none',
            padding: '6px 8px',
            borderRadius: '0 6px 6px 0',
            fontSize: '11px',
            fontWeight: 600,
            cursor: isExporting ? 'not-allowed' : 'pointer',
          }}
        >
          <Download size={12} /> Engine
        </button>
      </div>

      <button
        onClick={() => onExport('docx')}
        disabled={isExporting}
        title="Export via Rust Engine to OpenXML (.docx)"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#1e293b',
          color: '#60a5fa',
          border: '1px solid #1e40af',
          padding: '6px 12px',
          borderRadius: '6px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: isExporting ? 'not-allowed' : 'pointer',
        }}
      >
        <Download size={13} /> DOCX
      </button>

      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#1e293b',
          padding: '2px 8px',
          borderRadius: '6px',
        }}
      >
        <button
          onClick={() => setZoom(z => Math.max(0.4, Number((z - 0.1).toFixed(2))))}
          title="Zoom Out"
          style={{ background: 'none', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '2px' }}
        >
          <ZoomOut size={14} />
        </button>
        <span style={{ fontSize: '11px', minWidth: '35px', textAlign: 'center', color: '#cbd5e1' }}>
          {Math.round(zoom * 100)}%
        </span>
        <button
          onClick={() => setZoom(z => Math.min(1.5, Number((z + 0.1).toFixed(2))))}
          title="Zoom In"
          style={{ background: 'none', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '2px' }}
        >
          <ZoomIn size={14} />
        </button>
      </div>

      <button
        onClick={onRender}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          background: '#0284c7',
          color: '#fff',
          border: 'none',
          padding: '6px 14px',
          borderRadius: '6px',
          fontSize: '12px',
          fontWeight: 600,
          cursor: 'pointer',
        }}
      >
        <Play size={13} fill="#fff" /> Render
      </button>
    </div>
  );
};
