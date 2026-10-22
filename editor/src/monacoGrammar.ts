import type { Monaco } from '@monaco-editor/react';

// Tag dictionary with grammar documentation & snippet templates
export const DDL_GRAMMAR = [
  // Document Structure
  {
    label: 'doc',
    desc: 'Root container for the entire DDL document.',
    snippet: '<doc>\n\t<props>\n\t\t<paper size="A4" orientation="portrait" margin="20mm" />\n\t</props>\n\t<body>\n\t\t<section page-style="body">\n\t\t\t$0\n\t\t</section>\n\t</body>\n</doc>',
  },
  {
    label: 'props',
    desc: 'Global document configuration block containing <meta>, <paper>, <style>, and <page-styles>.',
    snippet: '<props>\n\t<paper size="A4" orientation="portrait" margin="20mm" />\n\t<style>\n\t\t$0\n\t</style>\n</props>',
  },
  {
    label: 'meta',
    desc: 'Document metadata containing <title>, <author>, <date>.',
    snippet: '<meta>\n\t<title>${1:Document Title}</title>\n\t<author>${2:Author Name}</author>\n</meta>',
  },
  {
    label: 'paper',
    desc: 'Paper size and orientation (e.g. size="A4|Letter", orientation="portrait|landscape", margin="20mm").',
    snippet: '<paper size="${1|A4,Letter,A3,A5|}" orientation="${2|portrait,landscape|}" margin="${3:20mm}" />',
    isSelfClosing: true,
  },
  {
    label: 'style',
    desc: 'CSS stylesheet block scoped to the document.',
    snippet: '<style>\n\th1 { color: #1e3a8a; }\n\tp { line-height: 1.5; }\n\t$0\n</style>',
  },
  {
    label: 'page-styles',
    desc: 'Container for named page styles/templates.',
    snippet: '<page-styles>\n\t<page-style name="${1:body}" default="true">\n\t\t<running-header>\n\t\t\t<left><text>${2:Header Text}</text></left>\n\t\t\t<right><page-number/> of <total-pages/></right>\n\t\t</running-header>\n\t\t<running-footer>\n\t\t\t<center><text>${3:© 2026}</text></center>\n\t\t</running-footer>\n\t</page-style>\n</page-styles>',
  },
  {
    label: 'page-style',
    desc: 'Defines running headers, footers, margins and watermarks for a named page template.',
    snippet: '<page-style name="${1:cover}">\n\t<watermark text="${2:CONFIDENTIAL}" opacity="0.08" angle="-45" />\n\t$0\n</page-style>',
  },
  {
    label: 'running-header',
    desc: 'Running header template containing <left>, <center>, and <right> zones.',
    snippet: '<running-header>\n\t<left><text>${1:Title}</text></left>\n\t<right><page-number/> of <total-pages/></right>\n</running-header>',
  },
  {
    label: 'running-footer',
    desc: 'Running footer template containing <left>, <center>, and <right> zones.',
    snippet: '<running-footer>\n\t<center><text>${1:© 2026 Company}</text></center>\n</running-footer>',
  },
  {
    label: 'watermark',
    desc: 'Semi-transparent page background watermark.',
    snippet: '<watermark text="${1:DRAFT}" opacity="${2:0.08}" angle="${3:-45}" />',
    isSelfClosing: true,
  },
  {
    label: 'page-number',
    desc: 'Dynamic page number field.',
    snippet: '<page-number />',
    isSelfClosing: true,
  },
  {
    label: 'total-pages',
    desc: 'Dynamic total pages field.',
    snippet: '<total-pages />',
    isSelfClosing: true,
  },

  // Document Content Blocks
  {
    label: 'body',
    desc: 'Document body wrapper containing sections and content blocks.',
    snippet: '<body>\n\t$0\n</body>',
  },
  {
    label: 'section',
    desc: 'Document section/chapter applying a page-style and optional page break.',
    snippet: '<section page-style="${1:body}" page-break="${2|false,true|}">\n\t$0\n</section>',
  },
  {
    label: 'grid',
    desc: 'Multi-column layout powered by CSS Grid with configurable columns and gap.',
    snippet: '<grid cols="${1:1fr 1fr}" gap="${2:6mm}">\n\t<div>\n\t\t$0\n\t</div>\n\t<div>\n\t</div>\n</grid>',
  },
  {
    label: 'callout',
    desc: 'Admonition callout box for highlighting notes, tips, warnings or summaries.',
    snippet: '<callout>\n\t<b>${1:Note:}</b> ${2:Callout message}\n</callout>',
  },
  {
    label: 'p',
    desc: 'Paragraph block with automatic line height and text flow.',
    snippet: '<p>$0</p>',
  },
  {
    label: 'h1',
    desc: 'Heading 1 (Document / Section Title).',
    snippet: '<h1>$0</h1>',
  },
  {
    label: 'h2',
    desc: 'Heading 2 (Major Subheading).',
    snippet: '<h2>$0</h2>',
  },
  {
    label: 'h3',
    desc: 'Heading 3 (Minor Subheading).',
    snippet: '<h3>$0</h3>',
  },
  {
    label: 'ul',
    desc: 'Unordered bullet list container.',
    snippet: '<ul>\n\t<li>$0</li>\n</ul>',
  },
  {
    label: 'ol',
    desc: 'Ordered numbered list container.',
    snippet: '<ol>\n\t<li>$0</li>\n</ol>',
  },
  {
    label: 'li',
    desc: 'List item entry.',
    snippet: '<li>$0</li>',
  },
  {
    label: 'table',
    desc: 'Data table container.',
    snippet: '<table>\n\t<tr>\n\t\t<th>${1:Header}</th>\n\t</tr>\n\t<tr>\n\t\t<td>${2:Data}</td>\n\t</tr>\n</table>',
  },
  {
    label: 'tr',
    desc: 'Table row.',
    snippet: '<tr>\n\t$0\n</tr>',
  },
  {
    label: 'th',
    desc: 'Table header cell.',
    snippet: '<th>$0</th>',
  },
  {
    label: 'td',
    desc: 'Table data cell.',
    snippet: '<td>$0</td>',
  },
  {
    label: 'blockquote',
    desc: 'Styled quotation block with left border.',
    snippet: '<blockquote>\n\t$0\n</blockquote>',
  },
  {
    label: 'img',
    desc: 'Inline or block image with src, alt, and style attributes.',
    snippet: '<img src="${1:image.png}" alt="${2:Description}" style="${3:max-width: 100%; height: auto;}" />',
    isSelfClosing: true,
  },
  {
    label: 'b',
    desc: 'Bold inline text.',
    snippet: '<b>$0</b>',
  },
  {
    label: 'i',
    desc: 'Italic inline text.',
    snippet: '<i>$0</i>',
  },
  {
    label: 'u',
    desc: 'Underlined inline text.',
    snippet: '<u>$0</u>',
  },
  {
    label: 'code',
    desc: 'Inline code span with monospaced font styling.',
    snippet: '<code>$0</code>',
  },
  {
    label: 'page-break',
    desc: 'Force break onto a new physical paper sheet.',
    snippet: '<page-break />',
    isSelfClosing: true,
  },
];

export function isInsideStyleTag(text: string, offset: number): boolean {
  const openIdx = text.lastIndexOf('<style', offset);
  if (openIdx === -1) return false;
  const closeIdx = text.lastIndexOf('</style>', offset);
  return openIdx > closeIdx;
}

export function getCssGrammarSuggestions(monaco: Monaco, range: any) {
  const cssProps = [
    { prop: 'font-size', val: '${1:12pt};', desc: 'Font size in pt, px, or em' },
    { prop: 'font-weight', val: '${1|bold,600,500,normal|};', desc: 'Font weight thickness' },
    { prop: 'font-family', val: '"${1:Inter}", sans-serif;', desc: 'Font family fallback chain' },
    { prop: 'color', val: '${1:#1e3a8a};', desc: 'Text foreground color' },
    { prop: 'background', val: '${1:#f0fdf4};', desc: 'Background color or gradient' },
    { prop: 'margin-bottom', val: '${1:6mm};', desc: 'Bottom outer margin' },
    { prop: 'margin-top', val: '${1:6mm};', desc: 'Top outer margin' },
    { prop: 'padding', val: '${1:4mm 6mm};', desc: 'Inner padding clearance' },
    { prop: 'border-left', val: '${1:4pt solid #16a34a};', desc: 'Left accent border line' },
    { prop: 'border-radius', val: '${1:4px};', desc: 'Corner rounding radius' },
    { prop: 'line-height', val: '${1:1.5};', desc: 'Line height spacing multiplier' },
    { prop: 'text-align', val: '${1|left,center,right,justify|};', desc: 'Paragraph text alignment' },
    { prop: 'display', val: '${1|grid,block,inline-block|};', desc: 'Box display layout model' },
    { prop: 'grid-template-columns', val: '${1:1fr 1fr};', desc: 'Grid tracks specification' },
    { prop: 'gap', val: '${1:6mm};', desc: 'Grid gutter gap' },
  ];

  return cssProps.map(item => ({
    label: item.prop,
    kind: monaco.languages.CompletionItemKind.Property,
    insertText: `${item.prop}: ${item.val}`,
    insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
    documentation: `CSS Property: ${item.desc}`,
    range,
  }));
}
