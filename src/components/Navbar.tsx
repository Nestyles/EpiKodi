import { TabsRoot, TabsList, TabsTrigger, Box, HStack, Text, TabsValueChangeDetails } from "@chakra-ui/react";
import { useNavigate, useLocation } from "react-router-dom";
import { MdVideoLibrary, MdAudiotrack, MdImage, MdLibraryBooks } from "react-icons/md";

const categories = [
  { key: "all", label: "All", icon: MdLibraryBooks },
  { key: "video", label: "Video", icon: MdVideoLibrary },
  { key: "audio", label: "Audio", icon: MdAudiotrack },
  { key: "image", label: "Image", icon: MdImage },
];

function Navbar() {
  const navigate = useNavigate();
  const location = useLocation();

  const searchParams = new URLSearchParams(location.search);
  const currentCategory = searchParams.get("category") || "all";

  const handleTabChange = (value: TabsValueChangeDetails) => {
    const newSearchParams = new URLSearchParams(location.search);
    console.log("val", value)
    if (value.value === "all") {
      newSearchParams.delete("category");
    } else {
      newSearchParams.set("category", value.value);
    }
    navigate({ search: newSearchParams.toString() });
  };

  return (
    <Box bg="gray.900" p={4} borderRight="1px" borderColor="gray.700" h="100%">
      <TabsRoot value={currentCategory} onValueChange={handleTabChange} variant="enclosed" colorScheme="blue" orientation="vertical">
        <TabsList flexDirection="column" alignItems="stretch">
          {categories.map((cat) => (
            <TabsTrigger key={cat.key} value={cat.key} justifyContent="flex-start">
              <HStack>
                <cat.icon />
                <Text>{cat.label}</Text>
              </HStack>
            </TabsTrigger>
          ))}
        </TabsList>
      </TabsRoot>
    </Box>
  );
}

export default Navbar;