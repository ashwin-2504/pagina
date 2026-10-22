import React from 'react';
import type { ElementNode } from './types';
import { AssetContext, PageContext, RenderChildren } from './RenderContext';
import { renderListElement } from './RenderList';

export { AssetContext, PageContext, RenderChildren };

export const RenderElement: React.FC<{ el: ElementNode }> = ({ el }) => {
  const { tag, attributes, classes, inline_styles, children, span } = el;
  const tagLower = tag.toLowerCase();
  const line = span?.line || (attributes && attributes['data-ddl-line'] ? parseInt(attributes['data-ddl-line'], 10) : undefined);

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
        data-ddl-line={line}
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
        data-ddl-line={line}
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
    return <span className={className} data-ddl-line={line} style={styleObj}>{pageNum}</span>;
  }

  if (tagLower === 'total-pages') {
    const { totalPages } = React.useContext(PageContext);
    return <span className={className} data-ddl-line={line} style={styleObj}>{totalPages}</span>;
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

  const listEl = renderListElement(tagLower, el, className, styleObj);
  if (listEl) {
    return listEl;
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
      'data-ddl-line': line,
      style: Object.keys(styleObj).length > 0 ? styleObj : undefined,
      ...attributes,
    },
    <RenderChildren nodes={children} />
  );
};
