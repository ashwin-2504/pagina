import { useMemo, useState, useLayoutEffect, type RefObject } from 'react';
import type { ParsedDocument, DocNode, ElementNode, PageStyle } from './types';
import { splitLargeBlocks, estimateNodeHeight, hasImageNode } from './renderUtils';

export interface RenderedPage {
  pageStyleName: string;
  pageNum: number;
  nodes: DocNode[];
}

export function useDocumentCss(doc: ParsedDocument | null): string {
  return useMemo(() => {
    if (!doc?.props?.stylesheet?.rules) return '';
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
  }, [doc?.props?.stylesheet]);
}

interface PaginationOptions {
  doc: ParsedDocument;
  heightMm: number;
  widthMm: number;
  marginMm: number;
  headerDistanceMm: number;
  footerDistanceMm: number;
  customCss: string;
  measureContainerRef: RefObject<HTMLDivElement | null>;
}

export function usePagination({
  doc,
  heightMm,
  widthMm,
  marginMm,
  headerDistanceMm,
  footerDistanceMm,
  customCss,
  measureContainerRef,
}: PaginationOptions) {
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
  }, [sectionsWithSplitBlocks, widthMm, marginMm, customCss, measureContainerRef]);

  // Calculate discrete physical pages based on available height per sheet
  const pages = useMemo<RenderedPage[]>(() => {
    const result: RenderedPage[] = [];
    let globalPageNum = 1;

    for (let sIdx = 0; sIdx < sectionsWithSplitBlocks.length; sIdx++) {
      const sec = sectionsWithSplitBlocks[sIdx];
      const pageStyle: PageStyle | undefined = doc.props?.page_styles?.[sec.pageStyleName];

      const pxPerMm = 96 / 25.4;
      const pageHeightPx = heightMm * pxPerMm;
      const topMarginPx = marginMm * pxPerMm;
      const bottomMarginPx = marginMm * pxPerMm;

      let headerHeightPx = 0;
      if (pageStyle?.running_header) {
        const hasImg = hasImageNode(pageStyle.running_header.center) ||
                       hasImageNode(pageStyle.running_header.left) ||
                       hasImageNode(pageStyle.running_header.right);
        headerHeightPx = (hasImg ? 38 : 14) * pxPerMm;
      }
      const headerGapPx = headerHeightPx > 0 ? (headerHeightPx > 20 * pxPerMm ? 3 : 8) * pxPerMm : 0;

      let footerHeightPx = 0;
      if (pageStyle?.running_footer) {
        const hasImg = hasImageNode(pageStyle.running_footer.center) ||
                       hasImageNode(pageStyle.running_footer.left) ||
                       hasImageNode(pageStyle.running_footer.right);
        footerHeightPx = (hasImg ? 28 : 14) * pxPerMm;
      }
      const footerGapPx = footerHeightPx > 0 ? 3 * pxPerMm : 0;

      const reservedTopPx = Math.max(topMarginPx, headerDistanceMm * pxPerMm + headerHeightPx + headerGapPx);
      const reservedBottomPx = Math.max(bottomMarginPx, footerDistanceMm * pxPerMm + footerHeightPx + footerGapPx);
      const availableHeightPx = Math.max(
        200,
        pageHeightPx - reservedTopPx - reservedBottomPx,
      );

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
  }, [sectionsWithSplitBlocks, measuredHeights, heightMm, widthMm, marginMm, headerDistanceMm, footerDistanceMm, doc.props?.page_styles]);

  return { pages, sectionsWithSplitBlocks };
}
