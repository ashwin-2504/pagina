import type { Monaco } from '@monaco-editor/react';
import {
  DDL_GRAMMAR,
  isInsideStyleTag,
  getCssGrammarSuggestions,
} from './monacoGrammar';

export { DDL_GRAMMAR };

export function registerMonacoDdlFeatures(monaco: Monaco) {
  // 1. Completion Provider for Tags, Attributes, and CSS Properties
  monaco.languages.registerCompletionItemProvider('ddl', {
    triggerCharacters: ['<', ' ', '"', ':', '.', '/', '{'],
    provideCompletionItems: (model: any, position: any) => {
      const textUntilPosition = model.getValueInRange({
        startLineNumber: 1,
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

      const fullText = model.getValue();
      const offset = model.getOffsetAt(position);

      // Context A: Inside <style> tag -> Offer CSS completions
      if (isInsideStyleTag(fullText, offset)) {
        return {
          suggestions: getCssGrammarSuggestions(monaco, range),
        };
      }

      // Context B: Opening a new tag e.g. `<` or typing inside a tag `<p`
      const isTagStart = textUntilPosition.match(/<([a-zA-Z0-9_-]*)$/);
      if (isTagStart) {
        const suggestions = DDL_GRAMMAR.map(item => ({
          label: item.label,
          kind: monaco.languages.CompletionItemKind.Class,
          insertText: item.snippet,
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          documentation: item.desc,
          range,
        }));
        return { suggestions };
      }

      // Context C: Inside an open tag typing attributes e.g. `<section `
      const isInsideTag = textUntilPosition.match(/<([a-zA-Z0-9_-]+)(?:\s+[^>]*?)?$/);
      if (isInsideTag) {
        const currentTag = isInsideTag[1].toLowerCase();
        const suggestions: any[] = [];

        // Tag-specific attributes
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
