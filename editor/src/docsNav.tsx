import React from 'react';
import { Copy, Check } from 'lucide-react';

export interface NavGroup {
  category: string;
  items: { id: string; label: string }[];
}

export const NAV_ITEMS: NavGroup[] = [
  {
    category: 'Getting Started',
    items: [
      { id: 'intro', label: 'Introduction & Vision' },
      { id: 'quickstart', label: 'Quickstart Guide' },
      { id: 'document-structure', label: 'Document Hierarchy' },
    ],
  },
  {
    category: 'Grammar & Tags',
    items: [
      { id: 'props-block', label: '<props> Configuration' },
      { id: 'paper-geometry', label: '<paper> & Page Sizing' },
      { id: 'page-styles', label: '<page-styles> & Running Headers' },
      { id: 'sections-blocks', label: '<section> & Core Elements' },
      { id: 'css-grid', label: '<grid> Multi-Column Flow' },
      { id: 'callouts-tables', label: '<callout> & <table>' },
    ],
  },
  {
    category: 'Modular Files & CSS',
    items: [
      { id: 'external-css', label: 'External CSS / DCSS & <link>' },
      { id: 'css-properties', label: 'Supported CSS Properties' },
    ],
  },
  {
    category: 'Rust Engine & CLI',
    items: [
      { id: 'engine-ast', label: 'AST & Parsing Core' },
      { id: 'backend-api', label: 'Rust HTTP Backend & CLI' },
    ],
  },
  {
    category: 'Export & Conversion',
    items: [
      { id: 'export-formats', label: 'PDF & OpenXML (DOCX) Export' },
      { id: 'reverse-engineering', label: 'Reverse Conversion (.dxml & .dcss)' },
    ],
  },
];

export const CodeSnippet: React.FC<{
  id: string;
  code: string;
  copiedId: string | null;
  onCopy: (text: string, id: string) => void;
}> = ({ id, code, copiedId, onCopy }) => {
  const isCopied = copiedId === id;
  return (
    <div style={{ position: 'relative', marginTop: '12px', marginBottom: '24px' }}>
      <button
        onClick={() => onCopy(code, id)}
        style={{
          position: 'absolute',
          top: '12px',
          right: '12px',
          background: '#1e293b',
          color: isCopied ? '#4ade80' : '#94a3b8',
          border: '1px solid #334155',
          borderRadius: '4px',
          padding: '4px 8px',
          fontSize: '11px',
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          gap: '4px',
        }}
      >
        {isCopied ? <Check size={12} /> : <Copy size={12} />}
        {isCopied ? 'Copied' : 'Copy'}
      </button>
      <pre
        style={{
          background: '#020617',
          border: '1px solid #1e293b',
          borderRadius: '8px',
          padding: '16px 20px',
          overflowX: 'auto',
          fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
          fontSize: '13px',
          color: '#e2e8f0',
          lineHeight: 1.6,
        }}
      >
        <code>{code}</code>
      </pre>
    </div>
  );
};
