import { useState, useRef, useCallback, useEffect } from 'react';
import type { Monaco } from '@monaco-editor/react';
import { INITIAL_FILES, type ProjectFile } from './defaultFiles';
import type { ParsedDocument } from './types';
import { parseDdlClient } from './parserClient';
import { isTauriApp, checkEngineStatus, type EngineStatus } from './engineBridge';
import { exportProjectDocument, importProjectDocument } from './projectIO';
import { useEditorActions } from './useEditorActions';

export function useProject(monaco: Monaco | null) {
  const editorRef = useRef<any>(null);
  const debounceRef = useRef<any>(null);

  // Multi-file state
  const [files, setFiles] = useState<ProjectFile[]>(INITIAL_FILES);
  const [activeFileId, setActiveFileId] = useState<string>(INITIAL_FILES[0].id);
  const [assets, setAssets] = useState<Record<string, string>>({});

  const [parsedDoc, setParsedDoc] = useState<ParsedDocument | null>(null);
  const [errorInfo, setErrorInfo] = useState<any>(null);
  const [isCompiling, setIsCompiling] = useState<boolean>(false);
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [isImporting, setIsImporting] = useState<boolean>(false);
  const [engineStatus, setEngineStatus] = useState<EngineStatus>({
    online: false,
    mode: 'offline',
    label: isTauriApp() ? 'Connecting to Tauri...' : 'Connecting...',
  });

  const activeFile = files.find(f => f.id === activeFileId) || files[0];

  useEffect(() => {
    const updateStatus = async () => {
      const status = await checkEngineStatus();
      setEngineStatus(status);
    };
    updateStatus();
    const interval = setInterval(updateStatus, isTauriApp() ? 15000 : 8000);
    return () => clearInterval(interval);
  }, []);

  const compileProject = useCallback((currentFiles: ProjectFile[]) => {
    setIsCompiling(true);
    try {
      const mainDocFile = currentFiles.find(f => f.name.endsWith('.dxml') || f.name.endsWith('.xml')) || currentFiles[0];

      const externalStylesheets: Record<string, string> = {};
      for (const f of currentFiles) {
        if (f.language === 'css' || f.name.endsWith('.css') || f.name.endsWith('.dcss')) {
          externalStylesheets[f.name] = f.content;
        }
      }

      const res = parseDdlClient(mainDocFile.content, externalStylesheets);
      setParsedDoc(res.doc);
      setErrorInfo(res.error);

      if (monaco && editorRef.current) {
        const model = editorRef.current.getModel();
        if (model) {
          if (res.error && activeFileId === mainDocFile.id) {
            const errLine = res.error.line || 1;
            const errCol = res.error.column || 1;
            const lineContent = model.getLineContent(errLine) || '';
            const maxCol = Math.max(errCol + 1, lineContent.length + 1);

            monaco.editor.setModelMarkers(model, 'ddl-syntax', [
              {
                startLineNumber: errLine,
                startColumn: Math.max(1, errCol - 1),
                endLineNumber: errLine,
                endColumn: maxCol,
                message: res.error.message,
                severity: monaco.MarkerSeverity.Error,
              },
            ]);
          } else {
            monaco.editor.setModelMarkers(model, 'ddl-syntax', []);
          }
        }
      }
    } catch (err: any) {
      setErrorInfo({
        message: err?.message || 'Failed to parse project files',
      });
    } finally {
      setIsCompiling(false);
    }
  }, [monaco, activeFileId]);

  useEffect(() => {
    compileProject(files);
  }, [compileProject]);

  const handleEditorChange = (value: string | undefined) => {
    const val = value || '';
    const nextFiles = files.map(f => (f.id === activeFileId ? { ...f, content: val } : f));
    setFiles(nextFiles);

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(() => {
      compileProject(nextFiles);
    }, 120);
  };

  const handleEditorMount = (editor: any) => {
    editorRef.current = editor;
  };

  const { handleFormat, handleNavigateToLine, handleImageSelect } = useEditorActions({
    editorRef,
    monaco,
    files,
    setFiles,
    activeFileId,
    setActiveFileId,
    activeFile,
    setAssets,
    compileProject,
  });

  const handleCreateFile = () => {
    const fileName = prompt('Enter new file name (e.g. data.dxml, theme.dcss, styles.css):', 'custom.dcss');
    if (!fileName || !fileName.trim()) return;

    const trimmed = fileName.trim();
    const isCss = trimmed.endsWith('.css') || trimmed.endsWith('.dcss');
    const newFile: ProjectFile = {
      id: `file-${Date.now()}`,
      name: trimmed,
      language: isCss ? 'css' : 'ddl',
      content: isCss ? '/* Custom stylesheet */\n\n' : '<section>\n  <p>New content section</p>\n</section>\n',
    };

    const nextFiles = [...files, newFile];
    setFiles(nextFiles);
    setActiveFileId(newFile.id);
  };

  const handleDeleteFile = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    if (files.length <= 1) {
      alert('Cannot delete the last remaining file.');
      return;
    }
    const nextFiles = files.filter(f => f.id !== id);
    setFiles(nextFiles);
    if (activeFileId === id) {
      setActiveFileId(nextFiles[0].id);
    }
    compileProject(nextFiles);
  };

  const handleExport = async (format: 'pdf' | 'docx') => {
    const mainDocFile = files.find(f => f.name.endsWith('.dxml') || f.name.endsWith('.xml')) || files[0];
    const dcssFile = files.find(f => f.name.endsWith('.dcss') || f.name.endsWith('.css'));
    setIsExporting(true);

    try {
      await exportProjectDocument(format, mainDocFile.content, assets, dcssFile?.content);
    } catch (err: any) {
      alert(`Export Error: ${err.message || 'Export failed.'}`);
    } finally {
      setIsExporting(false);
    }
  };

  const handleImportFileChange = async (e: React.ChangeEvent<HTMLInputElement>, fileInputRef: React.RefObject<HTMLInputElement | null>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const isDocx = file.name.endsWith('.docx');
    const isPdf = file.name.endsWith('.pdf');

    if (!isDocx && !isPdf) {
      alert('Please select a valid .docx or .pdf file.');
      return;
    }

    setIsImporting(true);

    try {
      const result = await importProjectDocument(file);
      if (result.assets) {
        setAssets(result.assets);
      }

      setFiles(result.files);
      setActiveFileId(result.files[0].id);
      compileProject(result.files);
    } catch (err: any) {
      alert(`Import Error: ${err.message || 'Failed to import document.'}`);
    } finally {
      setIsImporting(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  return {
    files,
    setFiles,
    activeFileId,
    setActiveFileId,
    activeFile,
    assets,
    parsedDoc,
    errorInfo,
    isCompiling,
    isExporting,
    isImporting,
    engineStatus,
    compileProject,
    handleEditorChange,
    handleEditorMount,
    handleFormat,
    handleCreateFile,
    handleDeleteFile,
    handleNavigateToLine,
    handleImageSelect,
    handleExport,
    handleImportFileChange,
  };
}
