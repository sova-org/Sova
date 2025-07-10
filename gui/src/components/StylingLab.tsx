import React from 'react';
import { useColorContext } from '../context/ColorContext';

export const StylingLab: React.FC = () => {
  const { palette } = useColorContext();

  return (
    <div className="p-4 h-full overflow-y-auto" style={{ backgroundColor: 'var(--color-surface)' }}>
      <h2 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)' }}>Styling Lab</h2>
      
      <div className="space-y-6">
        {/* Color Showcase */}
        <div className="rounded-lg border p-4" style={{ backgroundColor: 'var(--color-background)', borderColor: 'var(--color-border)' }}>
          <h3 className="text-md font-medium mb-3" style={{ color: 'var(--color-text)' }}>Current Color Palette</h3>
          <div className="grid grid-cols-2 gap-4">
            {Object.entries(palette).map(([name, color]) => (
              <div key={name} className="flex items-center space-x-3">
                <div
                  className="w-8 h-8 rounded-md border"
                  style={{ backgroundColor: color }}
                />
                <div>
                  <div className="text-sm font-medium capitalize" style={{ color: 'var(--color-text)' }}>{name}</div>
                  <div className="text-xs font-mono" style={{ color: 'var(--color-muted)' }}>{color}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Typography Showcase */}
        <div className="rounded-lg border p-4" style={{ backgroundColor: 'var(--color-background)', borderColor: 'var(--color-border)' }}>
          <h3 className="text-md font-medium mb-3" style={{ color: 'var(--color-text)' }}>Typography</h3>
          <div className="space-y-3">
            <div>
              <h1 className="text-2xl font-bold" style={{ color: 'var(--color-primary)' }}>
                Primary Heading
              </h1>
              <p className="text-sm" style={{ color: 'var(--color-muted)' }}>Large title using primary color</p>
            </div>
            <div>
              <h2 className="text-lg font-semibold" style={{ color: 'var(--color-secondary)' }}>
                Secondary Heading
              </h2>
              <p className="text-sm" style={{ color: 'var(--color-muted)' }}>Medium title using secondary color</p>
            </div>
            <div>
              <p className="text-base" style={{ color: 'var(--color-text)' }}>
                Body text using the text color from our palette. This shows how readable text looks with the current color scheme.
              </p>
            </div>
            <div>
              <p className="text-sm" style={{ color: 'var(--color-muted)' }}>
                Muted text for secondary information and captions.
              </p>
            </div>
          </div>
        </div>

        {/* Button Showcase */}
        <div className="rounded-lg border p-4" style={{ backgroundColor: 'var(--color-background)', borderColor: 'var(--color-border)' }}>
          <h3 className="text-md font-medium mb-3" style={{ color: 'var(--color-text)' }}>Buttons</h3>
          <div className="flex flex-wrap gap-2">
            <button
              className="px-4 py-2 rounded-md text-white font-medium transition-opacity hover:opacity-90"
              style={{ backgroundColor: 'var(--color-primary)' }}
            >
              Primary Button
            </button>
            <button
              className="px-4 py-2 rounded-md text-white font-medium transition-opacity hover:opacity-90"
              style={{ backgroundColor: 'var(--color-secondary)' }}
            >
              Secondary Button
            </button>
            <button
              className="px-4 py-2 rounded-md text-white font-medium transition-opacity hover:opacity-90"
              style={{ backgroundColor: 'var(--color-success)' }}
            >
              Success Button
            </button>
            <button
              className="px-4 py-2 rounded-md text-white font-medium transition-opacity hover:opacity-90"
              style={{ backgroundColor: 'var(--color-error)' }}
            >
              Error Button
            </button>
            <button
              className="px-4 py-2 rounded-md text-white font-medium transition-opacity hover:opacity-90"
              style={{ backgroundColor: 'var(--color-warning)' }}
            >
              Warning Button
            </button>
            <button
              className="px-4 py-2 rounded-md font-medium transition-colors"
              style={{ 
                backgroundColor: 'var(--color-surface)',
                color: 'var(--color-text)',
                border: '1px solid var(--color-border)'
              }}
            >
              Outline Button
            </button>
          </div>
        </div>

        {/* Card Showcase */}
        <div className="rounded-lg border p-4" style={{ backgroundColor: 'var(--color-background)', borderColor: 'var(--color-border)' }}>
          <h3 className="text-md font-medium mb-3" style={{ color: 'var(--color-text)' }}>Cards</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              className="p-4 rounded-lg border"
              style={{ 
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)'
              }}
            >
              <h4 className="font-medium mb-2" style={{ color: 'var(--color-text)' }}>
                Surface Card
              </h4>
              <p className="text-sm" style={{ color: 'var(--color-muted)' }}>
                This card uses the surface background color with proper text contrast.
              </p>
            </div>
            <div
              className="p-4 rounded-lg"
              style={{ backgroundColor: 'var(--color-primary)' }}
            >
              <h4 className="font-medium mb-2 text-white">
                Primary Card
              </h4>
              <p className="text-sm text-white opacity-90">
                This card uses the primary color as background.
              </p>
            </div>
          </div>
        </div>

        {/* Form Elements */}
        <div className="rounded-lg border p-4" style={{ backgroundColor: 'var(--color-background)', borderColor: 'var(--color-border)' }}>
          <h3 className="text-md font-medium mb-3" style={{ color: 'var(--color-text)' }}>Form Elements</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1" style={{ color: 'var(--color-text)' }}>
                Input Field
              </label>
              <input
                type="text"
                placeholder="Enter text..."
                className="w-full px-3 py-2 rounded-md"
                style={{ 
                  backgroundColor: 'var(--color-surface)',
                  border: '1px solid var(--color-border)',
                  color: 'var(--color-text)'
                }}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1" style={{ color: 'var(--color-text)' }}>
                Select Dropdown
              </label>
              <select
                className="w-full px-3 py-2 rounded-md"
                style={{ 
                  backgroundColor: 'var(--color-surface)',
                  border: '1px solid var(--color-border)',
                  color: 'var(--color-text)'
                }}
              >
                <option>Option 1</option>
                <option>Option 2</option>
                <option>Option 3</option>
              </select>
            </div>
          </div>
        </div>

        {/* Status Indicators */}
        <div className="rounded-lg border p-4" style={{ backgroundColor: 'var(--color-background)', borderColor: 'var(--color-border)' }}>
          <h3 className="text-md font-medium mb-3" style={{ color: 'var(--color-text)' }}>Status Indicators</h3>
          <div className="flex flex-wrap gap-4">
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 rounded-full" style={{ backgroundColor: 'var(--color-success)' }} />
              <span className="text-sm" style={{ color: 'var(--color-text)' }}>Success</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 rounded-full" style={{ backgroundColor: 'var(--color-info)' }} />
              <span className="text-sm" style={{ color: 'var(--color-text)' }}>Info</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 rounded-full" style={{ backgroundColor: 'var(--color-warning)' }} />
              <span className="text-sm" style={{ color: 'var(--color-text)' }}>Warning</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 rounded-full" style={{ backgroundColor: 'var(--color-error)' }} />
              <span className="text-sm" style={{ color: 'var(--color-text)' }}>Error</span>
            </div>
            <div className="flex items-center space-x-2">
              <div className="w-3 h-3 rounded-full" style={{ backgroundColor: 'var(--color-muted)' }} />
              <span className="text-sm" style={{ color: 'var(--color-text)' }}>Inactive</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};