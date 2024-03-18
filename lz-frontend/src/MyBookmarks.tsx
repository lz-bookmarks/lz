import { createUseQuery } from "./api";
import { BookmarksPage } from "./BookmarksPage";
import { TagContext } from "./TagContext";
import { Text } from "@chakra-ui/react";

export function MyBookmarks() {
  const queryResult = createUseQuery.useInfiniteFetch("get", "/bookmarks");
  return (
    <TagContext.Provider value={{}}>
      <BookmarksPage
        title={[<Text key="all">All bookmarks</Text>]}
        {...queryResult}
      ></BookmarksPage>
    </TagContext.Provider>
  );
}
