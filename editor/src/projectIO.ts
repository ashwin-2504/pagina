import { exportDocument, importDocument } from './engineBridge';
import type { ProjectFile } from './defaultFiles';

export async function exportProjectDocument(
  format: 'pdf' | 'docx',
  mainDocContent: string,
  assets: Record<string, string>,
  dcssContent?: string,
): Promise<void> {
  let exportDdl = mainDocContent;
  for (const [name, dataUrl] of Object.entries(assets)) {
    exportDdl = exportDdl.replaceAll(`src="${name}"`, `src="${dataUrl}"`);
    exportDdl = exportDdl.replaceAll(`src='${name}'`, `src='${dataUrl}'`);
  }

  if (dcssContent && !exportDdl.includes(dcssContent.slice(0, 30))) {
    if (exportDdl.includes('</props>')) {
      exportDdl = exportDdl.replace('</props>', `  <style>\n${dcssContent}\n  </style>\n  </props>`);
    }
  }

  const blob = await exportDocument(format, exportDdl);
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `document.${format}`;
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.URL.revokeObjectURL(url);
}

export async function importProjectDocument(file: File): Promise<{
  assets?: Record<string, string>;
  files: ProjectFile[];
}> {
  const result = await importDocument(file);
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

  return {
    assets: result.assets,
    files: newFiles,
  };
}
