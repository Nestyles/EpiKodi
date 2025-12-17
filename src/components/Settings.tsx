import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button, HStack, Input, Textarea, Box, Text, VStack } from "@chakra-ui/react";
import { open } from "@tauri-apps/plugin-dialog";

function Settings() {
  const navigate = useNavigate();
  const [scanPath, setScanPath] = useState<string>("");
  const [scanResult, setScanResult] = useState<any>(null);

  async function scanDirectory() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const res = await invoke("scan_directory", { path: scanPath });
      setScanResult(res);
    } catch (e) {
      setScanResult({ error: String(e) });
    }
  }

  return (
    <Box p={8} maxW="800px" mx="auto">
      <VStack spacing={6} align="stretch">
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

        <Button onClick={() => navigate('/')} colorScheme="blue">Back to Library</Button>
      </VStack>
    </Box>
  );
}

export default Settings;