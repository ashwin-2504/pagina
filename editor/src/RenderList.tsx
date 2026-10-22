import React from 'react';
import type { ElementNode } from './types';
import { RenderChildren } from './RenderContext';

export function renderListElement(
  tagLower: string,
  el: ElementNode,
  className?: string,
  styleObj: React.CSSProperties = {},
): React.ReactNode {
  const { attributes, inline_styles, children } = el;

  if (tagLower === 'ul') {
    const rawType = attributes['type'] || inline_styles?.['list-style-type'];
    let cssListType: string | undefined;
    if (rawType) {
      const lower = rawType.toLowerCase();
      if (lower === 'circle') cssListType = 'circle';
      else if (lower === 'square') cssListType = 'square';
      else if (lower === 'dash') cssListType = '"- "';
      else if (lower === 'none') cssListType = 'none';
      else if (lower === 'disc') cssListType = 'disc';
    }

    return (
      <ul
        className={className}
        data-type={rawType}
        style={{
          listStyleType: cssListType,
          listStylePosition: 'outside',
          textAlign: 'left',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </ul>
    );
  }

  if (tagLower === 'ol') {
    const rawType = attributes['type'] || inline_styles?.['list-style-type'];
    let cssListType: string | undefined;
    let htmlType: '1' | 'a' | 'A' | 'i' | 'I' | undefined;

    if (rawType) {
      const lower = rawType.toLowerCase();
      if (rawType === 'A' || lower === 'upper-alpha') {
        cssListType = 'upper-alpha';
        htmlType = 'A';
      } else if (rawType === 'I' || lower === 'upper-roman') {
        cssListType = 'upper-roman';
        htmlType = 'I';
      } else if (rawType === 'a' || lower === 'lower-alpha') {
        cssListType = 'lower-alpha';
        htmlType = 'a';
      } else if (rawType === 'i' || lower === 'lower-roman') {
        cssListType = 'lower-roman';
        htmlType = 'i';
      } else if (rawType === '1' || lower === 'decimal') {
        cssListType = 'decimal';
        htmlType = '1';
      } else if (lower === 'none') {
        cssListType = 'none';
      }
    }

    const startAttr = attributes['start'] ? parseInt(attributes['start'], 10) : undefined;

    return (
      <ol
        className={className}
        start={startAttr}
        type={htmlType}
        data-type={rawType}
        style={{
          listStyleType: cssListType,
          listStylePosition: 'outside',
          textAlign: 'left',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </ol>
    );
  }

  if (tagLower === 'li') {
    return (
      <li
        className={className}
        style={{
          marginBottom: '1.5mm',
          lineHeight: 1.6,
          textAlign: 'left',
          ...styleObj,
        }}
        {...attributes}
      >
        <RenderChildren nodes={children} />
      </li>
    );
  }

  return null;
}
