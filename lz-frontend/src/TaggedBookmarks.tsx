import invariant from "tiny-invariant";
import { useLocation, useParams } from "react-router-dom";
import { createUseQuery } from "./api";
import { BookmarksPage } from "./BookmarksPage";
import { Text } from "@chakra-ui/react";
import { BookmarkTag } from "./BookmarkTag";
import { TagContext, TagSet } from "./TagContext";

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
  const existingTags: TagSet = {};
  const location = useLocation();
  const onTagPage = location.pathname.match(/^\/tag\//);
  if (onTagPage) {
    const tagQuery = location.pathname.split("/", 3).slice(-1)[0] || "";
    for (const tag of tagQuery.split("%20")) {
      existingTags[tag] = true;
    }
  }

  return (
    <TagContext.Provider value={existingTags}>
      <BookmarksPage
        title={[<Text key="text tagged">Bookmarks tagged</Text>].concat(
          Object.keys(existingTags).map((name) => (
            <BookmarkTag key={name} name={name} />
          )),
        )}
        {...queryResult}
      ></BookmarksPage>
    </TagContext.Provider>
  );
}
