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
    desc: 'Heading 2 (Chapter / Subsection Title).',
    snippet: '<h2>$0</h2>',
  },
  {
    label: 'h3',
    desc: 'Heading 3 (Minor Topic Heading).',
    snippet: '<h3>$0</h3>',
  },
  {
    label: 'div',
    desc: 'Generic block container for custom CSS styling.',
    snippet: '<div class="${1:box}">\n\t$0\n</div>',
  },
  {
    label: 'table',
    desc: 'Tabular data structure with thead, tbody, tr, th, and td.',
    snippet: '<table style="width: 100%; margin-top: 4mm;">\n\t<thead>\n\t\t<tr>\n\t\t\t<th>${1:Col 1}</th>\n\t\t\t<th>${2:Col 2}</th>\n\t\t</tr>\n\t</thead>\n\t<tbody>\n\t\t<tr>\n\t\t\t<td>${3:Data}</td>\n\t\t\t<td>${4:Data}</td>\n\t\t</tr>\n\t</tbody>\n</table>',
  },
  {
    label: 'ul',
    desc: 'Unordered bulleted list.',
    snippet: '<ul>\n\t<li>${1:Item 1}</li>\n\t<li>${2:Item 2}</li>\n</ul>',
  },
  {
    label: 'ol',
    desc: 'Ordered numbered list.',
    snippet: '<ol>\n\t<li>${1:Step 1}</li>\n\t<li>${2:Step 2}</li>\n</ol>',
  },
  {
    label: 'li',
    desc: 'List item.',
    snippet: '<li>$0</li>',
  },

  // Inlines
  { label: 'b', desc: 'Bold text.', snippet: '<b>$0</b>' },
  { label: 'i', desc: 'Italic text.', snippet: '<i>$0</i>' },
  { label: 'u', desc: 'Underlined text.', snippet: '<u>$0</u>' },
  { label: 's', desc: 'Strikethrough text.', snippet: '<s>$0</s>' },
  { label: 'code', desc: 'Monospace inline code span.', snippet: '<code>$0</code>' },
  { label: 'span', desc: 'Generic inline span with class or style.', snippet: '<span class="${1:highlight}">$0</span>' },
  { label: 'hr', desc: 'Horizontal rule line separator.', snippet: '<hr />', isSelfClosing: true },
];

export function registerMonacoDdlFeatures(monaco: Monaco) {
  // 1. Completion Provider for tags, snippets and grammar suggestions
  monaco.languages.registerCompletionItemProvider('ddl', {
    triggerCharacters: ['<', ' ', ':'],
    provideCompletionItems: (model: any, position: any) => {
      const textUntilPosition = model.getValueInRange({
        startLineNumber: position.lineNumber,
        startColumn: 1,
        endLineNumber: position.lineNumber,
        endColumn: position.column,
      });

      const word = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };

      // Check if user is inside a <style> ... </style> block to offer CSS property grammar suggestions
      const fullText = model.getValue();
      const currentOffset = model.getOffsetAt(position);
      if (isInsideStyleTag(fullText, currentOffset)) {
        return { suggestions: getCssGrammarSuggestions(monaco, range) };
      }

      // Tag & Snippet Grammar Suggestions
      if (textUntilPosition.endsWith('<') || /<[a-zA-Z0-9_-]*$/.test(textUntilPosition)) {
        const suggestions = DDL_GRAMMAR.map(item => {
          return {
            label: item.label,
            kind: monaco.languages.CompletionItemKind.Snippet,
            insertText: item.snippet,
            insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
            documentation: {
              value: `**DDL Grammar: \`<${item.label}>\`**\n\n${item.desc}\n\n*Press Tab or Enter to insert snippet.*`,
            },
            detail: `DDL Element: <${item.label}>`,
            range,
          };
        });

        return { suggestions };
      }

      // Contextual Attribute Grammar Suggestions
      const tagMatch = textUntilPosition.match(/<([a-zA-Z0-9_-]+)[^>]*\s+([a-zA-Z0-9_-]*)$/);
      if (tagMatch) {
        const currentTag = tagMatch[1].toLowerCase();
        const suggestions: any[] = [];

        if (currentTag === 'grid') {
          suggestions.push(
            { label: 'cols', kind: monaco.languages.CompletionItemKind.Property, insertText: 'cols="${1:1fr 1fr}"', insertTextRules: 4, detail: 'Grid Column Tracks (e.g. 1fr 1fr, 2fr 1fr)', range },
            { label: 'gap', kind: monaco.languages.CompletionItemKind.Property, insertText: 'gap="${1:6mm}"', insertTextRules: 4, detail: 'Grid Gutter Spacing (e.g. 6mm, 12px)', range },
          );
        } else if (currentTag === 'paper') {
          suggestions.push(
            { label: 'size', kind: monaco.languages.CompletionItemKind.Property, insertText: 'size="${1|A4,Letter,A3,A5|}"', insertTextRules: 4, detail: 'Paper Standard (A4, Letter, etc.)', range },
            { label: 'orientation', kind: monaco.languages.CompletionItemKind.Property, insertText: 'orientation="${1|portrait,landscape|}"', insertTextRules: 4, detail: 'Page Orientation', range },
            { label: 'margin', kind: monaco.languages.CompletionItemKind.Property, insertText: 'margin="${1:20mm}"', insertTextRules: 4, detail: 'Page Margin Clearance (e.g. 20mm, 25mm)', range },
          );
        } else if (currentTag === 'section') {
          suggestions.push(
            { label: 'page-style', kind: monaco.languages.CompletionItemKind.Property, insertText: 'page-style="${1:body}"', insertTextRules: 4, detail: 'Apply a named page-style', range },
            { label: 'page-break', kind: monaco.languages.CompletionItemKind.Property, insertText: 'page-break="${1|true,false|}"', insertTextRules: 4, detail: 'Force new page before section', range },
          );
        } else if (currentTag === 'watermark') {
          suggestions.push(
            { label: 'text', kind: monaco.languages.CompletionItemKind.Property, insertText: 'text="${1:CONFIDENTIAL}"', insertTextRules: 4, detail: 'Watermark Display Text', range },
            { label: 'opacity', kind: monaco.languages.CompletionItemKind.Property, insertText: 'opacity="${1:0.08}"', insertTextRules: 4, detail: 'Watermark Opacity (0.0 to 1.0)', range },
            { label: 'angle', kind: monaco.languages.CompletionItemKind.Property, insertText: 'angle="${1:-45}"', insertTextRules: 4, detail: 'Rotation angle in degrees', range },
          );
        }

        // Global Attributes
        suggestions.push(
          { label: 'class', kind: monaco.languages.CompletionItemKind.Property, insertText: 'class="$1"', insertTextRules: 4, detail: 'CSS Class Selector', range },
          { label: 'id', kind: monaco.languages.CompletionItemKind.Property, insertText: 'id="$1"', insertTextRules: 4, detail: 'Unique Element ID', range },
          { label: 'style', kind: monaco.languages.CompletionItemKind.Property, insertText: 'style="$1"', insertTextRules: 4, detail: 'Inline CSS Declarations', range },
        );

        return { suggestions };
      }

      return { suggestions: [] };
    },
  });

  // 2. Hover Provider for Grammar Documentation
  monaco.languages.registerHoverProvider('ddl', {
    provideHover: (model: any, position: any) => {
      const word = model.getWordAtPosition(position);
      if (!word) return null;

      const tagMatch = DDL_GRAMMAR.find(t => t.label.toLowerCase() === word.word.toLowerCase());
      if (tagMatch) {
        return {
          range: new monaco.Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn),
          contents: [
            { value: `**DDL Tag: \`<${tagMatch.label}>\`**` },
            { value: tagMatch.desc },
            { value: `\`\`\`xml\n${tagMatch.snippet.replace(/\$\{\d+:?([^}]*)\}/g, '$1').replace(/\$0/g, '')}\n\`\`\`` },
          ],
        };
      }

      return null;
    },
  });

  // 3. Linked Editing Range (Simultaneous dual-tag renaming)
  monaco.languages.registerLinkedEditingRangeProvider('ddl', {
    provideLinkedEditingRanges: (model: any, position: any) => {
      const lineContent = model.getLineContent(position.lineNumber);
      const col = position.column;

      const openTagMatch = lineContent.slice(0, col).match(/<([a-zA-Z0-9_-]+)[^>]*$/);
      const closeTagMatch = lineContent.slice(0, col).match(/<\/([a-zA-Z0-9_-]+)$/);

      if (!openTagMatch && !closeTagMatch) {
        return null;
      }

      const fullText = model.getValue();
      const currentPos = model.getOffsetAt(position);
      const ranges = findMatchingTagRanges(fullText, currentPos, model);
      if (!ranges) return null;

      return {
        ranges,
        wordPattern: /[a-zA-Z0-9_-]+/,
      };
    },
  });
}

function isInsideStyleTag(text: string, offset: number): boolean {
  const openIdx = text.lastIndexOf('<style', offset);
  if (openIdx === -1) return false;
  const closeIdx = text.lastIndexOf('</style>', offset);
  return openIdx > closeIdx;
}

function getCssGrammarSuggestions(monaco: Monaco, range: any) {
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

function findMatchingTagRanges(text: string, offset: number, model: any) {
  const tagRegex = /<\/?([a-zA-Z0-9_-]+)(?:\s+[^>]*?)?(\/?)>/g;
  let match;
  const stack: { tag: string; start: number; end: number; nameStart: number; nameEnd: number }[] = [];
  const pairs: { openName: [number, number]; closeName: [number, number] }[] = [];

  while ((match = tagRegex.exec(text)) !== null) {
    const fullMatch = match[0];
    const tagName = match[1];
    const isSelfClosing = match[2] === '/' || fullMatch.endsWith('/>');
    const isCloseTag = fullMatch.startsWith('</');
    const matchStart = match.index;
    const matchEnd = match.index + fullMatch.length;

    if (isSelfClosing) continue;

    if (!isCloseTag) {
      const nameStart = matchStart + 1;
      const nameEnd = nameStart + tagName.length;
      stack.push({ tag: tagName, start: matchStart, end: matchEnd, nameStart, nameEnd });
    } else {
      const nameStart = matchStart + 2;
      const nameEnd = nameStart + tagName.length;

      for (let i = stack.length - 1; i >= 0; i--) {
        if (stack[i].tag === tagName) {
          const open = stack.splice(i, 1)[0];
          pairs.push({
            openName: [open.nameStart, open.nameEnd],
            closeName: [nameStart, nameEnd],
          });
          break;
        }
      }
    }
  }

  for (const pair of pairs) {
    const [openStart, openEnd] = pair.openName;
    const [closeStart, closeEnd] = pair.closeName;

    if ((offset >= openStart && offset <= openEnd) || (offset >= closeStart && offset <= closeEnd)) {
      const openPosStart = model.getPositionAt(openStart);
      const openPosEnd = model.getPositionAt(openEnd);
      const closePosStart = model.getPositionAt(closeStart);
      const closePosEnd = model.getPositionAt(closeEnd);

      return [
        {
          startLineNumber: openPosStart.lineNumber,
          startColumn: openPosStart.column,
          endLineNumber: openPosEnd.lineNumber,
          endColumn: openPosEnd.column,
        },
        {
          startLineNumber: closePosStart.lineNumber,
          startColumn: closePosStart.column,
          endLineNumber: closePosEnd.lineNumber,
          endColumn: closePosEnd.column,
        },
      ];
    }
  }

  return null;
}
