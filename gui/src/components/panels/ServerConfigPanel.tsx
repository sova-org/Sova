import React, { useEffect } from 'react';
import { serverManagerActions } from '../../stores/server/serverManager';
import { ServerControls } from '../server/ServerControls';
import { ServerConfigForm } from '../server/ServerConfigForm';

export const ServerConfigPanel: React.FC = () => {
  // Initialize on mount
  useEffect(() => {
    serverManagerActions.initialize();
  }, []);

  return (
    <div className="p-4 space-y-6">
      {/* Server Status and Controls */}
      <div>
        <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
          Server Control
        </h3>
        <ServerControls layout="grid" size="small" />
      </div>

      {/* Configuration */}
      <div>
        <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
          Configuration
        </h3>
        <ServerConfigForm compact={true} />
      </div>
    </div>
  );
};