import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { Button, HStack, Input, Textarea, Box, SimpleGrid, Image, Text, VStack, Spinner, Badge, CloseButton } from "@chakra-ui/react";
import { listMedias } from "./lib/tauri-commands";
import { open } from "@tauri-apps/plugin-dialog";
import { warn, debug, trace, info, error } from '@tauri-apps/plugin-log';
import ReactPlayer from "react-player";

import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [scanPath, setScanPath] = useState<string>("");
  const [scanResult, setScanResult] = useState<any>(null);
  const [mediaList, setMediaList] = useState<any[]>([]);
  const [playerPath, setPlayerPath] = useState<string>("");
  const [loading, setLoading] = useState(false);

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

  async function loadLibrary() {
    setLoading(true);
    try {
      const res: any = await listMedias({ page: 1, per_page: 200 });
      // res has { items, total, page, per_page }
      const items = res.items || [];
      setMediaList(items as any[]);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="container">
      <Box textAlign="center" my={8}>
        <Text fontSize="4xl" fontWeight="bold" mb={2}>Epikodi</Text>
        <Text fontSize="lg" color="gray.500">Scan and manage your media files easily</Text>
      </Box>


      <Box m={6}>
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
          <Button onClick={loadLibrary} colorScheme="blue" size="sm">
            Load Library
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

      <Box mt={8}>
        <Text fontSize="lg" fontWeight="bold" mb={3}>Library</Text>
        {loading ? (
          <Spinner />
        ) : (
          <SimpleGrid columns={[2, 3, 5]} spacing={4}>
            {mediaList.map((m) => {
              // parse synopsis_json for poster_path if present
              let poster: string | null = null;
              try {
                if (m.synopsis_json) {
                  const meta = typeof m.synopsis_json === 'string' ? JSON.parse(m.synopsis_json) : m.synopsis_json;
                  if (meta && meta.poster_path) poster = meta.poster_path;
                  else if (meta && meta.poster_url) poster = meta.poster_url;
                }
              } catch (e) {
                // ignore
              }

              // if poster is local path, convert to file:// URL for Tauri
              if (poster && !poster.startsWith('http') && !poster.startsWith('file://')) {
                // normalize Windows path
                const pathStr = poster.replace(/\\/g, '/');
                poster = `file:///${pathStr}`;
              }

              return (
                <Box 
                  key={m.path} 
                  borderRadius="md" 
                  overflow="hidden" 
                  bg="gray.800" 
                  p={2}
                  cursor="pointer"
                  _hover={{ bg: "gray.700" }}
                  onClick={() => {
                    try {
                      const pathStr = (m.path || '').replace(/\\/g, '/');
                      setPlayerPath(pathStr);
                    } catch (e) {
                      setPlayerPath("");
                    }
                    debug(m.path)
                  }}
                >
                  <VStack spacing={2} align="stretch">
                    <Box h="160px" display="flex" alignItems="center" justifyContent="center" bg="gray.700">
                      {poster ? (
                        <Image src={poster} alt={m.title} objectFit="cover" maxH="160px" />
                      ) : (
                        <Box color="gray.300">No poster</Box>
                      )}
                    </Box>
                    <Box>
                      <Text fontSize="sm" fontWeight="semibold" noOfLines={2}>{m.title}</Text>
                      <Text fontSize="xs" color="gray.400">{m.media_type} {m.tmdb_id ? <Badge ml={2} colorScheme="green">TMDB</Badge> : null}</Text>
                    </Box>
                  </VStack>
                </Box>
              );
            })}
          </SimpleGrid>
        )}
      </Box>
      {playerPath ? (
        <Box
          as="div"
          position="fixed"
          inset={0}
          bg="rgba(0,0,0,0.6)"
          display="flex"
          alignItems="center"
          justifyContent="center"
          zIndex={1400}
          role="dialog"
          aria-modal="true"
        >
          <Box position="relative" width={["95%", "80%", "70%"]} maxH="90%" bg="black">
            <CloseButton position="absolute" top="8px" right="8px" zIndex={1500} onClick={() => setPlayerPath(null)} />
            <Box as="div" p={0} height="100%">
              <video style={{ width: '100%', height: '100%' }} controls>
                <source src={`file:///${playerPath.replace(/\\/g, '/')}`} type="video/mp4" />
              </video>
            </Box>
          </Box>
        </Box>
      ) : null}
    </main>
  );
}

export default App;
