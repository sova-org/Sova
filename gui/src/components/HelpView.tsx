import React, { useState, Suspense } from 'react';
import { ChevronLeft, ChevronRight, Menu } from 'lucide-react';
import '../styles/mdx.css';

// Import MDX files
import GettingStarted from '../docs/getting-started.mdx';
import InterfaceOverview from '../docs/interface-overview.mdx';
import CodeEditor from '../docs/code-editor.mdx';
import GridInterface from '../docs/grid-interface.mdx';
import Collaboration from '../docs/collaboration.mdx';
import TipsAndTricks from '../docs/tips-and-tricks.mdx';

interface DocSection {
  id: string;
  title: string;
  component: React.ComponentType;
}

const sections: DocSection[] = [
  { id: 'getting-started', title: 'Getting Started', component: GettingStarted },
  { id: 'interface-overview', title: 'Interface Overview', component: InterfaceOverview },
  { id: 'code-editor', title: 'Code Editor', component: CodeEditor },
  { id: 'grid-interface', title: 'Grid Interface', component: GridInterface },
  { id: 'collaboration', title: 'Collaboration', component: Collaboration },
  { id: 'tips-and-tricks', title: 'Tips & Tricks', component: TipsAndTricks },
];

export const HelpView: React.FC = () => {
  const [activeSection, setActiveSection] = useState('getting-started');
  const [tocPosition, setTocPosition] = useState<'left' | 'right'>('left');
  const [isTocCollapsed, setIsTocCollapsed] = useState(false);

  const ActiveComponent = sections.find(s => s.id === activeSection)?.component || sections[0]!.component;

  const TableOfContents = () => (
    <div 
      className="border-r h-full overflow-y-auto"
      style={{ 
        backgroundColor: 'var(--color-surface)', 
        borderColor: 'var(--color-border)',
        width: isTocCollapsed ? '0' : '20%',
        transition: 'width 0.3s ease'
      }}
    >
      {!isTocCollapsed && (
        <div className="p-4">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold" style={{ color: 'var(--color-text)' }}>
              Contents
            </h2>
            <button
              onClick={() => setTocPosition(tocPosition === 'left' ? 'right' : 'left')}
              className="p-1 transition-colors hover:opacity-80"
              style={{ color: 'var(--color-text)' }}
              title={`Move to ${tocPosition === 'left' ? 'right' : 'left'}`}
            >
              {tocPosition === 'left' ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
            </button>
          </div>
          <nav className="space-y-2">
            {sections.map((section) => (
              <button
                key={section.id}
                onClick={() => setActiveSection(section.id)}
                className="w-full text-left p-2 transition-colors hover:opacity-80"
                style={{
                  backgroundColor: activeSection === section.id ? 'var(--color-primary)' : 'transparent',
                  color: activeSection === section.id ? 'white' : 'var(--color-text)',
                }}
              >
                <span className="text-sm font-medium">{section.title}</span>
              </button>
            ))}
          </nav>
        </div>
      )}
    </div>
  );

  const MainContent = () => (
    <div 
      className="flex-1 overflow-auto"
      style={{ 
        backgroundColor: 'var(--color-background)',
        width: isTocCollapsed ? '100%' : '80%',
        transition: 'width 0.3s ease'
      }}
    >
      <div className="p-6">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-3xl font-bold" style={{ color: 'var(--color-text)' }}>
            Sova Documentation
          </h1>
          <button
            onClick={() => setIsTocCollapsed(!isTocCollapsed)}
            className="p-2 border transition-colors"
            style={{ 
              borderColor: 'var(--color-border)',
              color: 'var(--color-text)',
              backgroundColor: 'transparent'
            }}
            title={isTocCollapsed ? 'Show table of contents' : 'Hide table of contents'}
          >
            <Menu size={20} />
          </button>
        </div>
        
        <div className="mdx-content max-w-none">
          <Suspense fallback={<div style={{ color: 'var(--color-text-secondary)' }}>Loading...</div>}>
            <ActiveComponent />
          </Suspense>
        </div>
      </div>
    </div>
  );

  return (
    <div className="flex-1 flex h-full" style={{ backgroundColor: 'var(--color-background)' }}>
      {tocPosition === 'left' && <TableOfContents />}
      <MainContent />
      {tocPosition === 'right' && <TableOfContents />}
    </div>
  );
};