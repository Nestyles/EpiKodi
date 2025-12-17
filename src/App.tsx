import { Routes, Route } from "react-router-dom";
import Library from "./components/Library";
import Details from "./components/Details";

function App() {
  return (
    <Routes>
      <Route path="/" element={<Library />} />
      <Route path="/details" element={<Details />} />
    </Routes>
  );
}

export default App;
