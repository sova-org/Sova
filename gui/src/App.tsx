import { MainLayout } from "./components/MainLayout";
import { ColorProvider } from "./context/ColorContext";
import "./App.css";

function App() {
  return (
    <ColorProvider>
      <MainLayout />
    </ColorProvider>
  );
}

export default App;
