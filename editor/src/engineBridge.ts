import { invoke } from '@tauri-apps/api/core';

export const ENGINE_API_BASE = 'http://127.0.0.1:3001';

/**
 * Checks if the application is currently running inside the native Tauri desktop container.
 */
export function isTauriApp(): boolean {
  return typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__);
}

export interface EngineStatus {
  online: boolean;
  mode: 'tauri' | 'server' | 'offline';
  label: string;
}

export interface ImportResult {
  dxml: string;
  dcss: string;
}

/**
 * Health check that detects whether Tauri native commands are ready or fallback HTTP server is online.
 */
export async function checkEngineStatus(): Promise<EngineStatus> {
  if (isTauriApp()) {
    try {
      const res = await invoke<string>('ping_engine');
      if (res === 'native-tauri') {
        return { online: true, mode: 'tauri', label: 'Tauri Native Engine' };
      }
    } catch (e) {
      console.warn('Tauri ping_engine failed:', e);
    }
  }

  // Fallback to local HTTP engine daemon
  try {
    const res = await fetch(`${ENGINE_API_BASE}/health`, { signal: AbortSignal.timeout(2000) });
    if (res.ok) {
      return { online: true, mode: 'server', label: 'Local Daemon (Port 3001)' };
    }
  } catch {
    // server not reachable
  }

  return { online: false, mode: 'offline', label: 'Engine Offline' };
}

/**
 * Helper to convert standard Base64 string to Uint8Array.
 */
function base64ToUint8Array(base64: string): Uint8Array {
  const binaryString = window.atob(base64);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

/**
 * Helper to convert an ArrayBuffer to Base64 string.
 */
function arrayBufferToBase64(buffer: ArrayBuffer): string {
  let binary = '';
  const bytes = new Uint8Array(buffer);
  const len = bytes.byteLength;
  for (let i = 0; i < len; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return window.btoa(binary);
}

/**
 * Export document content to PDF or DOCX binary via native Tauri command or Axum daemon.
 */
export async function exportDocument(format: 'pdf' | 'docx', ddlContent: string): Promise<Blob> {
  const mimeType = format === 'pdf' ? 'application/pdf' : 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';

  if (isTauriApp()) {
    // Native direct IPC call to Rust ddl-engine
    const commandName = format === 'pdf' ? 'export_pdf' : 'export_docx';
    const b64Data = await invoke<string>(commandName, { ddl: ddlContent });
    const bytes = base64ToUint8Array(b64Data);
    return new Blob([bytes.buffer as ArrayBuffer], { type: mimeType });
  }

  // Fallback to HTTP daemon
  const response = await fetch(`${ENGINE_API_BASE}/api/export/${format}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ddl: ddlContent }),
  });

  if (!response.ok) {
    const errText = await response.text();
    throw new Error(errText || `Export to ${format.toUpperCase()} failed.`);
  }

  return await response.blob();
}

/**
 * Reverse-convert a PDF or DOCX file into DXML and DCSS.
 */
export async function importDocument(file: File): Promise<ImportResult> {
  const isDocx = file.name.toLowerCase().endsWith('.docx');
  const isPdf = file.name.toLowerCase().endsWith('.pdf');

  if (!isDocx && !isPdf) {
    throw new Error('Please select a valid .docx or .pdf file.');
  }

  const fileType = isDocx ? 'docx' : 'pdf';
  const arrayBuffer = await file.arrayBuffer();

  if (isTauriApp()) {
    // Native direct IPC call to Rust ddl-engine
    const commandName = isDocx ? 'import_docx' : 'import_pdf';
    const bytesBase64 = arrayBufferToBase64(arrayBuffer);
    const result = await invoke<ImportResult>(commandName, { bytesBase64 });
    return result;
  }

  // Fallback to HTTP daemon
  const response = await fetch(`${ENGINE_API_BASE}/api/import/${fileType}`, {
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

  return {
    dxml: result.dxml,
    dcss: result.dcss || '',
  };
}
