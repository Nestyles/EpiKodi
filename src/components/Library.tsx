import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Button, Box, SimpleGrid, Image, Text, VStack, Spinner, Badge } from "@chakra-ui/react";
import { listMedias, fetchMetadata } from "../lib/tauri-commands";
import { toaster } from "../components/ui/toaster";
import { convertFileSrc } from '@tauri-apps/api/core';
import Navbar from "./Navbar";

function Library() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [mediaList, setMediaList] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  const category = searchParams.get("category") || "all";

  useEffect(() => {
    loadLibrary();
  }, [category]);

  async function loadLibrary() {
    setLoading(true);
    try {
      const mediaType = category === "all" ? null : category;
      const res: any = await listMedias({ page: 1, per_page: 200, media_type: mediaType });
      const items = res.items || [];
      setMediaList(items as any[]);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  }

  async function onFetchMetadata(media: any) {
    try {
      console.log(`Fetching metadata for ${media.title}...`);
      if (media.media_type !== 'video') {
        return;
      }
      const result = await fetchMetadata(media.title);
      console.log('Metadata result:', result);
      toaster.create({
        title: "Metadata fetched",
        description: `Updated metadata for ${media.title}`,
        type: "success",
        duration: 3000,
      });
      await loadLibrary();
    } catch (e) {
      console.error('fetchMetadata error:', e);
      toaster.create({
        title: "Error",
        description: String(e),
        type: "error",
        duration: 5000,
      });
    }
  }

  return (
    <main className="container">
      <Box textAlign="center" my={8} position="relative">
        <Text fontSize="4xl" fontWeight="bold" mb={2}>Epikodi</Text>
        <Text fontSize="lg" color="gray.500">Scan and manage your media files easily</Text>
        <Button position="absolute" top="0" right="0" onClick={() => navigate('/settings')} colorScheme="gray" size="sm">
          Settings
        </Button>
      </Box>

      <Box mt={8}>
        <Text fontSize="lg" fontWeight="bold" mb={3}>Library - {category.charAt(0).toUpperCase() + category.slice(1)}</Text>
        <Navbar />
        {loading ? (
          <Spinner />
        ) : (
          <SimpleGrid columns={[2, 3, 5]} spacing={4}>
            {mediaList.filter(el => el.media_type === category || category === "all").map((m) => {
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

              if (poster && !poster.startsWith('http') && !poster.startsWith('file://')) {
                const pathStr = poster.replace(/\\/g, '/');
                poster = `${convertFileSrc(pathStr)}`;
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
                    navigate('/details', { state: { media: m } });
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
                      <Button
                        mt={2}
                        size="xs"
                        colorScheme="orange"
                        variant="outline"
                        onClick={(e) => {
                          e.stopPropagation();
                          onFetchMetadata(m);
                        }}
                      >
                        Fetch Metadata
                      </Button>
                    </Box>
                  </VStack>
                </Box>
              );
            })}
          </SimpleGrid>
        )}
      </Box>
    </main>
  );
}

export default Library;