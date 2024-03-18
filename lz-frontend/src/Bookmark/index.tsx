import {
  Card,
  CardBody,
  HStack,
  Heading,
  Stack,
  StackDivider,
  Tag,
  TagLabel,
  TagLeftIcon,
  Text,
} from "@chakra-ui/react";
import type { components } from "../api/v1.d.ts";
import { Link, useLocation } from "react-router-dom";
import { CiHashtag } from "react-icons/ci";

interface TagSet {
  [key: string]: boolean | undefined;
}

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
            <Tag key={name}>
              <TagLeftIcon as={CiHashtag} />
              <TagLabel>
                <Link
                  to={{
                    pathname: `/tag/${Object.keys({
                      ...existingTags,
                      [name]: true,
                    }).join(" ")}`,
                  }}
                >
                  {name}
                </Link>
              </TagLabel>
            </Tag>
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
