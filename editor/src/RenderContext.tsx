import React from 'react';
import type { DocNode } from './types';
import { RenderElement } from './RenderElement';

export const AssetContext = React.createContext<Record<string, string>>({});

export const PageContext = React.createContext<{ pageNum: number; totalPages: number }>({
  pageNum: 1,
  totalPages: 1,
});

export const RenderChildren: React.FC<{ nodes: DocNode[] }> = ({ nodes }) => {
  if (!nodes || nodes.length === 0) return null;
  return (
    <>
      {nodes.map((node, i) => (
        <RenderNode key={i} node={node} />
      ))}
    </>
  );
};

export const RenderNode: React.FC<{ node: DocNode }> = ({ node }) => {
  if (node.type === 'Text') {
    return <>{node.content}</>;
  }

  if (node.type === 'Comment') {
    return null;
  }

  if (node.type === 'Section') {
    return <RenderChildren nodes={node.children} />;
  }

  if (node.type === 'Element') {
    return <RenderElement el={node} />;
  }

  return null;
};
