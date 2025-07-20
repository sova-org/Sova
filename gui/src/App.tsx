import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import { MainLayout } from "./components/MainLayout";
import { ServerManagerPanel } from "./components/ServerManagerPanel";
import { ConfirmCloseModal } from "./components/ConfirmCloseModal";
import { ColorProvider } from "./context/ColorContext";
import { initializeLanguages } from "./languages";
import { editorSettingsStore } from "./stores/editorSettingsStore";
import { showCloseConfirmation } from "./stores/appCloseStore";
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import "./App.css";

// Initialize language support
initializeLanguages();

function App() {
  const editorSettings = useStore(editorSettingsStore);
  const showCloseModal = useStore(showCloseConfirmation);

  // Apply font family to the entire document
  useEffect(() => {
    // Set on the root element which will cascade to everything
    document.documentElement.style.fontFamily = editorSettings.fontFamily;
    
    // Also update the CSS variable
    document.documentElement.style.setProperty('--app-font-family', editorSettings.fontFamily);
  }, [editorSettings.fontFamily]);

  // Listen for close confirmation events from Tauri
  useEffect(() => {
    const unlisten = listen('show-close-confirmation', () => {
      showCloseConfirmation.set(true);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleCloseCancel = () => {
    showCloseConfirmation.set(false);
  };

  const handleCloseConfirm = async () => {
    try {
      // First perform cleanup
      await invoke('shutdown_app');
      
      // Then actually close the app
      await invoke('close_app');
    } catch (error) {
      console.error('Error during shutdown:', error);
      // Close anyway if shutdown fails
      await invoke('close_app');
    }
  };

  return (
    <ColorProvider>
      <MainLayout />
      <ServerManagerPanel />
      <ConfirmCloseModal
        isOpen={showCloseModal}
        onClose={handleCloseCancel}
        onConfirm={handleCloseConfirm}
      />
    </ColorProvider>
  );
}

export default App;
