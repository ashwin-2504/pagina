import React, { useMemo, useState, useLayoutEffect, useRef } from 'react';
import type { ParsedDocument, DocNode, ElementNode, PageStyle } from './types';
import { AlertCircle } from 'lucide-react';

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
}

const AssetContext = React.createContext<Record<string, string>>({});

export const PageContext = React.createContext<{ pageNum: number; totalPages: number }>({
  pageNum: 1,
  totalPages: 1,
});

function hasFullBleedImage(nodes?: DocNode[]): boolean {
  if (!nodes) return false;
  return nodes.some(n => {
    if (n.type === 'Element') {
      const el = n as ElementNode;
      if (el.tag.toLowerCase() === 'img') {
        if (el.classes?.includes('full-bleed') || el.inline_styles?.width === '100%') {
          return true;
        }
      }
      return hasFullBleedImage(el.children);
    }
    return false;
  });
}

function hasImageNode(nodes?: DocNode[]): boolean {
  if (!nodes) return false;
  return nodes.some(n => {
    if (n.type === 'Element') {
      const el = n as ElementNode;
      if (el.tag.toLowerCase() === 'img') {
        return true;
      }
      return hasImageNode(el.children);
    }
    return false;
  });
}

function extractElementPlainText(node: DocNode): string {
  if (node.type === 'Text') return node.content;
  if (node.type === 'Element') {
    return (node as ElementNode).children.map(extractElementPlainText).join(' ');
  }
  return '';
}

function estimateNodeHeight(node: DocNode): number {
  if (node.type === 'Text') {
    const text = node.content.trim();
    if (!text) return 0;
    const lines = Math.max(1, Math.ceil(text.length / 75));
    return lines * 22;
  }
  if (node.type === 'Element') {
    const el = node as ElementNode;
    const tag = el.tag.toLowerCase();
    if (tag === 'page-break') return 0;
    if (tag === 'h1') return 55;
    if (tag === 'h2') return 45;
    if (tag === 'h3') return 38;
    if (tag === 'p') {
      const text = extractElementPlainText(node);
      const lines = Math.max(1, Math.ceil(text.length / 80));
      return lines * 24 + 14;
    }
    if (tag === 'table') {
      const rowCount = el.children.filter(c => c.type === 'Element' && (c as ElementNode).tag.toLowerCase() === 'tr').length || 1;
      return rowCount * 36 + 20;
    }
    if (tag === 'ul' || tag === 'ol') {
      const liCount = el.children.filter(c => c.type === 'Element' && (c as ElementNode).tag.toLowerCase() === 'li').length || 1;
      return liCount * 28 + 16;
    }
    if (tag === 'img') {
      const styleH = el.inline_styles?.['height'] || el.inline_styles?.['max-height'];
      if (styleH && styleH.endsWith('mm')) {
        return parseFloat(styleH) * 3.78;
      }
      return 180;
    }
    if (tag === 'grid') return 160;
    if (tag === 'callout') return 75;
    if (tag === 'blockquote') return 60;
  }
  return 30;
}

function splitLargeBlocks(nodes: DocNode[]): DocNode[] {
  const result: DocNode[] = [];

  for (const node of nodes) {
    if (node.type === 'Element') {
      const el = node as ElementNode;
      const tagLower = el.tag.toLowerCase();

      // Split large unordered or ordered lists so items flow across pages naturally
      if (tagLower === 'ul' || tagLower === 'ol') {
        const liItems = el.children.filter(c => c.type === 'Element' && (c as ElementNode).tag.toLowerCase() === 'li');
        if (liItems.length > 7) {
          const chunkSize = 6;
          const isOl = tagLower === 'ol';
          const baseStart = isOl && el.attributes['start'] ? parseInt(el.attributes['start'], 10) : 1;

          for (let i = 0; i < liItems.length; i += chunkSize) {
            const chunkLis = liItems.slice(i, i + chunkSize);
            const chunkAttrs = { ...el.attributes };
            if (isOl) {
              chunkAttrs['start'] = String(baseStart + i);
            }
            result.push({
              ...el,
              attributes: chunkAttrs,
              children: chunkLis,
            });
          }
          continue;
        }
      }

      // Split large tables so rows flow across pages cleanly
      if (tagLower === 'table') {
        const rows = el.children.filter(c => c.type === 'Element' && (c as ElementNode).tag.toLowerCase() === 'tr');
        if (rows.length > 7) {
          const hasHeader = rows.length > 0 && rows[0].type === 'Element' &&
            (rows[0] as ElementNode).children.some(c => c.type === 'Element' && (c as ElementNode).tag.toLowerCase() === 'th');
          const headerRow = hasHeader ? rows[0] : null;
          const bodyRows = hasHeader ? rows.slice(1) : rows;
          const chunkSize = 6;

          for (let i = 0; i < bodyRows.length; i += chunkSize) {
            const chunkRows = bodyRows.slice(i, i + chunkSize);
            const tableRows = headerRow ? [headerRow, ...chunkRows] : chunkRows;
            result.push({
              ...el,
              children: tableRows,
            });
          }
          continue;
        }
      }
    }

    result.push(node);
  }

  return result;
}

export const DocumentRenderer: React.FC<DocumentRendererProps> = ({ doc, error, assets = {} }) => {
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

  // Generate CSS rules from stylesheet
  const customCss = useMemo(() => {
    if (!doc.props?.stylesheet?.rules) return '';
    return doc.props.stylesheet.rules.map(rule => {
      const decls = Object.entries(rule.declarations)
        .map(([k, v]) => `  ${k}: ${v} !important;`)
        .join('\n');
      const selectors = rule.selectors.map(s => {
        const trimmed = s.trim();
        if (trimmed === 'body' || trimmed === ':root' || trimmed === 'html') {
          return '.ddl-paper';
        }
        if (trimmed.startsWith('body ')) {
          return `.ddl-paper ${trimmed.slice(5)}`;
        }
        return `.ddl-paper ${trimmed}`;
      }).join(', ');
      return `${selectors} {\n${decls}\n}`;
    }).join('\n\n');
  }, [doc.props?.stylesheet]);

  const paperSize = doc.props?.paper?.size || 'A4';
  const orientation = doc.props?.paper?.orientation || 'portrait';
  const marginMm = doc.props?.paper?.margin?.val ?? 20;

  const isLandscape = orientation === 'landscape';
  const isLetter = paperSize.toUpperCase() === 'LETTER';
  const widthMm = isLetter ? (isLandscape ? 279.4 : 215.9) : (isLandscape ? 297 : 210);
  const heightMm = isLetter ? (isLandscape ? 215.9 : 279.4) : (isLandscape ? 210 : 297);

  // Group nodes by section
  const sections = useMemo(() => {
    const list: { pageStyleName: string; nodes: DocNode[] }[] = [];
    const defaultStyle = doc.props.default_page_style || 'default';
    let currentNodes: DocNode[] = [];

    const commitSection = (styleName: string) => {
      const meaningfulNodes = currentNodes.filter(n => {
        if (n.type === 'Text') {
          return n.content.trim().length > 0;
        }
        return true;
      });

      if (meaningfulNodes.length > 0) {
        list.push({ pageStyleName: styleName, nodes: meaningfulNodes });
      }
      currentNodes = [];
    };

    for (const child of doc.body.children) {
      if (child.type === 'Section') {
        commitSection(defaultStyle);
        const styleName = child.page_style || defaultStyle;
        const sectionNodes = child.children.filter(n => {
          if (n.type === 'Text') {
            return n.content.trim().length > 0;
          }
          return true;
        });
        list.push({ pageStyleName: styleName, nodes: sectionNodes });
      } else {
        currentNodes.push(child);
      }
    }
    commitSection(defaultStyle);

    if (list.length === 0) {
      list.push({ pageStyleName: defaultStyle, nodes: [] });
    }
    return list;
  }, [doc]);

  // Pre-split large lists and tables so they paginate smoothly across pages
  const sectionsWithSplitBlocks = useMemo(() => {
    return sections.map(sec => ({
      pageStyleName: sec.pageStyleName,
      blocks: splitLargeBlocks(sec.nodes),
    }));
  }, [sections]);

  // Measure exact rendered heights in DOM
  const measureContainerRef = useRef<HTMLDivElement>(null);
  const [measuredHeights, setMeasuredHeights] = useState<Record<string, number>>({});

  useLayoutEffect(() => {
    if (!measureContainerRef.current) return;
    const children = Array.from(measureContainerRef.current.children) as HTMLElement[];
    const heights: Record<string, number> = {};
    for (const child of children) {
      const sIdx = child.getAttribute('data-sec-idx');
      const bIdx = child.getAttribute('data-block-idx');
      if (sIdx !== null && bIdx !== null) {
        const rect = child.getBoundingClientRect();
        const style = window.getComputedStyle(child);
        const mt = parseFloat(style.marginTop) || 0;
        const mb = parseFloat(style.marginBottom) || 0;
        heights[`${sIdx}-${bIdx}`] = Math.ceil(rect.height + mt + mb);
      }
    }
    setMeasuredHeights(heights);
  }, [sectionsWithSplitBlocks, widthMm, marginMm, customCss]);

  interface RenderedPage {
    pageStyleName: string;
    pageNum: number;
    nodes: DocNode[];
  }

  // Calculate discrete physical pages based on available height per sheet
  const pages = useMemo<RenderedPage[]>(() => {
    const result: RenderedPage[] = [];
    let globalPageNum = 1;

    for (let sIdx = 0; sIdx < sectionsWithSplitBlocks.length; sIdx++) {
      const sec = sectionsWithSplitBlocks[sIdx];
      const pageStyle: PageStyle | undefined = doc.props?.page_styles?.[sec.pageStyleName];
      const hasFullBleedFooter = pageStyle?.running_footer && (
        hasFullBleedImage(pageStyle.running_footer.center) ||
        hasFullBleedImage(pageStyle.running_footer.left) ||
        hasFullBleedImage(pageStyle.running_footer.right)
      );

      const pxPerMm = 96 / 25.4;
      const pageHeightPx = heightMm * pxPerMm;
      const topMarginPx = marginMm * pxPerMm;
      const bottomMarginPx = (hasFullBleedFooter ? 36 : marginMm) * pxPerMm;

      let headerHeightPx = 0;
      if (pageStyle?.running_header) {
        const hasImg = hasImageNode(pageStyle.running_header.center) ||
                       hasImageNode(pageStyle.running_header.left) ||
                       hasImageNode(pageStyle.running_header.right);
        headerHeightPx = (hasImg ? 38 : 14) * pxPerMm;
      }

      let footerHeightPx = 0;
      if (pageStyle?.running_footer) {
        if (!hasFullBleedFooter) {
          const hasImg = hasImageNode(pageStyle.running_footer.center) ||
                         hasImageNode(pageStyle.running_footer.left) ||
                         hasImageNode(pageStyle.running_footer.right);
          footerHeightPx = (hasImg ? 28 : 14) * pxPerMm;
        }
      }

      // Safety buffer to ensure text never pushes over physical page boundaries
      const availableHeightPx = Math.max(200, pageHeightPx - topMarginPx - bottomMarginPx - headerHeightPx - footerHeightPx - 30);

      let curPageNodes: DocNode[] = [];
      let curH = 0;

      for (let bIdx = 0; bIdx < sec.blocks.length; bIdx++) {
        const block = sec.blocks[bIdx];
        const isPageBreak = block.type === 'Element' && (
          (block as ElementNode).tag.toLowerCase() === 'page-break' ||
          (block as ElementNode).inline_styles?.['page-break-before'] === 'always' ||
          (block as ElementNode).inline_styles?.['page-break-after'] === 'always'
        );

        if (isPageBreak) {
          if (curPageNodes.length > 0) {
            result.push({
              pageStyleName: sec.pageStyleName,
              pageNum: globalPageNum++,
              nodes: curPageNodes,
            });
            curPageNodes = [];
            curH = 0;
          }
          continue;
        }

        const key = `${sIdx}-${bIdx}`;
        const blockH = measuredHeights[key] || estimateNodeHeight(block);

        if (curH + blockH > availableHeightPx && curPageNodes.length > 0) {
          result.push({
            pageStyleName: sec.pageStyleName,
            pageNum: globalPageNum++,
            nodes: curPageNodes,
          });
          curPageNodes = [block];
          curH = blockH;
        } else {
          curPageNodes.push(block);
          curH += blockH;
        }
      }

      if (curPageNodes.length > 0 || result.length === 0) {
        result.push({
          pageStyleName: sec.pageStyleName,
          pageNum: globalPageNum++,
          nodes: curPageNodes,
        });
      }
    }

    return result;
  }, [sectionsWithSplitBlocks, measuredHeights, heightMm, widthMm, marginMm, doc.props?.page_styles]);

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
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '40px',
        alignItems: 'center',
        paddingBottom: '60px',
      }}>
        {pages.map((page, pageIdx) => {
          const pageStyle: PageStyle | undefined = doc.props.page_styles[page.pageStyleName];
          const hasFullBleedFooter = pageStyle?.running_footer && (
            hasFullBleedImage(pageStyle.running_footer.center) ||
            hasFullBleedImage(pageStyle.running_footer.left) ||
            hasFullBleedImage(pageStyle.running_footer.right)
          );
          return (
            <div key={pageIdx} className="page-wrapper" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
              {/* Page Indicator Header */}
              <div
                className="page-header-indicator"
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
                }}
              >
                <span>Page {page.pageNum} of {pages.length}</span>
                <span>{paperSize} &bull; {orientation}</span>
              </div>

              {/* Physical Paper Sheet */}
              <div
                className="ddl-paper"
                data-page-number={page.pageNum}
                style={{
                  width: `${widthMm}mm`,
                  height: `${heightMm}mm`,
                  minHeight: `${heightMm}mm`,
                  maxHeight: `${heightMm}mm`,
                  backgroundColor: '#ffffff',
                  boxShadow: '0 8px 30px rgba(0, 0, 0, 0.28)',
                  padding: `${marginMm}mm`,
                  paddingBottom: hasFullBleedFooter ? '36mm' : `${marginMm}mm`,
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
                <PageContext.Provider value={{ pageNum: page.pageNum, totalPages: pages.length }}>
                  {/* Watermark */}
                  {pageStyle?.watermark && (
                    <div style={{
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
                    }}>
                      {pageStyle.watermark.text}
                    </div>
                  )}

                  {/* Running Header */}
                  {pageStyle?.running_header && (() => {
                    const hasImg = hasImageNode(pageStyle.running_header.center) ||
                                   hasImageNode(pageStyle.running_header.left) ||
                                   hasImageNode(pageStyle.running_header.right);
                    const hasLeft = pageStyle.running_header.left && pageStyle.running_header.left.length > 0;
                    const hasCenter = pageStyle.running_header.center && pageStyle.running_header.center.length > 0;
                    const hasRight = pageStyle.running_header.right && pageStyle.running_header.right.length > 0;

                    if (hasCenter && !hasLeft && !hasRight) {
                      return (
                        <header className="running-header header-center-only" style={{
                          width: '100%',
                          textAlign: 'center',
                          display: 'flex',
                          flexDirection: 'column',
                          alignItems: 'center',
                          justifyContent: 'center',
                          borderBottom: hasImg ? 'none' : '1px solid #e2e8f0',
                          paddingBottom: hasImg ? '0' : '3mm',
                          marginBottom: hasImg ? '3mm' : '8mm',
                          fontSize: '9pt',
                          userSelect: 'none',
                        }}>
                          <div className="header-center" style={{ width: '100%', display: 'flex', justifyContent: 'center' }}>
                            <RenderChildren nodes={pageStyle.running_header.center} />
                          </div>
                        </header>
                      );
                    }

                    return (
                      <header className="running-header" style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        borderBottom: hasImg ? 'none' : '1px solid #e2e8f0',
                        paddingBottom: hasImg ? '0' : '3mm',
                        marginBottom: hasImg ? '4mm' : '8mm',
                        fontSize: '9pt',
                        color: '#64748b',
                        userSelect: 'none',
                      }}>
                        <div className="header-left" style={{ flex: hasLeft ? 1 : 0, textAlign: 'left' }}><RenderChildren nodes={pageStyle.running_header.left} /></div>
                        <div className="header-center" style={{ flex: hasCenter ? 2 : 0, textAlign: 'center' }}><RenderChildren nodes={pageStyle.running_header.center} /></div>
                        <div className="header-right" style={{ flex: hasRight ? 1 : 0, textAlign: 'right' }}><RenderChildren nodes={pageStyle.running_header.right} /></div>
                      </header>
                    );
                  })()}

                  {/* Page Content Body */}
                  <div style={{ position: 'relative', zIndex: 1, flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
                    <RenderChildren nodes={page.nodes} />
                  </div>

                  {/* Running Footer */}
                  {pageStyle?.running_footer && (() => {
                    if (hasFullBleedFooter) {
                      const bleedNodes = (pageStyle.running_footer.center && pageStyle.running_footer.center.length > 0)
                        ? pageStyle.running_footer.center
                        : ((pageStyle.running_footer.left && pageStyle.running_footer.left.length > 0)
                          ? pageStyle.running_footer.left
                          : pageStyle.running_footer.right);
                      return (
                        <footer className="running-footer full-bleed" style={{
                          position: 'absolute',
                          bottom: 0,
                          left: 0,
                          width: '100%',
                          lineHeight: 0,
                          zIndex: 2,
                          userSelect: 'none',
                        }}>
                          <RenderChildren nodes={bleedNodes} />
                        </footer>
                      );
                    }
                    const hasImg = hasImageNode(pageStyle.running_footer.center) ||
                                   hasImageNode(pageStyle.running_footer.left) ||
                                   hasImageNode(pageStyle.running_footer.right);
                    return (
                      <footer className="running-footer" style={{
                        marginTop: 'auto',
                        borderTop: hasImg ? 'none' : '1px solid #e2e8f0',
                        paddingTop: hasImg ? '0' : '3mm',
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        fontSize: '9pt',
                        color: '#94a3b8',
                        userSelect: 'none',
                      }}>
                        <div className="footer-left" style={{ flex: 1, textAlign: 'left' }}><RenderChildren nodes={pageStyle.running_footer.left} /></div>
                        <div className="footer-center" style={{ flex: 1, textAlign: 'center' }}><RenderChildren nodes={pageStyle.running_footer.center} /></div>
                        <div className="footer-right" style={{ flex: 1, textAlign: 'right' }}><RenderChildren nodes={pageStyle.running_footer.right} /></div>
                      </footer>
                    );
                  })()}
                </PageContext.Provider>
              </div>
            </div>
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

export const RenderChildren: React.FC<{ nodes: DocNode[] }> = ({ nodes }) => {
  if (!nodes || nodes.length === 0) return null;
  return (
    <>
      {nodes.map((node, i) => (
        <RenderNode key={i} node={node} />
      ))}
    </>
  );
};

export const RenderNode: React.FC<{ node: DocNode }> = ({ node }) => {
  if (node.type === 'Text') {
    return <>{node.content}</>;
  }

  if (node.type === 'Comment') {
    return null;
  }

  if (node.type === 'Section') {
    return <RenderChildren nodes={node.children} />;
  }

  if (node.type === 'Element') {
    return <RenderElement el={node} />;
  }

  return null;
};

const RenderElement: React.FC<{ el: ElementNode }> = ({ el }) => {
  const { tag, attributes, classes, inline_styles, children } = el;
  const tagLower = tag.toLowerCase();

  const styleObj: React.CSSProperties = {};
  for (const [k, v] of Object.entries(inline_styles || {})) {
    const camel = k.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
    (styleObj as Record<string, string>)[camel] = v;
  }

  // Handle align attribute e.g. align="center"
  if (attributes['align']) {
    const alignVal = attributes['align'].toLowerCase();
    styleObj.textAlign = alignVal === 'both' ? 'justify' : (alignVal as any);
  }

  const className = classes ? classes.join(' ') : undefined;

  // Void Elements: br, hr, img
  if (tagLower === 'br') {
    return <br className={className} />;
  }

  if (tagLower === 'hr') {
    return (
      <hr
        className={className}
        style={{
          border: 'none',
          borderBottom: '1px solid #cbd5e1',
          margin: '4mm 0',
          width: '100%',
          ...styleObj,
        }}
      />
    );
  }

  if (tagLower === 'img') {
    const assets = React.useContext(AssetContext);
    const rawSrc = attributes['src'] || '';
    const resolvedSrc = (assets && assets[rawSrc]) ? assets[rawSrc] : rawSrc;
    return (
      <img
        src={resolvedSrc}
        alt={attributes['alt'] || ''}
        className={className}
        style={{
          maxWidth: '100%',
          display: 'block',
          margin: '0 auto',
          ...styleObj,
        }}
      />
    );
  }

  if (tagLower === 'center') {
    return (
      <div
        className={className}
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          textAlign: 'center',
          width: '100%',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </div>
    );
  }

  if (tagLower === 'left') {
    return (
      <div
        className={className}
        style={{
          textAlign: 'left',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </div>
    );
  }

  if (tagLower === 'right') {
    return (
      <div
        className={className}
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'flex-end',
          textAlign: 'right',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </div>
    );
  }

  // Grid Element
  if (tagLower === 'grid') {
    const cols = attributes['cols'] || '1fr 1fr';
    const gap = attributes['gap'] || '16px';
    return (
      <div
        className={className}
        style={{
          display: 'grid',
          gridTemplateColumns: cols,
          gap,
          width: '100%',
          boxSizing: 'border-box',
          ...styleObj,
        }}
      >
        <RenderChildren nodes={children} />
      </div>
    );
  }

  if (tagLower === 'page-number') {
    const { pageNum } = React.useContext(PageContext);
    return <span className={className} style={styleObj}>{pageNum}</span>;
  }

  if (tagLower === 'total-pages') {
    const { totalPages } = React.useContext(PageContext);
    return <span className={className} style={styleObj}>{totalPages}</span>;
  }

  if (tagLower === 'callout') {
    return (
      <div
        className={`callout ${className || ''}`}
        style={{
          background: '#f0fdf4',
          borderLeft: '4pt solid #16a34a',
          padding: '4mm 6mm',
          margin: '4mm 0',
          borderRadius: '4px',
          textAlign: 'left',
          ...styleObj,
        }}
      >
        <RenderChildren nodes={children} />
      </div>
    );
  }

  // Default table styling
  if (tagLower === 'table') {
    const isBorderless = classes?.includes('borderless') || styleObj.border === 'none';
    const isAlignRight = classes?.includes('align-right') || attributes['align'] === 'right';
    return (
      <table
        className={className}
        style={{
          width: isAlignRight ? '48%' : '100%',
          marginLeft: isAlignRight ? 'auto' : undefined,
          marginRight: isAlignRight ? '0' : undefined,
          borderCollapse: 'collapse',
          margin: '4mm 0',
          border: isBorderless ? 'none' : undefined,
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </table>
    );
  }

  if (tagLower === 'th') {
    const isBorderless = classes?.includes('borderless') || styleObj.border === 'none';
    return (
      <th
        className={className}
        style={{
          border: isBorderless ? 'none' : '1px solid #cbd5e1',
          padding: '2mm 3mm',
          textAlign: 'left',
          backgroundColor: isBorderless ? 'transparent' : '#f8fafc',
          fontWeight: 'bold',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </th>
    );
  }

  if (tagLower === 'td') {
    const isBorderless = classes?.includes('borderless') || styleObj.border === 'none';
    return (
      <td
        className={className}
        style={{
          border: isBorderless ? 'none' : '1px solid #cbd5e1',
          padding: '2mm 3mm',
          textAlign: 'left',
          backgroundColor: isBorderless ? 'transparent' : undefined,
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </td>
    );
  }

  if (tagLower === 'ul') {
    const rawType = attributes['type'] || inline_styles?.['list-style-type'];
    let cssListType: string | undefined;
    if (rawType) {
      const lower = rawType.toLowerCase();
      if (lower === 'circle') cssListType = 'circle';
      else if (lower === 'square') cssListType = 'square';
      else if (lower === 'dash') cssListType = '"- "';
      else if (lower === 'none') cssListType = 'none';
      else if (lower === 'disc') cssListType = 'disc';
    }

    return (
      <ul
        className={className}
        data-type={rawType}
        style={{
          listStyleType: cssListType,
          listStylePosition: 'outside',
          textAlign: 'left',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </ul>
    );
  }

  if (tagLower === 'ol') {
    const rawType = attributes['type'] || inline_styles?.['list-style-type'];
    let cssListType: string | undefined;
    let htmlType: '1' | 'a' | 'A' | 'i' | 'I' | undefined;

    if (rawType) {
      const lower = rawType.toLowerCase();
      if (rawType === 'A' || lower === 'upper-alpha') {
        cssListType = 'upper-alpha';
        htmlType = 'A';
      } else if (rawType === 'I' || lower === 'upper-roman') {
        cssListType = 'upper-roman';
        htmlType = 'I';
      } else if (rawType === 'a' || lower === 'lower-alpha') {
        cssListType = 'lower-alpha';
        htmlType = 'a';
      } else if (rawType === 'i' || lower === 'lower-roman') {
        cssListType = 'lower-roman';
        htmlType = 'i';
      } else if (rawType === '1' || lower === 'decimal') {
        cssListType = 'decimal';
        htmlType = '1';
      } else if (lower === 'none') {
        cssListType = 'none';
      }
    }

    const startAttr = attributes['start'] ? parseInt(attributes['start'], 10) : undefined;

    return (
      <ol
        className={className}
        start={startAttr}
        type={htmlType}
        data-type={rawType}
        style={{
          listStyleType: cssListType,
          listStylePosition: 'outside',
          textAlign: 'left',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </ol>
    );
  }

  if (tagLower === 'li') {
    return (
      <li
        className={className}
        style={{
          marginBottom: '1.5mm',
          lineHeight: 1.6,
          textAlign: 'left',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </li>
    );
  }

  const validTags = new Set([
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'p', 'b', 'strong', 'i', 'em', 'u', 's', 'del', 'mark', 'sup', 'sub', 'small',
    'div', 'span', 'code', 'pre', 'blockquote',
    'thead', 'tbody', 'tfoot', 'tr',
    'ul', 'ol', 'li'
  ]);

  const HtmlTag = validTags.has(tagLower) ? (tagLower as keyof React.JSX.IntrinsicElements) : 'div';

  return React.createElement(
    HtmlTag,
    {
      className,
      style: Object.keys(styleObj).length > 0 ? styleObj : undefined,
      ...attributes,
    },
    <RenderChildren nodes={children} />
  );
};
