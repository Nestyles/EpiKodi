import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { Button, HStack, Input, Textarea, Box } from "@chakra-ui/react";

import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [scanPath, setScanPath] = useState<string>("");
  const [scanResult, setScanResult] = useState<any>(null);

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  async function scanDirectory() {
    try {
      // `scan_directory` will be implemented on the Rust side in a later step.
      const res = await invoke("scan_directory", { path: scanPath });
      setScanResult(res);
    } catch (e) {
      setScanResult({ error: String(e) });
    }
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <Input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
          size="sm"
          maxW="240px"
        />
        <Button type="submit" colorScheme="teal" size="sm">
          Greet
        </Button>
      </form>
      <p>{greetMsg}</p>

      <Box mt={6}>
        <HStack>
          <Input
            placeholder="Path to scan (e.g. C:/Movies)"
            value={scanPath}
            onChange={(e) => setScanPath(e.currentTarget.value)}
            size="sm"
            maxW="480px"
          />
          <Button onClick={scanDirectory} colorScheme="purple" size="sm">
            Scan Directory
          </Button>
        </HStack>
        <Box mt={3}>
          <Textarea
            readOnly
            value={scanResult ? JSON.stringify(scanResult, null, 2) : "No result yet"}
            minH="120px"
          />
        </Box>
      </Box>
    </main>
  );
}

export default App;
