import type { ParsedDocument, DocNode, PageStyle } from './types';

export interface ParseResult {
  doc: ParsedDocument | null;
  error: {
    message: string;
    line?: number;
    column?: number;
    snippet?: string;
  } | null;
}

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

export function parseDdlClient(input: string, externalStylesheets?: Record<string, string>): ParseResult {
  const annotatedInput = annotateDxmlWithLineNumbers(input);
  const parser = new DOMParser();
  let xmlDoc = parser.parseFromString(annotatedInput, 'text/xml');

  let parserError = xmlDoc.querySelector('parsererror');
  if (parserError) {
    // Try fallback on original input in case annotation encountered an unusual token
    const fallbackDoc = parser.parseFromString(input, 'text/xml');
    if (!fallbackDoc.querySelector('parsererror')) {
      xmlDoc = fallbackDoc;
      parserError = null;
    }
  }

  if (parserError) {
    const errorText = parserError.textContent || 'XML Syntax Error';
    const lineMatch = errorText.match(/line\s+(\d+)/i);
    const colMatch = errorText.match(/column\s+(\d+)/i);
    const line = lineMatch ? parseInt(lineMatch[1], 10) : undefined;
    const column = colMatch ? parseInt(colMatch[1], 10) : undefined;

    let snippet: string | undefined;
    if (line !== undefined) {
      const lines = input.split('\n');
      const start = Math.max(0, line - 3);
      const end = Math.min(lines.length, line + 2);
      snippet = lines.slice(start, end)
        .map((l, i) => `${start + i + 1 === line ? ' > ' : '   '}${start + i + 1} | ${l}`)
        .join('\n');
    }

    return {
      doc: null,
      error: {
        message: cleanErrorMessage(errorText),
        line,
        column,
        snippet,
      },
    };
  }

  try {
    const docEl = xmlDoc.querySelector('doc') || xmlDoc.documentElement;
    const propsEl = docEl.querySelector('props');
    const bodyEl = docEl.querySelector('body') || docEl;

    // 1. Parse Paper
    const paperEl = propsEl?.querySelector('paper');
    const paperLine = paperEl?.getAttribute('data-ddl-line') ? parseInt(paperEl.getAttribute('data-ddl-line')!, 10) : undefined;
    const size = paperEl?.getAttribute('size') || 'A4';
    const orientation = paperEl?.getAttribute('orientation') || 'portrait';
    const marginStr = paperEl?.getAttribute('margin') || '20mm';
    const headerDistanceStr = paperEl?.getAttribute('header-distance');
    const footerDistanceStr = paperEl?.getAttribute('footer-distance');
    const marginVal = parseFloat(marginStr) || 20;
    const parsePaperLength = (value?: string | null) => {
      if (!value) return undefined;
      const val = parseFloat(value);
      return Number.isFinite(val) ? { val, unit: 'Mm' as const } : undefined;
    };

    // 2. Parse Stylesheets (Both inline <style> AND linked <link rel="stylesheet" href="..." />)
    const rules: { selectors: string[]; declarations: Record<string, string>; span: any }[] = [];

    // A. Parse linked external stylesheets
    if (propsEl) {
      const linkEls = propsEl.querySelectorAll('link[rel="stylesheet"], link[href]');
      linkEls.forEach(link => {
        const href = link.getAttribute('href');
        if (href && externalStylesheets && externalStylesheets[href]) {
          const cssContent = externalStylesheets[href];
          parseCssIntoRules(cssContent, rules);
        }
      });
    }

    // B. Parse inline <style> tags
    const styleEl = propsEl?.querySelector('style');
    if (styleEl && styleEl.textContent) {
      parseCssIntoRules(styleEl.textContent, rules);
    }

    // Also check if any standalone external style should be loaded by default if no links specified
    if (rules.length === 0 && externalStylesheets) {
      for (const [filename, css] of Object.entries(externalStylesheets)) {
        if (filename.endsWith('.css') || filename.endsWith('.dcss')) {
          parseCssIntoRules(css, rules);
        }
      }
    }

    // 3. Parse Page Styles
    const pageStyles: Record<string, PageStyle> = {};
    const pageStylesEl = propsEl?.querySelector('page-styles');
    if (pageStylesEl) {
      const styleEls = pageStylesEl.querySelectorAll('page-style');
      styleEls.forEach((p) => {
        const name = p.getAttribute('name') || 'default';
        const isDefault = p.getAttribute('default') === 'true';
        const pageStyleLine = p.getAttribute('data-ddl-line') ? parseInt(p.getAttribute('data-ddl-line')!, 10) : undefined;

        const headerEl = p.querySelector('running-header');
        const footerEl = p.querySelector('running-footer');
        const watermarkEl = p.querySelector('watermark');

        const headerLine = headerEl?.getAttribute('data-ddl-line') ? parseInt(headerEl.getAttribute('data-ddl-line')!, 10) : undefined;
        const footerLine = footerEl?.getAttribute('data-ddl-line') ? parseInt(footerEl.getAttribute('data-ddl-line')!, 10) : undefined;
        const watermarkLine = watermarkEl?.getAttribute('data-ddl-line') ? parseInt(watermarkEl.getAttribute('data-ddl-line')!, 10) : undefined;

        let headerLeft = parseChildNodes(headerEl?.querySelector('left') || null, headerLine || 1);
        let headerCenter = parseChildNodes(headerEl?.querySelector('center') || null, headerLine || 1);
        let headerRight = parseChildNodes(headerEl?.querySelector('right') || null, headerLine || 1);
        if (headerEl && headerLeft.length === 0 && headerCenter.length === 0 && headerRight.length === 0) {
          headerCenter = parseChildNodes(headerEl, headerLine || 1);
        }

        let footerLeft = parseChildNodes(footerEl?.querySelector('left') || null, footerLine || 1);
        let footerCenter = parseChildNodes(footerEl?.querySelector('center') || null, footerLine || 1);
        let footerRight = parseChildNodes(footerEl?.querySelector('right') || null, footerLine || 1);
        if (footerEl && footerLeft.length === 0 && footerCenter.length === 0 && footerRight.length === 0) {
          footerCenter = parseChildNodes(footerEl, footerLine || 1);
        }

        pageStyles[name] = {
          name,
          is_default: isDefault,
          line: pageStyleLine,
          running_header: headerEl ? {
            left: headerLeft,
            center: headerCenter,
            right: headerRight,
            line: headerLine,
          } : undefined,
          running_footer: footerEl ? {
            left: footerLeft,
            center: footerCenter,
            right: footerRight,
            line: footerLine,
          } : undefined,
          watermark: watermarkEl ? {
            text: watermarkEl.getAttribute('text') || '',
            opacity: parseFloat(watermarkEl.getAttribute('opacity') || '0.08'),
            angle_deg: parseFloat(watermarkEl.getAttribute('angle') || '-45'),
            line: watermarkLine,
          } : undefined,
        };
      });
    }

    if (!pageStyles['body'] && !pageStyles['default']) {
      pageStyles['default'] = {
        name: 'default',
        is_default: true,
      };
    }

    // 4. Parse Body Elements
    const bodyLine = bodyEl?.getAttribute('data-ddl-line') ? parseInt(bodyEl.getAttribute('data-ddl-line')!, 10) : 1;
    const bodyNodes = parseChildNodes(bodyEl, bodyLine);

    return {
      doc: {
        props: {
          metadata: {},
          paper: {
            size,
            orientation,
            margin: { val: marginVal, unit: 'Mm' },
            header_distance: parsePaperLength(headerDistanceStr),
            footer_distance: parsePaperLength(footerDistanceStr),
          },
          stylesheet: { rules },
          page_styles: pageStyles,
          default_page_style: Object.keys(pageStyles).find(k => pageStyles[k].is_default) || Object.keys(pageStyles)[0] || 'default',
          paper_line: paperLine,
        },
        body: {
          attributes: {},
          children: bodyNodes,
          span: { start: 0, end: input.length, line: bodyLine, col: 1 },
        },
        span: { start: 0, end: input.length, line: 1, col: 1 },
      },
      error: null,
    };
  } catch (err: any) {
    return {
      doc: null,
      error: {
        message: err?.message || 'Unexpected parsing error',
      },
    };
  }
}

function cleanErrorMessage(raw: string): string {
  // Strip Chrome/Firefox boilerplates
  return raw
    .replace(/^error on line \d+ at column \d+:\s*/i, '')
    .replace(/^XML Parsing Error:\s*/i, '')
    .trim();
}

function parseCssIntoRules(css: string, rules: { selectors: string[]; declarations: Record<string, string>; span: any }[]) {
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

function parseChildNodes(parent: globalThis.Element | null, parentLine: number = 1): DocNode[] {
  if (!parent) return [];
  const nodes: DocNode[] = [];

  for (let i = 0; i < parent.childNodes.length; i++) {
    const child = parent.childNodes[i];

    if (child.nodeType === Node.TEXT_NODE) {
      const text = child.textContent || '';
      // Only keep text if it has non-whitespace characters
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

      // Collect attributes
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
