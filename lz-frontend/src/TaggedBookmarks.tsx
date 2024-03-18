import invariant from "tiny-invariant";
import { useParams } from "react-router-dom";
import { createUseQuery } from "./api";
import { BookmarksPage } from "./BookmarksPage";
import { Text, HStack, Tag, TagLabel, TagLeftIcon } from "@chakra-ui/react";
import { CiHashtag } from "react-icons/ci";
import { BookmarkTag } from "./BookmarkTag";

type Params = {
  tag: string;
};

export function TaggedBookmarks() {
  const { tag } = useParams<Params>();
  invariant(tag);

  const queryResult = createUseQuery.useInfiniteFetch(
    "get",
    "/bookmarks/tagged/{query}",
    { params: { path: { query: tag! } } },
  );
  return (
    <BookmarksPage
      title={[<Text>Bookmarks tagged</Text>].concat(
        tag
          .split(" ")
          .map((name) => <BookmarkTag name={name} existingTags={{}} />),
      )}
      {...queryResult}
    ></BookmarksPage>
  );
}
