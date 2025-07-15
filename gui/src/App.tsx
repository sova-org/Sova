import { useEffect } from "react";
import { useStore } from "@nanostores/react";
import { MainLayout } from "./components/MainLayout";
import { ColorProvider } from "./context/ColorContext";
import { initializeLanguages } from "./languages";
import { editorSettingsStore } from "./stores/editorSettingsStore";
import "./App.css";

// Initialize language support
initializeLanguages();

function App() {
  const editorSettings = useStore(editorSettingsStore);

  // Apply font family to the entire document
  useEffect(() => {
    // Set on the root element which will cascade to everything
    document.documentElement.style.fontFamily = editorSettings.fontFamily;
    
    // Also update the CSS variable
    document.documentElement.style.setProperty('--app-font-family', editorSettings.fontFamily);
  }, [editorSettings.fontFamily]);

  return (
    <ColorProvider>
      <MainLayout />
    </ColorProvider>
  );
}

export default App;
