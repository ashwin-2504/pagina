import React, { useRef } from 'react';
import type { ParsedDocument, PageStyle } from './types';
import { AlertCircle } from 'lucide-react';
import { findLineByText } from './renderUtils';
import { AssetContext, RenderChildren, PageContext } from './RenderElement';
import { RenderNode } from './RenderContext';
import { PageSheet } from './PageSheet';
import { useDocumentCss, usePagination } from './PageLayout';

export { RenderChildren, RenderNode, PageContext };

interface ErrorInfo {
  message: string;
  line?: number;
  column?: number;
  snippet?: string;
}

interface DocumentRendererProps {
  doc: ParsedDocument | null;
  error: ErrorInfo | null;
  assets?: Record<string, string>;
  onNavigateToLine?: (line: number) => void;
}

export const DocumentRenderer: React.FC<DocumentRendererProps> = ({
  doc,
  error,
  assets = {},
  onNavigateToLine,
}) => {
  const measureContainerRef = useRef<HTMLDivElement>(null);

  const handleDoubleClick = (e: React.MouseEvent) => {
    if (!onNavigateToLine) return;
    const target = e.target as HTMLElement | null;
    if (!target) return;

    // 1. Closest element with data-ddl-line
    const lineEl = target.closest('[data-ddl-line]');
    if (lineEl) {
      const lineStr = lineEl.getAttribute('data-ddl-line');
      const line = lineStr ? parseInt(lineStr, 10) : 0;
      if (line > 0) {
        window.getSelection()?.removeAllRanges();
        onNavigateToLine(line);
        return;
      }
    }

    // 2. Text search fallback
    const sel = window.getSelection()?.toString().trim();
    const rawText = sel || target.innerText?.trim() || target.textContent?.trim();
    if (rawText && doc) {
      const matched = findLineByText(doc, rawText);
      if (matched && matched > 0) {
        window.getSelection()?.removeAllRanges();
        onNavigateToLine(matched);
      }
    }
  };

  if (error) {
    return (
      <div style={{
        padding: '24px',
        margin: '20px auto',
        background: '#fff1f2',
        border: '1px solid #fecdd3',
        borderRadius: '8px',
        color: '#9f1239',
        fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
        fontSize: '13px',
        lineHeight: 1.6,
        maxWidth: '850px',
        width: '100%',
        boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
        textAlign: 'left',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
          <AlertCircle size={20} color="#e11d48" />
          <h4 style={{ margin: 0, fontSize: '15px', fontWeight: 700, color: '#e11d48' }}>
            DDL Syntax Diagnostic {error.line !== undefined ? `(Line ${error.line}, Col ${error.column || 1})` : ''}
          </h4>
        </div>
        <div style={{ background: '#ffe4e6', padding: '10px 14px', borderRadius: '6px', marginBottom: '12px', fontWeight: 600 }}>
          {error.message}
        </div>
        {error.snippet && (
          <div>
            <div style={{ fontSize: '11px', color: '#be123c', textTransform: 'uppercase', fontWeight: 700, marginBottom: '6px' }}>
              Error Location Snippet:
            </div>
            <pre style={{
              margin: 0,
              background: '#881337',
              color: '#ffe4e6',
              padding: '12px',
              borderRadius: '6px',
              overflowX: 'auto',
              fontSize: '12px',
            }}>
              {error.snippet}
            </pre>
          </div>
        )}
      </div>
    );
  }

  if (!doc) {
    return (
      <div style={{ color: '#94a3b8', padding: '40px', textAlign: 'center' }}>
        No document AST loaded.
      </div>
    );
  }

  const customCss = useDocumentCss(doc);
  const paperSize = doc.props?.paper?.size || 'A4';
  const orientation = doc.props?.paper?.orientation || 'portrait';
  const marginMm = doc.props?.paper?.margin?.val ?? 20;
  const defaultRunningDistanceMm = Math.min(marginMm / 2, 12.7);
  const headerDistanceMm = doc.props?.paper?.header_distance?.val ?? defaultRunningDistanceMm;
  const footerDistanceMm = doc.props?.paper?.footer_distance?.val ?? defaultRunningDistanceMm;

  const isLandscape = orientation === 'landscape';
  const isLetter = paperSize.toUpperCase() === 'LETTER';
  const widthMm = isLetter ? (isLandscape ? 279.4 : 215.9) : (isLandscape ? 297 : 210);
  const heightMm = isLetter ? (isLandscape ? 215.9 : 279.4) : (isLandscape ? 210 : 297);

  const { pages, sectionsWithSplitBlocks } = usePagination({
    doc,
    heightMm,
    widthMm,
    marginMm,
    headerDistanceMm,
    footerDistanceMm,
    customCss,
    measureContainerRef,
  });

  return (
    <AssetContext.Provider value={assets}>
      {customCss && <style>{customCss}</style>}
      <style>{`
        @media print {
          body {
            background: transparent !important;
            margin: 0 !important;
            padding: 0 !important;
          }
          .page-header-indicator {
            display: none !important;
          }
          .page-wrapper {
            margin: 0 !important;
            padding: 0 !important;
            page-break-after: always !important;
            break-after: page !important;
          }
          .ddl-paper {
            box-shadow: none !important;
            margin: 0 !important;
            page-break-after: always !important;
            break-after: page !important;
            page-break-inside: avoid !important;
            break-inside: avoid !important;
          }
        }
      `}</style>
      <div
        onDoubleClick={handleDoubleClick}
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: '40px',
          alignItems: 'center',
          paddingBottom: '60px',
          userSelect: 'text',
        }}
      >
        {pages.map((page, pageIdx) => {
          const pageStyle: PageStyle | undefined = doc.props.page_styles[page.pageStyleName];
          return (
            <PageSheet
              key={pageIdx}
              pageNum={page.pageNum}
              totalPages={pages.length}
              nodes={page.nodes}
              pageStyleName={page.pageStyleName}
              pageStyle={pageStyle}
              widthMm={widthMm}
              heightMm={heightMm}
              marginMm={marginMm}
              headerDistanceMm={headerDistanceMm}
              footerDistanceMm={footerDistanceMm}
              paperSize={paperSize}
              orientation={orientation}
              paperLine={doc.props.paper_line || 3}
            />
          );
        })}
      </div>

      {/* Invisible Measurement Container to compute exact block heights */}
      <div
        ref={measureContainerRef}
        style={{
          position: 'absolute',
          top: -99999,
          left: -99999,
          width: `${widthMm - 2 * marginMm}mm`,
          visibility: 'hidden',
          pointerEvents: 'none',
          boxSizing: 'border-box',
          fontFamily: 'inherit',
          lineHeight: 1.5,
        }}
        aria-hidden="true"
      >
        {sectionsWithSplitBlocks.flatMap((sec, sIdx) =>
          sec.blocks.map((block, bIdx) => (
            <div
              key={`${sIdx}-${bIdx}`}
              data-sec-idx={sIdx}
              data-block-idx={bIdx}
              style={{ boxSizing: 'border-box' }}
            >
              <RenderNode node={block} />
            </div>
          ))
        )}
      </div>
    </AssetContext.Provider>
  );
};
