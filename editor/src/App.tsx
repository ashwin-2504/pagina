import { useRef, useState, useEffect } from 'react';
import { useMonaco } from '@monaco-editor/react';
import { Eye, MousePointerClick } from 'lucide-react';
import { DocumentRenderer } from './DocumentRenderer';
import { DocumentationSite } from './DocumentationSite';
import { registerMonacoDdlFeatures } from './monacoFeatures';
import { setupDdlLanguageTokenizer } from './ddlLanguage';
import { PreviewErrorBoundary } from './ErrorBoundary';
import { useProject } from './useProject';
import { Toolbar } from './Toolbar';
import { EditorPane } from './EditorPane';

export function App() {
  const monaco = useMonaco();
  const previewContainerRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const imageInputRef = useRef<HTMLInputElement>(null);

  const [zoom, setZoom] = useState<number>(0.85);
  const [viewMode, setViewMode] = useState<string>('editor');

  const {
    files,
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
  } = useProject(monaco);

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

  return (
    <div style={{ display: 'flex', flexDirection: 'column', width: '100vw', height: '100vh', background: '#0f172a' }}>
      {/* Hidden File Input for Native Reverse-Engineering */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".docx,.pdf"
        style={{ display: 'none' }}
        onChange={(e) => handleImportFileChange(e, fileInputRef)}
      />

      {/* Hidden File Input for Image Insertion */}
      <input
        type="file"
        ref={imageInputRef}
        accept="image/*"
        onChange={handleImageSelect}
        style={{ display: 'none' }}
      />

      {viewMode === 'docs' ? (
        <DocumentationSite onBackToStudio={() => setViewMode('editor')} />
      ) : (
        <>
          <Toolbar
            engineStatus={engineStatus}
            isCompiling={isCompiling}
            viewMode={viewMode}
            setViewMode={setViewMode}
            onFormat={handleFormat}
            onOpenImagePicker={() => imageInputRef.current?.click()}
            onOpenImportPicker={() => fileInputRef.current?.click()}
            isImporting={isImporting}
            isExporting={isExporting}
            onExport={handleExport}
            zoom={zoom}
            setZoom={setZoom}
            onRender={() => compileProject(files)}
          />

          {/* Split Workspace */}
          <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
            <EditorPane
              files={files}
              activeFileId={activeFileId}
              activeFile={activeFile}
              onSelectFile={setActiveFileId}
              onDeleteFile={handleDeleteFile}
              onCreateFile={handleCreateFile}
              onEditorMount={handleEditorMount}
              onEditorChange={handleEditorChange}
            />

            {/* Right Pane: Live Document Render Engine (Paginated Sheet Canvas) */}
            <div style={{ width: '52%', display: 'flex', flexDirection: 'column', background: '#0b1120', position: 'relative' }}>
              {/* Preview Status Ribbon */}
              <div className="no-print" style={{
                height: '38px',
                background: '#090d16',
                borderBottom: '1px solid #1e293b',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '0 16px',
                fontSize: '11px',
                color: '#94a3b8',
                fontWeight: 600,
                flexShrink: 0,
              }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <Eye size={13} /> Live Paged Canvas ({parsedDoc?.props?.paper?.size || 'A4'} {parsedDoc?.props?.paper?.orientation || 'portrait'})
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '16px', fontSize: '11px' }}>
                  <span style={{ display: 'flex', alignItems: 'center', gap: '4px', color: '#64748b' }}>
                    <MousePointerClick size={12} color="#38bdf8" /> Double-click text to jump to editor
                  </span>
                  <span style={{ color: '#64748b' }}>Ctrl + Scroll to Zoom</span>
                </div>
              </div>

              {/* Scrollable Viewport for Physical Paper Sheets */}
              <div
                ref={previewContainerRef}
                style={{
                  flex: 1,
                  overflow: 'auto',
                  background: '#0b1120',
                  padding: '32px 16px',
                  display: 'flex',
                  justifyContent: 'center',
                }}
              >
                <div className="zoom-wrapper" style={{
                  transform: `scale(${zoom})`,
                  transformOrigin: 'top center',
                  transition: 'transform 0.08s ease-out',
                }}>
                  <PreviewErrorBoundary onRetry={() => compileProject(files)}>
                    <DocumentRenderer
                      doc={parsedDoc}
                      error={errorInfo}
                      assets={assets}
                      onNavigateToLine={handleNavigateToLine}
                    />
                  </PreviewErrorBoundary>
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
