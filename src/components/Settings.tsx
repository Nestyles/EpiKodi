import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Button, HStack, Input, Textarea, Box, Text, VStack, List } from "@chakra-ui/react";
import { open } from "@tauri-apps/plugin-dialog";

interface ScannedDirectory {
  path: string;
  last_scanned: string;
}

function Settings() {
  const navigate = useNavigate();
  const [scanPath, setScanPath] = useState<string>("");
  const [scanResult, setScanResult] = useState<any>(null);
  const [scannedDirectories, setScannedDirectories] = useState<ScannedDirectory[]>([]);

  async function fetchScannedDirectories() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const res = await invoke("list_scanned_directories");
      console.log("Scan result:", res);
      setScannedDirectories(res as ScannedDirectory[]);
    } catch (e) {
      console.error("Failed to fetch scanned directories:", e);
    }
  }

  useEffect(() => {
    fetchScannedDirectories();
  }, []);

  async function scanDirectory() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const res = await invoke("scan_directory", { path: scanPath });
      setScanResult(res);
      // Refresh the scanned directories list
      await fetchScannedDirectories();
    } catch (e) {
      setScanResult({ error: String(e) });
    }
  }

  return (
    <Box p={8} maxW="800px" mx="auto">
      <VStack align="stretch">
        <Box textAlign="center">
          <Text fontSize="3xl" fontWeight="bold" mb={2}>Settings</Text>
          <Text fontSize="lg" color="gray.500">Configure your media library</Text>
        </Box>

        <Box>
          <Text fontSize="xl" fontWeight="semibold" mb={4}>Scan Directory</Text>
          <HStack>
            <Button onClick={async () => {
              const selected = await open({ directory: true });
              if (selected) setScanPath(selected as string);
            }} colorScheme="teal" size="sm">
              Pick Directory
            </Button>
            <Input
              placeholder="Pick a directory to scan"
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

        <Box>
          <Text fontSize="xl" fontWeight="semibold" mb={4}>Scanned Directories</Text>
          <List.Root>
            {scannedDirectories.length > 0 ? (
              scannedDirectories.map((dir, index) => (
                <List.Item key={index}>{dir.path} - Last scanned: {dir.last_scanned}</List.Item>
              ))
            ) : (
              <Text color="gray.500">No directories scanned yet.</Text>
            )}
          </List.Root>
        </Box>

        <Button onClick={() => navigate('/')} colorScheme="blue">Back to Library</Button>
      </VStack>
    </Box>
  );
}

export default Settings;