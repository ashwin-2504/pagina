import type { Monaco } from '@monaco-editor/react';

export const DDL_LANGUAGE_ID = 'ddl';

export function setupDdlLanguageTokenizer(monaco: Monaco) {
  // Register custom 'ddl' language
  monaco.languages.register({ id: DDL_LANGUAGE_ID });

  // Custom Monarch syntax highlighter with embedded CSS tokenizer inside <style>...</style>
  monaco.languages.setMonarchTokensProvider(DDL_LANGUAGE_ID, {
    defaultToken: '',
    tokenPostfix: '.ddl',

    // Keywords and tags
    keywords: [
      'doc', 'props', 'meta', 'title', 'author', 'date',
      'paper', 'style', 'page-styles', 'page-style',
      'running-header', 'running-footer', 'watermark',
      'left', 'center', 'right', 'page-number', 'total-pages',
      'body', 'section', 'grid', 'callout', 'div', 'p',
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'table', 'thead', 'tbody', 'tr', 'th', 'td',
      'ul', 'ol', 'li', 'b', 'strong', 'i', 'em', 'u', 's',
      'sup', 'sub', 'code', 'span', 'hr'
    ],

    tokenizer: {
      root: [
        // Comments
        [/<!--/, 'comment', '@comment'],

        // Transition into embedded CSS inside <style>
        [/(<style\b[^>]*>)/, { token: 'tag.style', next: '@styleBlock' }],

        // XML tags
        [/<\/?[a-zA-Z0-9_-]+/, 'tag', '@tagContent'],

        // Text
        [/[^<]+/, ''],
      ],

      comment: [
        [/-->/, 'comment', '@pop'],
        [/./, 'comment'],
      ],

      tagContent: [
        [/>/, 'tag', '@pop'],
        [/\/>/, 'tag', '@pop'],
        // Attribute names
        [/([a-zA-Z0-9_-]+)(\s*)(=)(\s*)/, ['attribute.name', '', 'delimiter', '']],
        // Quoted attribute values
        [/"([^"]*)"/, 'attribute.value'],
        [/'([^']*)'/, 'attribute.value'],
        [/\s+/, ''],
      ],

      // Embedded CSS Block inside <style>
      styleBlock: [
        // Exit CSS block on </style>
        [/<\/style>/, { token: 'tag.style', next: '@pop' }],

        // CSS Comments
        [/\/\*/, 'comment', '@cssComment'],

        // Selectors: tag, class, id
        [/([.#]?[a-zA-Z0-9_-]+)(\s*)(?=\{)/, ['type.identifier', '']],
        [/([.#]?[a-zA-Z0-9_-]+)/, 'type.identifier'],

        // Rule block
        [/\{/, 'delimiter.bracket', '@styleBody'],
        [/\s+/, ''],
      ],

      styleBody: [
        // Exit block on '}'
        [/\}/, 'delimiter.bracket', '@pop'],
        [/<\/style>/, { token: 'tag.style', next: '@pop' }],

        // CSS Comments
        [/\/\*/, 'comment', '@cssComment'],

        // CSS Properties
        [/([a-zA-Z0-9_-]+)(\s*)(:)/, ['key', '', 'delimiter']],

        // Values: colors (#hex, rgb), lengths (px, mm, pt), keywords, strings
        [/#[a-fA-F0-9]{3,8}/, 'number.hex'],
        [/[0-9]+(\.[0-9]+)?(mm|pt|px|cm|in|em|rem|%|fr)?/, 'number'],
        [/"([^"]*)"/, 'string'],
        [/'([^']*)'/, 'string'],
        [/([a-zA-Z_-]+)/, 'variable'],
        [/;/, 'delimiter'],
        [/\s+/, ''],
      ],

      cssComment: [
        [/\*\//, 'comment', '@pop'],
        [/./, 'comment'],
      ],
    },
  });

  // Define language configuration (auto-closing pairs, brackets, indentation rules)
  monaco.languages.setLanguageConfiguration(DDL_LANGUAGE_ID, {
    comments: {
      blockComment: ['<!--', '-->'],
    },
    brackets: [
      ['<', '>'],
      ['{', '}'],
      ['(', ')'],
      ['[', ']'],
    ],
    autoClosingPairs: [
      { open: '<', close: '>' },
      { open: '{', close: '}' },
      { open: '(', close: ')' },
      { open: '[', close: ']' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
    surroundingPairs: [
      { open: '<', close: '>' },
      { open: '{', close: '}' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
    ],
  });
}
