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

export function parseDdlClient(input: string, externalStylesheets?: Record<string, string>): ParseResult {
  const parser = new DOMParser();
  const xmlDoc = parser.parseFromString(input, 'text/xml');

  const parserError = xmlDoc.querySelector('parsererror');
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
    const size = paperEl?.getAttribute('size') || 'A4';
    const orientation = paperEl?.getAttribute('orientation') || 'portrait';
    const marginStr = paperEl?.getAttribute('margin') || '20mm';
    const marginVal = parseFloat(marginStr) || 20;

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
        if (filename.endsWith('.css')) {
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

        const headerEl = p.querySelector('running-header');
        const footerEl = p.querySelector('running-footer');
        const watermarkEl = p.querySelector('watermark');

        pageStyles[name] = {
          name,
          is_default: isDefault,
          running_header: headerEl ? {
            left: parseChildNodes(headerEl.querySelector('left')),
            center: parseChildNodes(headerEl.querySelector('center')),
            right: parseChildNodes(headerEl.querySelector('right')),
          } : undefined,
          running_footer: footerEl ? {
            left: parseChildNodes(footerEl.querySelector('left')),
            center: parseChildNodes(footerEl.querySelector('center')),
            right: parseChildNodes(footerEl.querySelector('right')),
          } : undefined,
          watermark: watermarkEl ? {
            text: watermarkEl.getAttribute('text') || '',
            opacity: parseFloat(watermarkEl.getAttribute('opacity') || '0.08'),
            angle_deg: parseFloat(watermarkEl.getAttribute('angle') || '-45'),
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
    const bodyNodes = parseChildNodes(bodyEl);

    return {
      doc: {
        props: {
          metadata: {},
          paper: {
            size,
            orientation,
            margin: { val: marginVal, unit: 'Mm' },
          },
          stylesheet: { rules },
          page_styles: pageStyles,
          default_page_style: Object.keys(pageStyles).find(k => pageStyles[k].is_default) || Object.keys(pageStyles)[0] || 'default',
        },
        body: {
          attributes: {},
          children: bodyNodes,
          span: { start: 0, end: input.length, line: 1, col: 1 },
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

function parseChildNodes(parent: globalThis.Element | null): DocNode[] {
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
          span: { start: 0, end: 0, line: 1, col: 1 },
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

      if (tag === 'section') {
        nodes.push({
          type: 'Section',
          page_style: attributes['page-style'],
          page_break: attributes['page-break'] === 'true',
          attributes,
          children: parseChildNodes(el),
          span: { start: 0, end: 0, line: 1, col: 1 },
        });
      } else {
        nodes.push({
          type: 'Element',
          tag,
          attributes,
          id: attributes['id'],
          classes,
          inline_styles,
          children: parseChildNodes(el),
          span: { start: 0, end: 0, line: 1, col: 1 },
        });
      }
    }
  }

  return nodes;
}
