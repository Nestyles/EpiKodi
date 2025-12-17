import { Routes, Route } from "react-router-dom";
import Library from "./components/Library";
import Details from "./components/Details";
import Settings from "./components/Settings";

function App() {
  return (
    <Routes>
      <Route path="/" element={<Library />} />
      <Route path="/details" element={<Details />} />
      <Route path="/settings" element={<Settings />} />
    </Routes>
  );
}

export default App;
