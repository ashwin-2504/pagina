import type { DocNode } from './types';

/**
 * Pre-annotates DXML markup by injecting `data-ddl-line="<line>"` into every opening tag.
 * Does not insert any newlines, preserving all 1-based source line numbers exactly.
 */
export function annotateDxmlWithLineNumbers(input: string): string {
  let result = '';
  let i = 0;
  const len = input.length;
  let currentLine = 1;

  while (i < len) {
    const ch = input[i];

    if (ch === '\n') {
      currentLine++;
      result += ch;
      i++;
      continue;
    }

    if (ch === '<') {
      // 1. Skip XML Comments: <!-- ... -->
      if (input.startsWith('<!--', i)) {
        const endComment = input.indexOf('-->', i + 4);
        const chunk = endComment !== -1 ? input.slice(i, endComment + 3) : input.slice(i);
        for (let c = 0; c < chunk.length; c++) {
          if (chunk[c] === '\n') currentLine++;
        }
        result += chunk;
        i += chunk.length;
        continue;
      }

      // 2. Skip CDATA: <![CDATA[ ... ]]>
      if (input.startsWith('<![CDATA[', i)) {
        const endCdata = input.indexOf(']]>', i + 9);
        const chunk = endCdata !== -1 ? input.slice(i, endCdata + 3) : input.slice(i);
        for (let c = 0; c < chunk.length; c++) {
          if (chunk[c] === '\n') currentLine++;
        }
        result += chunk;
        i += chunk.length;
        continue;
      }

      // 3. Skip Closing Tags: </tag>
      if (input[i + 1] === '/') {
        result += ch;
        i++;
        continue;
      }

      // 4. Skip Processing Instructions or DOCTYPE: <?...?> or <!DOCTYPE ...>
      if (input[i + 1] === '?' || input[i + 1] === '!') {
        result += ch;
        i++;
        continue;
      }

      // 5. Opening Tag: <tag ...
      const nextChar = input[i + 1];
      if (nextChar && /[a-zA-Z_]/.test(nextChar)) {
        const tagLine = currentLine;
        result += ch;
        i++; // skip '<'

        let tagName = '';
        while (i < len && /[a-zA-Z0-9_\-:]/.test(input[i])) {
          tagName += input[i];
          i++;
        }
        result += tagName;
        result += ` data-ddl-line="${tagLine}"`;
        continue;
      }
    }

    result += ch;
    i++;
  }

  return result;
}

export function cleanErrorMessage(raw: string): string {
  // Strip Chrome/Firefox boilerplates
  return raw
    .replace(/^error on line \d+ at column \d+:\s*/i, '')
    .replace(/^XML Parsing Error:\s*/i, '')
    .trim();
}

export function parseCssIntoRules(
  css: string,
  rules: { selectors: string[]; declarations: Record<string, string>; span: any }[]
) {
  const parts = css.split('}');
  for (const part of parts) {
    const clean = part.trim();
    if (!clean) continue;
    const [selStr, declsStr] = clean.split('{');
    if (selStr && declsStr) {
      const selectors = selStr.split(',').map(s => s.trim()).filter(Boolean);
      const declarations: Record<string, string> = {};
      for (const decl of declsStr.split(';')) {
        const [prop, val] = decl.split(':');
        if (prop && val) {
          declarations[prop.trim()] = val.trim();
        }
      }
      rules.push({ selectors, declarations, span: { start: 0, end: 0, line: 1, col: 1 } });
    }
  }
}

export function parseChildNodes(parent: globalThis.Element | null, parentLine: number = 1): DocNode[] {
  if (!parent) return [];
  const nodes: DocNode[] = [];

  for (let i = 0; i < parent.childNodes.length; i++) {
    const child = parent.childNodes[i];

    if (child.nodeType === Node.TEXT_NODE) {
      const text = child.textContent || '';
      if (text.trim().length > 0) {
        nodes.push({
          type: 'Text',
          content: text,
          span: { start: 0, end: 0, line: parentLine, col: 1 },
        });
      }
    } else if (child.nodeType === Node.ELEMENT_NODE) {
      const el = child as globalThis.Element;
      const tag = el.tagName.toLowerCase();

      const attributes: Record<string, string> = {};
      const inline_styles: Record<string, string> = {};
      const classes: string[] = [];

      for (let a = 0; a < el.attributes.length; a++) {
        const attr = el.attributes[a];
        if (attr.name === 'class') {
          classes.push(...attr.value.split(/\s+/).filter(Boolean));
        } else if (attr.name === 'style') {
          attr.value.split(';').forEach(pair => {
            const [p, v] = pair.split(':');
            if (p && v) inline_styles[p.trim()] = v.trim();
          });
        } else {
          attributes[attr.name] = attr.value;
        }
      }

      const lineAttr = el.getAttribute('data-ddl-line');
      const nodeLine = lineAttr ? parseInt(lineAttr, 10) : parentLine;
      attributes['data-ddl-line'] = String(nodeLine);

      if (tag === 'section') {
        nodes.push({
          type: 'Section',
          page_style: attributes['page-style'],
          page_break: attributes['page-break'] === 'true',
          attributes,
          children: parseChildNodes(el, nodeLine),
          span: { start: 0, end: 0, line: nodeLine, col: 1 },
        });
      } else {
        nodes.push({
          type: 'Element',
          tag,
          attributes,
          id: attributes['id'],
          classes,
          inline_styles,
          children: parseChildNodes(el, nodeLine),
          span: { start: 0, end: 0, line: nodeLine, col: 1 },
        });
      }
    }
  }

  return nodes;
}
