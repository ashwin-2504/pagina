import type { ParsedDocument, PageStyle } from './types';
import {
  annotateDxmlWithLineNumbers,
  cleanErrorMessage,
  parseCssIntoRules,
  parseChildNodes,
} from './parseAst';

export { annotateDxmlWithLineNumbers };

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
