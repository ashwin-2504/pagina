import React from 'react';
import type { DocNode, PageStyle } from './types';
import { hasImageNode } from './renderUtils';
import { PageContext, RenderChildren } from './RenderElement';

interface PageSheetProps {
  pageNum: number;
  totalPages: number;
  nodes: DocNode[];
  pageStyleName: string;
  pageStyle: PageStyle | undefined;
  widthMm: number;
  heightMm: number;
  marginMm: number;
  headerDistanceMm: number;
  footerDistanceMm: number;
  paperSize: string;
  orientation: string;
  paperLine: number;
}

export const PageSheet: React.FC<PageSheetProps> = ({
  pageNum, totalPages, nodes, pageStyle,
  widthMm, heightMm, marginMm,
  headerDistanceMm, footerDistanceMm,
  paperSize, orientation, paperLine,
}) => {
  const headerHasImage = Boolean(pageStyle?.running_header && (
    hasImageNode(pageStyle.running_header.center) ||
    hasImageNode(pageStyle.running_header.left) ||
    hasImageNode(pageStyle.running_header.right)
  ));
  const footerHasImage = Boolean(pageStyle?.running_footer && (
    hasImageNode(pageStyle.running_footer.center) ||
    hasImageNode(pageStyle.running_footer.left) ||
    hasImageNode(pageStyle.running_footer.right)
  ));
  const headerHeightMm = pageStyle?.running_header ? (headerHasImage ? 38 : 14) : 0;
  const footerHeightMm = pageStyle?.running_footer ? (footerHasImage ? 28 : 14) : 0;
  const headerGapMm = headerHeightMm > 0 ? (headerHasImage ? 3 : 8) : 0;
  const footerGapMm = footerHeightMm > 0 ? 3 : 0;
  const contentTopMm = Math.max(marginMm, headerDistanceMm + headerHeightMm + headerGapMm);
  const contentBottomMm = Math.max(marginMm, footerDistanceMm + footerHeightMm + footerGapMm);

  return (
    <div className="page-wrapper" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
      {/* Page Indicator Header */}
      <div
        className="page-header-indicator"
        data-ddl-line={paperLine}
        title="Double-click to go to paper setup in editor"
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          width: `${widthMm}mm`,
          color: '#94a3b8',
          fontSize: '11px',
          fontWeight: 600,
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
          marginBottom: '10px',
          padding: '0 4px',
          userSelect: 'none',
          cursor: 'pointer',
        }}
      >
        <span>Page {pageNum} of {totalPages}</span>
        <span>{paperSize} &bull; {orientation}</span>
      </div>

      {/* Physical Paper Sheet */}
      <div
        className="ddl-paper"
        data-page-number={pageNum}
        data-ddl-line={nodes[0]?.span?.line || 1}
        style={{
          width: `${widthMm}mm`,
          height: `${heightMm}mm`,
          minHeight: `${heightMm}mm`,
          maxHeight: `${heightMm}mm`,
          backgroundColor: '#ffffff',
          boxShadow: '0 8px 30px rgba(0, 0, 0, 0.28)',
          padding: 0,
          boxSizing: 'border-box',
          position: 'relative',
          overflow: 'hidden',
          borderRadius: '2px',
          color: '#1e293b',
          fontFamily: 'inherit',
          display: 'flex',
          flexDirection: 'column',
          textAlign: 'left',
          lineHeight: 1.5,
        }}
      >
        <PageContext.Provider value={{ pageNum, totalPages }}>
          {/* Watermark */}
          {pageStyle?.watermark && (
            <div
              data-ddl-line={pageStyle.watermark.line || 4}
              title="Double-click to go to watermark in editor"
              style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: `translate(-50%, -50%) rotate(${pageStyle.watermark.angle_deg || -45}deg)`,
                fontSize: '56pt',
                fontWeight: 900,
                color: `rgba(0,0,0,${pageStyle.watermark.opacity || 0.08})`,
                userSelect: 'none',
                pointerEvents: 'none',
                zIndex: 0,
                whiteSpace: 'nowrap',
              }}
            >
              {pageStyle.watermark.text}
            </div>
          )}

          {/* Running Header */}
          {pageStyle?.running_header && <RunningHeader
            header={pageStyle.running_header}
            hasImage={headerHasImage}
            marginMm={marginMm}
            headerDistanceMm={headerDistanceMm}
          />}

          {/* Page Content Body */}
          <div style={{
            position: 'absolute',
            top: `${contentTopMm}mm`,
            right: `${marginMm}mm`,
            bottom: `${contentBottomMm}mm`,
            left: `${marginMm}mm`,
            zIndex: 1,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
          }}>
            <RenderChildren nodes={nodes} />
          </div>

          {/* Running Footer */}
          {pageStyle?.running_footer && <RunningFooter
            footer={pageStyle.running_footer}
            marginMm={marginMm}
            footerDistanceMm={footerDistanceMm}
          />}
        </PageContext.Provider>
      </div>
    </div>
  );
};

// --- Sub-components for header and footer ---

interface RunningHeaderProps {
  header: NonNullable<PageStyle['running_header']>;
  hasImage: boolean;
  marginMm: number;
  headerDistanceMm: number;
}

const RunningHeader: React.FC<RunningHeaderProps> = ({ header, hasImage, marginMm, headerDistanceMm }) => {
  const hasLeft = header.left && header.left.length > 0;
  const hasCenter = header.center && header.center.length > 0;
  const hasRight = header.right && header.right.length > 0;

  if (hasCenter && !hasLeft && !hasRight) {
    return (
      <header className="running-header header-center-only" data-ddl-line={header.line} style={{
        position: 'absolute',
        top: `${headerDistanceMm}mm`,
        left: `${marginMm}mm`,
        right: `${marginMm}mm`,
        zIndex: 2,
        width: 'auto',
        textAlign: 'center',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        borderBottom: hasImage ? 'none' : '1px solid #e2e8f0',
        paddingBottom: hasImage ? '0' : '3mm',
        marginBottom: 0,
        fontSize: '9pt',
        userSelect: 'none',
      }}>
        <div className="header-center" style={{ width: '100%', display: 'flex', justifyContent: 'center' }}>
          <RenderChildren nodes={header.center} />
        </div>
      </header>
    );
  }

  return (
    <header className="running-header" data-ddl-line={header.line} style={{
      position: 'absolute',
      top: `${headerDistanceMm}mm`,
      left: `${marginMm}mm`,
      right: `${marginMm}mm`,
      zIndex: 2,
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      borderBottom: hasImage ? 'none' : '1px solid #e2e8f0',
      paddingBottom: hasImage ? '0' : '3mm',
      marginBottom: 0,
      fontSize: '9pt',
      color: '#64748b',
      userSelect: 'none',
    }}>
      <div className="header-left" style={{ flex: hasLeft ? 1 : 0, textAlign: 'left' }}><RenderChildren nodes={header.left} /></div>
      <div className="header-center" style={{ flex: hasCenter ? 2 : 0, textAlign: 'center' }}><RenderChildren nodes={header.center} /></div>
      <div className="header-right" style={{ flex: hasRight ? 1 : 0, textAlign: 'right' }}><RenderChildren nodes={header.right} /></div>
    </header>
  );
};

interface RunningFooterProps {
  footer: NonNullable<PageStyle['running_footer']>;
  marginMm: number;
  footerDistanceMm: number;
}

const RunningFooter: React.FC<RunningFooterProps> = ({ footer, marginMm, footerDistanceMm }) => {
  const hasImg = hasImageNode(footer.center) ||
                 hasImageNode(footer.left) ||
                 hasImageNode(footer.right);
  return (
    <footer className="running-footer" data-ddl-line={footer.line} style={{
      position: 'absolute',
      right: `${marginMm}mm`,
      bottom: `${footerDistanceMm}mm`,
      left: `${marginMm}mm`,
      zIndex: 2,
      marginTop: 0,
      borderTop: hasImg ? 'none' : '1px solid #e2e8f0',
      paddingTop: hasImg ? '0' : '3mm',
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      fontSize: '9pt',
      color: '#94a3b8',
      userSelect: 'none',
    }}>
      <div className="footer-left" style={{ flex: 1, textAlign: 'left' }}><RenderChildren nodes={footer.left} /></div>
      <div className="footer-center" style={{ flex: 1, textAlign: 'center' }}><RenderChildren nodes={footer.center} /></div>
      <div className="footer-right" style={{ flex: 1, textAlign: 'right' }}><RenderChildren nodes={footer.right} /></div>
    </footer>
  );
};
