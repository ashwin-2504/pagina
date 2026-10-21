import React, { useMemo } from 'react';
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
      if (el.tag.toLowerCase() === 'img') return true;
      return hasImageNode(el.children);
    }
    return false;
  });
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
  const minHeightMm = isLetter ? (isLandscape ? 215.9 : 279.4) : (isLandscape ? 210 : 297);

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

  return (
    <AssetContext.Provider value={assets}>
      {customCss && <style>{customCss}</style>}
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '40px',
        alignItems: 'center',
        paddingBottom: '60px',
      }}>
        {sections.map((sec, idx) => {
          const pageStyle: PageStyle | undefined = doc.props.page_styles[sec.pageStyleName];
          const hasFullBleedFooter = pageStyle?.running_footer && (
            hasFullBleedImage(pageStyle.running_footer.center) ||
            hasFullBleedImage(pageStyle.running_footer.left) ||
            hasFullBleedImage(pageStyle.running_footer.right)
          );
          return (
            <div
              key={idx}
              className="ddl-paper"
              style={{
                width: `${widthMm}mm`,
                minHeight: `${minHeightMm}mm`,
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
                    <div style={{
                      width: '100%',
                      textAlign: 'center',
                      borderBottom: hasImg ? 'none' : '1px solid #e2e8f0',
                      paddingBottom: hasImg ? '0' : '3mm',
                      marginBottom: hasImg ? '3mm' : '8mm',
                      fontSize: '9pt',
                      userSelect: 'none',
                    }}>
                      <RenderChildren nodes={pageStyle.running_header.center} />
                    </div>
                  );
                }

                return (
                  <div style={{
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
                    <div style={{ flex: hasLeft ? 1 : 0, textAlign: 'left' }}><RenderChildren nodes={pageStyle.running_header.left} /></div>
                    <div style={{ flex: hasCenter ? 2 : 0, textAlign: 'center' }}><RenderChildren nodes={pageStyle.running_header.center} /></div>
                    <div style={{ flex: hasRight ? 1 : 0, textAlign: 'right' }}><RenderChildren nodes={pageStyle.running_header.right} /></div>
                  </div>
                );
              })()}

              {/* Page Content Body */}
              <div style={{ position: 'relative', zIndex: 1, flex: 1, display: 'flex', flexDirection: 'column' }}>
                <RenderChildren nodes={sec.nodes} />
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
                    <div style={{
                      position: 'absolute',
                      bottom: 0,
                      left: 0,
                      width: '100%',
                      lineHeight: 0,
                      zIndex: 2,
                      userSelect: 'none',
                    }}>
                      <RenderChildren nodes={bleedNodes} />
                    </div>
                  );
                }
                const hasImg = hasImageNode(pageStyle.running_footer.center) ||
                               hasImageNode(pageStyle.running_footer.left) ||
                               hasImageNode(pageStyle.running_footer.right);
                return (
                  <div style={{
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
                    <div style={{ flex: 1, textAlign: 'left' }}><RenderChildren nodes={pageStyle.running_footer.left} /></div>
                    <div style={{ flex: 1, textAlign: 'center' }}><RenderChildren nodes={pageStyle.running_footer.center} /></div>
                    <div style={{ flex: 1, textAlign: 'right' }}><RenderChildren nodes={pageStyle.running_footer.right} /></div>
                  </div>
                );
              })()}
            </div>
          );
        })}
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
          ...styleObj,
        }}
      />
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
    return <span>1</span>;
  }

  if (tagLower === 'total-pages') {
    return <span>1</span>;
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
