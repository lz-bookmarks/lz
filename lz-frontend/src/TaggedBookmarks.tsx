import { createUseQuery } from "./api";
import { BookmarksPage } from "./BookmarksPage";

interface TaggedBookmarksArgs {
  tag: string;
}

export function TaggedBookmarks({ tag }: TaggedBookmarksArgs) {
  const queryResult = createUseQuery.useInfiniteFetch(
    "get",
    "/bookmarks/tagged/{tag}",
    { params: { path: { tag: tag } } },
  );
  return <BookmarksPage {...queryResult}></BookmarksPage>;
}
