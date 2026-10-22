import React, { useState } from 'react';
import { BookOpen, ArrowLeft, ExternalLink, ChevronRight } from 'lucide-react';
import { NAV_ITEMS, DocsContent } from './docsContent';

interface DocsProps {
  onBackToStudio: () => void;
}

export const DocumentationSite: React.FC<DocsProps> = ({ onBackToStudio }) => {
  const [activeSection, setActiveSection] = useState<string>('intro');
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedCode(id);
    setTimeout(() => setCopiedCode(null), 2000);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', width: '100vw', background: '#090d16', color: '#f8fafc', fontFamily: 'Inter, system-ui, sans-serif' }}>
      {/* Top Navbar */}
      <header style={{
        height: '54px',
        borderBottom: '1px solid #1e293b',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 24px',
        background: '#0f172a',
        flexShrink: 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <button
            onClick={onBackToStudio}
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
            <ArrowLeft size={14} /> Back to Studio
          </button>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <BookOpen size={20} color="#38bdf8" />
            <span style={{ fontWeight: 700, fontSize: '16px', color: '#f8fafc' }}>Pagina Documentation</span>
            <span style={{ fontSize: '11px', background: '#1e293b', color: '#94a3b8', padding: '2px 8px', borderRadius: '12px' }}>v0.1 Spec</span>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '16px', fontSize: '13px' }}>
          <a
            href="https://github.com/ashwin-2504/pagina"
            target="_blank"
            rel="noreferrer"
            style={{ display: 'flex', alignItems: 'center', gap: '6px', color: '#94a3b8', textDecoration: 'none' }}
          >
            GitHub <ExternalLink size={13} />
          </a>
        </div>
      </header>

      {/* Main Documentation Split */}
      <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
        {/* Sidebar Nav */}
        <aside style={{
          width: '280px',
          borderRight: '1px solid #1e293b',
          background: '#0b1120',
          overflowY: 'auto',
          padding: '24px 16px',
          flexShrink: 0,
        }}>
          {NAV_ITEMS.map((group, gIdx) => (
            <div key={gIdx} style={{ marginBottom: '24px' }}>
              <div style={{
                fontSize: '11px',
                fontWeight: 700,
                color: '#64748b',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
                marginBottom: '8px',
                paddingLeft: '10px',
              }}>
                {group.category}
              </div>
              <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
                {group.items.map(item => {
                  const isActive = activeSection === item.id;
                  return (
                    <li key={item.id}>
                      <button
                        onClick={() => setActiveSection(item.id)}
                        style={{
                          width: '100%',
                          textAlign: 'left',
                          background: isActive ? '#1e293b' : 'transparent',
                          color: isActive ? '#38bdf8' : '#94a3b8',
                          border: 'none',
                          padding: '8px 12px',
                          borderRadius: '6px',
                          fontSize: '13px',
                          fontWeight: isActive ? 600 : 500,
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          marginBottom: '2px',
                          transition: 'all 0.15s ease',
                        }}
                      >
                        <span>{item.label}</span>
                        {isActive && <ChevronRight size={14} color="#38bdf8" />}
                      </button>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </aside>

        {/* Content Pane */}
        <DocsContent
          activeSection={activeSection}
          copiedCode={copiedCode}
          onCopy={copyToClipboard}
        />
      </div>
    </div>
  );
};
