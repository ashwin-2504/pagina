import { useCallback } from 'react';
import type { Monaco } from '@monaco-editor/react';
import type { ProjectFile } from './defaultFiles';
import { formatDdl, formatCss } from './formatter';

interface EditorActionsProps {
  editorRef: React.MutableRefObject<any>;
  monaco: Monaco | null;
  files: ProjectFile[];
  setFiles: React.Dispatch<React.SetStateAction<ProjectFile[]>>;
  activeFileId: string;
  setActiveFileId: (id: string) => void;
  activeFile: ProjectFile;
  setAssets: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  compileProject: (files: ProjectFile[]) => void;
}

export function useEditorActions({
  editorRef,
  monaco,
  files,
  setFiles,
  activeFileId,
  setActiveFileId,
  activeFile,
  setAssets,
  compileProject,
}: EditorActionsProps) {
  const handleFormat = () => {
    try {
      const currentVal = editorRef.current ? editorRef.current.getValue() : activeFile.content;
      let formatted = currentVal;

      if (activeFile.language === 'ddl' || activeFile.name.endsWith('.dxml')) {
        formatted = formatDdl(currentVal);
      } else if (activeFile.language === 'css' || activeFile.name.endsWith('.css') || activeFile.name.endsWith('.dcss')) {
        formatted = formatCss(currentVal, '');
      }

      const nextFiles = files.map(f => (f.id === activeFileId ? { ...f, content: formatted } : f));
      setFiles(nextFiles);

      if (editorRef.current) {
        editorRef.current.setValue(formatted);
      }
      compileProject(nextFiles);
    } catch {
      // fallback
    }
  };

  const handleNavigateToLine = useCallback((line: number) => {
    if (!line || line < 1) return;

    const mainDocFile = files.find(f => f.name.endsWith('.dxml') || f.name.endsWith('.xml')) || files[0];
    if (mainDocFile && activeFileId !== mainDocFile.id) {
      setActiveFileId(mainDocFile.id);
    }

    window.getSelection()?.removeAllRanges();

    requestAnimationFrame(() => {
      setTimeout(() => {
        if (!editorRef.current) return;
        const editor = editorRef.current;
        const model = editor.getModel();
        if (!model) return;

        const totalLines = model.getLineCount();
        const targetLine = Math.min(Math.max(1, line), totalLines);
        const firstCol = model.getLineFirstNonWhitespaceColumn(targetLine) || 1;

        editor.revealLineInCenter(targetLine);
        editor.setPosition({ lineNumber: targetLine, column: firstCol });
        editor.focus();

        if (monaco) {
          const decorations = editor.deltaDecorations([], [
            {
              range: new monaco.Range(targetLine, 1, targetLine, model.getLineMaxColumn(targetLine)),
              options: {
                isWholeLine: true,
                className: 'monaco-line-flash-highlight',
              },
            },
          ]);

          setTimeout(() => {
            if (editorRef.current) {
              editorRef.current.deltaDecorations(decorations, []);
            }
          }, 1200);
        }
      }, 40);
    });
  }, [files, activeFileId, monaco, setActiveFileId, editorRef]);

  const handleImageSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      const dataUrl = event.target?.result as string;
      if (!dataUrl) return;

      const altName = file.name.replace(/\.[^/.]+$/, '');
      const assetFileName = file.name.replace(/\s+/g, '_');

      setAssets(prev => ({
        ...prev,
        [assetFileName]: dataUrl,
      }));

      const snippet = `\n<div align="center" style="margin: 4mm 0;">\n  <img src="${assetFileName}" alt="${altName}" style="max-width: 100%; height: auto;" />\n</div>\n`;

      if (editorRef.current) {
        const selection = editorRef.current.getSelection();
        editorRef.current.executeEdits('insert-image', [
          {
            range: selection,
            text: snippet,
            forceMoveMarkers: true,
          },
        ]);
        editorRef.current.focus();
      } else {
        const nextFiles = files.map(f =>
          f.id === activeFileId ? { ...f, content: f.content + snippet } : f
        );
        setFiles(nextFiles);
        compileProject(nextFiles);
      }
    };
    reader.readAsDataURL(file);
    e.target.value = '';
  };

  return {
    handleFormat,
    handleNavigateToLine,
    handleImageSelect,
  };
}
