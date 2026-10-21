export function formatDdl(input: string): string {
  // 1. Temporarily extract <style> blocks
  const styles: string[] = [];
  const placeholderPrefix = '___STYLE_BLOCK_';

  const withPlaceholders = input.replace(/(<style\b[^>]*>)([\s\S]*?)(<\/style>)/gi, (_, openTag, cssBody) => {
    const idx = styles.length;
    styles.push(cssBody);
    return `${openTag}\n${placeholderPrefix}${idx}___\n</style>`;
  });

  // 2. Parse and re-serialize through a recursive tree builder
  const formattedXml = formatXmlWithTree(withPlaceholders);

  // 3. Re-inject formatted CSS into the placeholder blocks with proper indentation
  return formattedXml.replace(new RegExp(`${placeholderPrefix}(\\d+)___`, 'g'), (_, indexStr) => {
    const idx = parseInt(indexStr, 10);
    const rawCss = styles[idx] || '';
    return formatCss(rawCss, '      ');
  });
}

export function formatCss(css: string, indent: string = '      '): string {
  const lines: string[] = [];
  const rules = css.split('}');

  for (const rule of rules) {
    const cleanRule = rule.trim();
    if (!cleanRule) continue;

    const [selectorPart, declsPart] = cleanRule.split('{');
    if (!selectorPart || declsPart === undefined) continue;

    const selector = selectorPart.trim();
    lines.push(`${indent}${selector} {`);

    const decls = declsPart.split(';');
    for (const decl of decls) {
      const cleanDecl = decl.trim();
      if (!cleanDecl) continue;
      const [prop, val] = cleanDecl.split(':');
      if (prop && val) {
        lines.push(`${indent}  ${prop.trim()}: ${val.trim()};`);
      }
    }
    lines.push(`${indent}}\n`);
  }

  return lines.join('\n').trim();
}

/**
 * True DOM Tree-based XML Formatter.
 * Serializes nodes recursively, ensuring exact indentation depth for parents and children.
 */
function formatXmlWithTree(xmlString: string): string {
  const parser = new DOMParser();
  const xmlDoc = parser.parseFromString(xmlString, 'text/xml');

  // If there are XML parse errors, fall back safely
  if (xmlDoc.querySelector('parsererror')) {
    return xmlString;
  }

  const root = xmlDoc.documentElement;
  return serializeNode(root, 0);
}

function serializeNode(node: Node, depth: number): string {
  const indent = '  '.repeat(depth);

  if (node.nodeType === Node.TEXT_NODE) {
    const text = node.textContent?.trim() || '';
    return text;
  }

  if (node.nodeType === Node.COMMENT_NODE) {
    return `${indent}<!--${node.textContent}-->`;
  }

  if (node.nodeType === Node.ELEMENT_NODE) {
    const el = node as Element;
    const tagName = el.tagName;

    // Build attributes string
    let attrs = '';
    for (let i = 0; i < el.attributes.length; i++) {
      const attr = el.attributes[i];
      attrs += ` ${attr.name}="${attr.value}"`;
    }

    // Check children
    const childNodes = Array.from(el.childNodes).filter(child => {
      if (child.nodeType === Node.TEXT_NODE) {
        return (child.textContent || '').trim().length > 0;
      }
      return true;
    });

    // Self closing tags
    const selfClosingTags = new Set(['paper', 'watermark', 'page-number', 'total-pages', 'hr', 'img', 'br']);
    if (childNodes.length === 0) {
      if (selfClosingTags.has(tagName.toLowerCase())) {
        return `${indent}<${tagName}${attrs} />`;
      }
      return `${indent}<${tagName}${attrs}></${tagName}>`;
    }

    // Single inline text child (e.g. <title>My Title</title> or <b>text</b>)
    if (childNodes.length === 1 && childNodes[0].nodeType === Node.TEXT_NODE) {
      const textContent = childNodes[0].textContent?.trim() || '';
      return `${indent}<${tagName}${attrs}>${textContent}</${tagName}>`;
    }

    // Multi-line block with nested children
    const renderedChildren = childNodes
      .map(child => serializeNode(child, depth + 1))
      .filter(Boolean)
      .join('\n');

    return `${indent}<${tagName}${attrs}>\n${renderedChildren}\n${indent}</${tagName}>`;
  }

  return '';
}
