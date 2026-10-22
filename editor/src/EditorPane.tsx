import React from 'react';
import Editor from '@monaco-editor/react';
import { Palette, Code2, Trash2, Plus } from 'lucide-react';
import type { ProjectFile } from './defaultFiles';

interface EditorPaneProps {
  files: ProjectFile[];
  activeFileId: string;
  activeFile: ProjectFile;
  onSelectFile: (id: string) => void;
  onDeleteFile: (e: React.MouseEvent, id: string) => void;
  onCreateFile: () => void;
  onEditorMount: (editor: any) => void;
  onEditorChange: (value: string | undefined) => void;
}

export const EditorPane: React.FC<EditorPaneProps> = ({
  files,
  activeFileId,
  activeFile,
  onSelectFile,
  onDeleteFile,
  onCreateFile,
  onEditorMount,
  onEditorChange,
}) => {
  return (
    <div className="editor-pane no-print" style={{
      width: '48%',
      borderRight: '1px solid #1e293b',
      display: 'flex',
      flexDirection: 'column',
      background: '#1e1e1e',
    }}>
      {/* File Tabs Header */}
      <div style={{
        height: '38px',
        background: '#18181b',
        borderBottom: '1px solid #27272a',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingRight: '8px',
        flexShrink: 0,
        overflowX: 'auto',
      }}>
        {/* Tabs List */}
        <div style={{ display: 'flex', alignItems: 'center', height: '100%' }}>
          {files.map(file => {
            const isActive = file.id === activeFileId;
            const isCss = file.language === 'css' || file.name.endsWith('.css') || file.name.endsWith('.dcss');
            return (
              <div
                key={file.id}
                onClick={() => onSelectFile(file.id)}
                style={{
                  height: '100%',
                  padding: '0 14px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  fontSize: '12px',
                  fontWeight: isActive ? 600 : 500,
                  color: isActive ? '#f8fafc' : '#71717a',
                  background: isActive ? '#1e1e1e' : 'transparent',
                  borderRight: '1px solid #27272a',
                  borderTop: isActive ? '2px solid #38bdf8' : '2px solid transparent',
                  cursor: 'pointer',
                  userSelect: 'none',
                }}
              >
                {isCss ? <Palette size={13} color="#a855f7" /> : <Code2 size={13} color="#38bdf8" />}
                <span>{file.name}</span>
                {files.length > 1 && (
                  <button
                    onClick={(e) => onDeleteFile(e, file.id)}
                    title="Delete file"
                    style={{
                      background: 'none',
                      border: 'none',
                      color: '#71717a',
                      padding: '2px',
                      display: 'flex',
                      alignItems: 'center',
                      cursor: 'pointer',
                      borderRadius: '3px',
                    }}
                  >
                    <Trash2 size={11} />
                  </button>
                )}
              </div>
            );
          })}
        </div>

        {/* Add File Button */}
        <button
          onClick={onCreateFile}
          title="Add new file (.dxml or .dcss)"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
            background: '#27272a',
            color: '#e4e4e7',
            border: 'none',
            padding: '4px 8px',
            borderRadius: '4px',
            fontSize: '11px',
            cursor: 'pointer',
          }}
        >
          <Plus size={12} /> New File
        </button>
      </div>

      {/* Monaco Editor Pane */}
      <div style={{ flex: 1, minHeight: 0 }}>
        <Editor
          key={activeFile.id}
          height="100%"
          language={activeFile.language}
          theme="vs-dark"
          value={activeFile.content}
          onMount={onEditorMount}
          onChange={onEditorChange}
          options={{
            minimap: { enabled: false },
            fontSize: 13,
            wordWrap: 'on',
            formatOnPaste: true,
            formatOnType: true,
            scrollBeyondLastLine: false,
            lineNumbers: 'on',
            renderWhitespace: 'selection',
            automaticLayout: true,
            tabSize: 2,
            autoClosingBrackets: 'always',
            autoClosingQuotes: 'always',
            linkedEditing: true,
            suggestOnTriggerCharacters: true,
          }}
        />
      </div>
    </div>
  );
};
