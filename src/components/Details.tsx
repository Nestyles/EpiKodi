import { useLocation, useNavigate } from "react-router-dom";
import { Box, Button, Image, Text, VStack, HStack, Badge } from "@chakra-ui/react";
import { convertFileSrc } from '@tauri-apps/api/core';
import ReactPlayer from "react-player";
import { useState } from "react";
import VideoPlayer from "./VideoPlayer";

function Details() {
  const location = useLocation();
  const navigate = useNavigate();
  const media = location.state?.media;
  const [playing, setPlaying] = useState(false);

  if (!media) {
    return <Box p={8}><Text>No media selected</Text></Box>;
  }

  let poster: string | null = null;
  let synopsis: string = "";
  let cast: string[] = [];
  let director: string = "";
  let year: string = "";
  let genres: string[] = [];

  try {
    if (media.synopsis_json) {
      const meta = typeof media.synopsis_json === 'string' ? JSON.parse(media.synopsis_json) : media.synopsis_json;
      if (meta) {
        poster = meta.poster_path || meta.poster_url;
        synopsis = meta.overview || meta.synopsis || "";
        cast = meta.cast || [];
        director = meta.director || "";
        year = meta.release_date ? new Date(meta.release_date).getFullYear().toString() : "";
        genres = meta.genres || [];
      }
    }
  } catch (e) {
    // ignore
  }

  if (poster && !poster.startsWith('http') && !poster.startsWith('file://')) {
    const pathStr = poster.replace(/\\/g, '/');
    poster = `${convertFileSrc(pathStr)}`;
  }

  const videoPathStr = media.path.replace(/\\/g, '/');
  const videoSrc = `${convertFileSrc(videoPathStr)}`;

  return (
    <Box p={8} maxW="1200px" mx="auto">
      <Button mb={4} onClick={() => navigate(-1)}>Back</Button>
      <HStack align="start" gap={8}>
        <Box flexShrink={0}>
          {poster ? (
            <Image src={poster} alt={media.title} maxH="400px" objectFit="cover" borderRadius="md" />
          ) : (
            <Box w="300px" h="400px" bg="gray.700" display="flex" alignItems="center" justifyContent="center" borderRadius="md">
              <Text color="gray.300">No poster</Text>
            </Box>
          )}
        </Box>
        <VStack align="start" gap={4} flex={1}>
          <Text fontSize="3xl" fontWeight="bold">{media.title}</Text>
          <HStack>
            <Badge colorScheme="blue">{media.media_type}</Badge>
            {year && <Text fontSize="lg">{year}</Text>}
            {media.tmdb_id && <Badge colorScheme="green">TMDB</Badge>}
          </HStack>
          {genres.length > 0 && (
            <HStack wrap="wrap">
              {genres.map((g: string) => <Badge key={g}>{g}</Badge>)}
            </HStack>
          )}
          <Text fontSize="md" color="gray.300">{synopsis}</Text>
          {director && <Text><strong>Director:</strong> {director}</Text>}
          {cast.length > 0 && (
            <Text><strong>Cast:</strong> {cast.slice(0, 5).join(', ')}{cast.length > 5 ? '...' : ''}</Text>
          )}
          <Button colorScheme="blue" onClick={() => setPlaying(true)}>Play</Button>
        </VStack>
      </HStack>
      {playing && videoSrc && (
        <Box mt={8}>
          <VideoPlayer src={videoSrc} />
        </Box>
      )}
    </Box>
  );
}

export default Details;