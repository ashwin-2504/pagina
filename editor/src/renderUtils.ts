import type { ParsedDocument, DocNode, ElementNode } from './types';

/** Check whether any node in the tree is an `<img>` element. */
export function hasImageNode(nodes?: DocNode[]): boolean {
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

/** Recursively extract all plain text from a node tree. */
export function extractElementPlainText(node: DocNode): string {
  if (node.type === 'Text') return node.content;
  if (node.type === 'Element') {
    return (node as ElementNode).children.map(extractElementPlainText).join(' ');
  }
  return '';
}

/** Search the document AST for a text snippet and return its source line. */
export function findLineByText(doc: ParsedDocument, text: string): number | null {
  const clean = text.trim().slice(0, 40).replace(/\s+/g, ' ').toLowerCase();
  if (!clean) return null;

  function searchNodes(nodes: DocNode[]): number | null {
    for (const node of nodes) {
      if (node.type === 'Text') {
        const contentClean = node.content.replace(/\s+/g, ' ').toLowerCase();
        if (contentClean.includes(clean) || clean.includes(contentClean)) {
          if (node.span?.line && node.span.line > 0) return node.span.line;
        }
      } else if (node.type === 'Element') {
        const childMatch = searchNodes(node.children);
        if (childMatch) return childMatch;
        const plain = extractElementPlainText(node).replace(/\s+/g, ' ').toLowerCase();
        if (plain.includes(clean) && node.span?.line && node.span.line > 0) {
          return node.span.line;
        }
      } else if (node.type === 'Section') {
        const childMatch = searchNodes(node.children);
        if (childMatch) return childMatch;
      }
    }
    return null;
  }

  const bodyMatch = searchNodes(doc.body.children);
  if (bodyMatch) return bodyMatch;

  for (const style of Object.values(doc.props.page_styles || {})) {
    if (style.running_header) {
      const match = searchNodes([
        ...(style.running_header.left || []),
        ...(style.running_header.center || []),
        ...(style.running_header.right || []),
      ]);
      if (match) return match;
    }
    if (style.running_footer) {
      const match = searchNodes([
        ...(style.running_footer.left || []),
        ...(style.running_footer.center || []),
        ...(style.running_footer.right || []),
      ]);
      if (match) return match;
    }
  }

  return null;
}

/** Heuristic pixel height estimate for a single AST block node. */
export function estimateNodeHeight(node: DocNode): number {
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

/**
 * Split large lists and tables into smaller chunks so they paginate
 * smoothly across pages without a single massive block.
 */
export function splitLargeBlocks(nodes: DocNode[]): DocNode[] {
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
