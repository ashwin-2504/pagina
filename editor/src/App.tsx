import { useRef, useState, useCallback, useEffect } from 'react';
import Editor, { useMonaco } from '@monaco-editor/react';
import {
  FileText,
  Play,
  Eye,
  RefreshCw,
  ZoomIn,
  ZoomOut,
  Wand2,
  Plus,
  Trash2,
  Code2,
  Palette,
  BookOpen,
  Download,
  Upload,
  CheckCircle2,
  AlertTriangle
} from 'lucide-react';
import { parseDdlClient } from './parserClient';
import { DocumentRenderer } from './DocumentRenderer';
import { DocumentationSite } from './DocumentationSite';
import { formatDdl, formatCss } from './formatter';
import { registerMonacoDdlFeatures } from './monacoFeatures';
import { setupDdlLanguageTokenizer } from './ddlLanguage';
import { INITIAL_FILES } from './defaultFiles';
import type { ProjectFile } from './defaultFiles';
import type { ParsedDocument } from './types';

const ENGINE_API_BASE = 'http://127.0.0.1:3001';

export function App() {
  const monaco = useMonaco();
  const editorRef = useRef<any>(null);
  const previewContainerRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<any>(null);

  // Multi-file state
  const [files, setFiles] = useState<ProjectFile[]>(INITIAL_FILES);
  const [activeFileId, setActiveFileId] = useState<string>(INITIAL_FILES[0].id);

  const [parsedDoc, setParsedDoc] = useState<ParsedDocument | null>(null);
  const [errorInfo, setErrorInfo] = useState<any>(null);
  const [isCompiling, setIsCompiling] = useState<boolean>(false);
  const [isExporting, setIsExporting] = useState<boolean>(false);
  const [isImporting, setIsImporting] = useState<boolean>(false);
  const [engineOnline, setEngineOnline] = useState<boolean>(false);
  const [zoom, setZoom] = useState<number>(0.85);
  const [viewMode, setViewMode] = useState<string>('editor');

  const activeFile = files.find(f => f.id === activeFileId) || files[0];

  // Check Rust Engine Health
  useEffect(() => {
    const checkEngine = async () => {
      try {
        const res = await fetch(`${ENGINE_API_BASE}/health`, { signal: AbortSignal.timeout(2000) });
        setEngineOnline(res.ok);
      } catch {
        setEngineOnline(false);
      }
    };
    checkEngine();
    const interval = setInterval(checkEngine, 8000);
    return () => clearInterval(interval);
  }, []);

  // Helper to compile the active project
  const compileProject = useCallback((currentFiles: ProjectFile[]) => {
    setIsCompiling(true);
    try {
      // Find main document file (dxml or xml)
      const mainDocFile = currentFiles.find(f => f.name.endsWith('.dxml') || f.name.endsWith('.xml')) || currentFiles[0];

      // Build map of external stylesheets (.css and .dcss)
      const externalStylesheets: Record<string, string> = {};
      for (const f of currentFiles) {
        if (f.language === 'css' || f.name.endsWith('.css') || f.name.endsWith('.dcss')) {
          externalStylesheets[f.name] = f.content;
        }
      }

      const res = parseDdlClient(mainDocFile.content, externalStylesheets);
      setParsedDoc(res.doc);
      setErrorInfo(res.error);

      // In-editor squiggly markers if on main doc
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

  // Initial compile
  useEffect(() => {
    compileProject(files);
  }, [compileProject]);

  // Register Monaco language and features
  useEffect(() => {
    if (monaco) {
      setupDdlLanguageTokenizer(monaco);
      registerMonacoDdlFeatures(monaco);
    }
  }, [monaco]);

  // Handle Ctrl + Wheel zoom on preview canvas
  useEffect(() => {
    const container = previewContainerRef.current;
    if (!container) return;

    const handleWheel = (e: WheelEvent) => {
      if (e.ctrlKey) {
        e.preventDefault();
        const delta = e.deltaY < 0 ? 0.05 : -0.05;
        setZoom(prev => {
          const next = Math.round((prev + delta) * 100) / 100;
          return Math.min(2.0, Math.max(0.3, next));
        });
      }
    };

    container.addEventListener('wheel', handleWheel, { passive: false });
    return () => {
      container.removeEventListener('wheel', handleWheel);
    };
  }, []);

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

  // Format the current active file
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

  // Create new file
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

  // Delete file
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

  // Export Document via Native Rust Engine
  const handleExport = async (format: 'pdf' | 'docx') => {
    const mainDocFile = files.find(f => f.name.endsWith('.dxml') || f.name.endsWith('.xml')) || files[0];
    setIsExporting(true);

    try {
      const response = await fetch(`${ENGINE_API_BASE}/api/export/${format}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ddl: mainDocFile.content }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || `Export to ${format.toUpperCase()} failed.`);
      }

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `document.${format}`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      window.URL.revokeObjectURL(url);
    } catch (err: any) {
      alert(`Export Error: ${err.message || 'Make sure the Rust backend engine is running on port 3001.'}`);
    } finally {
      setIsExporting(false);
    }
  };

  // Reverse-Convert (Import DOCX / PDF via Native Rust Engine)
  const handleImportFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const isDocx = file.name.endsWith('.docx');
    const isPdf = file.name.endsWith('.pdf');

    if (!isDocx && !isPdf) {
      alert('Please select a valid .docx or .pdf file.');
      return;
    }

    const endpoint = isDocx ? 'docx' : 'pdf';
    setIsImporting(true);

    try {
      const arrayBuffer = await file.arrayBuffer();
      const response = await fetch(`${ENGINE_API_BASE}/api/import/${endpoint}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/octet-stream' },
        body: arrayBuffer,
      });

      if (!response.ok) {
        throw new Error(`Engine returned ${response.status}: ${await response.text()}`);
      }

      const result = await response.json();
      if (!result.success || !result.dxml) {
        throw new Error(result.error || 'Failed to reverse-engineer document structure.');
      }

      const newFiles: ProjectFile[] = [
        {
          id: 'document-dxml',
          name: 'document.dxml',
          language: 'ddl',
          content: result.dxml,
        },
        {
          id: 'styles-dcss',
          name: 'styles.dcss',
          language: 'css',
          content: result.dcss || '/* Generated styles */\n',
        },
      ];

      setFiles(newFiles);
      setActiveFileId('document-dxml');
      compileProject(newFiles);
    } catch (err: any) {
      alert(`Import Failed: ${err.message || 'Ensure Rust engine is operational.'}`);
    } finally {
      setIsImporting(false);
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', width: '100vw', height: '100vh', background: '#0f172a' }}>
      {/* Hidden File Input for Native Reverse-Engineering */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".docx,.pdf"
        style={{ display: 'none' }}
        onChange={handleImportFileChange}
      />

      {viewMode === 'docs' ? (
        <DocumentationSite onBackToStudio={() => setViewMode('editor')} />
      ) : (
        <>
          {/* Top Navbar */}
          <header style={{
            height: '48px',
            borderBottom: '1px solid #1e293b',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '0 16px',
            background: '#0f172a',
            color: '#f8fafc',
            flexShrink: 0,
            zIndex: 10,
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <FileText size={18} color="#38bdf8" />
              <span style={{ fontWeight: 700, fontSize: '15px', color: '#f8fafc' }}>Pagina Studio</span>
              <span style={{ fontSize: '11px', background: '#1e293b', color: '#38bdf8', padding: '2px 8px', borderRadius: '6px', fontWeight: 600 }}>Modular DDL</span>
              
              {/* Rust Engine Status Badge */}
              <div style={{
                display: 'flex',
                alignItems: 'center',
                gap: '5px',
                fontSize: '11px',
                padding: '2px 8px',
                borderRadius: '6px',
                background: engineOnline ? 'rgba(34, 197, 94, 0.15)' : 'rgba(239, 68, 68, 0.15)',
                color: engineOnline ? '#4ade80' : '#f87171',
                border: engineOnline ? '1px solid rgba(34, 197, 94, 0.3)' : '1px solid rgba(239, 68, 68, 0.3)',
                marginLeft: '8px',
              }}>
                {engineOnline ? <CheckCircle2 size={12} /> : <AlertTriangle size={12} />}
                <span>{engineOnline ? 'Rust Engine Online' : 'Engine Offline (Port 3001)'}</span>
              </div>

              {isCompiling && (
                <span style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '11px', color: '#38bdf8', marginLeft: '6px' }}>
                  <RefreshCw size={11} className="animate-spin" /> Compiling...
                </span>
              )}
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              {/* Docs Toggle Button */}
              <button
                onClick={() => setViewMode(v => v === 'docs' ? 'editor' : 'docs')}
                title="Open Documentation"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  background: viewMode === 'docs' ? '#38bdf8' : '#1e293b',
                  color: viewMode === 'docs' ? '#0f172a' : '#38bdf8',
                  border: '1px solid #334155',
                  padding: '6px 12px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 600,
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                <BookOpen size={13} /> {viewMode === 'docs' ? 'Close Docs' : 'Docs'}
              </button>

              {/* Format Button */}
              <button
                onClick={handleFormat}
                title="Format current file"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  background: '#1e293b',
                  color: '#38bdf8',
                  border: '1px solid #334155',
                  padding: '6px 12px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                <Wand2 size={13} /> Format
              </button>

              {/* Import / Reverse Engineer Button */}
              <button
                onClick={() => fileInputRef.current?.click()}
                disabled={isImporting}
                title="Reverse-engineer DOCX or PDF into .dxml and .dcss"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  background: '#1e293b',
                  color: '#a78bfa',
                  border: '1px solid #4c1d95',
                  padding: '6px 12px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 600,
                  cursor: isImporting ? 'not-allowed' : 'pointer',
                }}
              >
                <Upload size={13} /> {isImporting ? 'Importing...' : 'Import DOCX/PDF'}
              </button>

              {/* Export PDF Button */}
              <button
                onClick={() => handleExport('pdf')}
                disabled={isExporting}
                title="Export via Rust Engine to PDF"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  background: '#1e293b',
                  color: '#34d399',
                  border: '1px solid #065f46',
                  padding: '6px 12px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 600,
                  cursor: isExporting ? 'not-allowed' : 'pointer',
                }}
              >
                <Download size={13} /> PDF
              </button>

              {/* Export DOCX Button */}
              <button
                onClick={() => handleExport('docx')}
                disabled={isExporting}
                title="Export via Rust Engine to OpenXML (.docx)"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  background: '#1e293b',
                  color: '#60a5fa',
                  border: '1px solid #1e40af',
                  padding: '6px 12px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 600,
                  cursor: isExporting ? 'not-allowed' : 'pointer',
                }}
              >
                <Download size={13} /> DOCX
              </button>

              {/* Zoom controls */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', background: '#1e293b', padding: '2px 8px', borderRadius: '6px' }}>
                <button
                  onClick={() => setZoom(z => Math.max(0.4, Number((z - 0.1).toFixed(2))))}
                  title="Zoom Out"
                  style={{ background: 'none', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '2px' }}
                >
                  <ZoomOut size={14} />
                </button>
                <span style={{ fontSize: '11px', minWidth: '35px', textAlign: 'center', color: '#cbd5e1' }}>
                  {Math.round(zoom * 100)}%
                </span>
                <button
                  onClick={() => setZoom(z => Math.min(1.5, Number((z + 0.1).toFixed(2))))}
                  title="Zoom In"
                  style={{ background: 'none', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '2px' }}
                >
                  <ZoomIn size={14} />
                </button>
              </div>

              {/* Live Render Button */}
              <button
                onClick={() => compileProject(files)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  background: '#0284c7',
                  color: '#fff',
                  border: 'none',
                  padding: '6px 14px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                <Play size={13} /> Render
              </button>
            </div>
          </header>

          {/* Split Workspace */}
          <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
            {/* Left Pane: Multi-File Tab Bar & Monaco Editor */}
            <div style={{ width: '48%', borderRight: '1px solid #1e293b', display: 'flex', flexDirection: 'column', background: '#1e1e1e' }}>
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
                        onClick={() => setActiveFileId(file.id)}
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
                            onClick={(e) => handleDeleteFile(e, file.id)}
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
                  onClick={handleCreateFile}
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
                  onMount={handleEditorMount}
                  onChange={handleEditorChange}
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

            {/* Right Pane: Live Paged Document Preview */}
            <div style={{ width: '52%', display: 'flex', flexDirection: 'column', background: '#334155' }}>
              <div style={{
                height: '38px',
                padding: '0 16px',
                background: '#1e293b',
                borderBottom: '1px solid #0f172a',
                fontSize: '12px',
                fontWeight: 600,
                color: '#cbd5e1',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                flexShrink: 0,
              }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Eye size={13} /> Live Paged Canvas ({parsedDoc?.props?.paper?.size || 'A4'} {parsedDoc?.props?.paper?.orientation || 'portrait'})
                </div>
                <span style={{ fontSize: '11px', color: '#94a3b8' }}>
                  {files.length} project files (Ctrl + Scroll to zoom)
                </span>
              </div>

              {/* Virtual Paper Sheet Scroll Area */}
              <div
                ref={previewContainerRef}
                style={{
                  flex: 1,
                  minHeight: 0,
                  overflow: 'auto',
                  padding: '32px 16px',
                  display: 'flex',
                  justifyContent: 'center',
                }}
              >
                <div style={{
                  transform: `scale(${zoom})`,
                  transformOrigin: 'top center',
                  transition: 'transform 0.08s ease-out',
                }}>
                  <DocumentRenderer doc={parsedDoc} error={errorInfo} />
                </div>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export default App;
