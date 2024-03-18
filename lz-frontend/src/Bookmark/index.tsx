import { Card, CardBody, HStack, Heading, Stack, Text } from "@chakra-ui/react";
import type { components } from "../api/v1.d.ts";
import { useLocation } from "react-router-dom";
import { BookmarkTag } from "../BookmarkTag.tsx";
import type { TagSet } from "../BookmarkTag.tsx";

export function Bookmark({
  bookmark,
  tags,
}: components["schemas"]["AnnotatedBookmark"]) {
  const location = useLocation();
  const onTagPage = location.pathname.match(/^\/tag\//);
  const existingTags: TagSet = {};
  if (onTagPage) {
    const tagQuery = location.pathname.split("/", 3).slice(-1)[0] || "";
    for (const tag of tagQuery.split(" ")) {
      existingTags[tag] = true;
    }
  }
  return (
    <Card key={bookmark.id}>
      <CardBody>
        <Heading size="sm">
          <a href={bookmark.url}>{bookmark.title}</a>
        </Heading>
        <HStack>
          {tags.map(({ name }) => (
            <BookmarkTag key={name} name={name}></BookmarkTag>
          ))}
        </HStack>
        <Stack spacing={2}>
          {bookmark.description && bookmark.description !== "" && (
            <Text>{bookmark.description}</Text>
          )}
          {bookmark.notes && bookmark.notes !== "" && (
            <Card variant="filled">
              <CardBody>
                <Text>{bookmark.notes}</Text>
              </CardBody>
            </Card>
          )}
        </Stack>
      </CardBody>
    </Card>
  );
}
