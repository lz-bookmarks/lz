import { Card, CardBody, HStack, Heading, Stack, Text } from "@chakra-ui/react";
import type { components } from "../api/v1.d.ts";
import { BookmarkTag } from "../BookmarkTag.tsx";

export function Bookmark({
  bookmark,
  tags,
  associations,
}: components["schemas"]["AnnotatedBookmark"]) {
  return (
    <Card key={bookmark.id}>
      <CardBody>
        <Heading size="sm">
          <HStack spacing={10} align={"left"}>
            <a href={bookmark.url}>{bookmark.title}</a>
            {associations !== [] && (
              <HStack>
                {associations.map(({ context, link }) => (
                  <a href={link}>{context || "=>"}</a>
                ))}
              </HStack>
            )}
          </HStack>
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
