import { createUseQuery } from "./api";
import { BookmarksPage } from "./BookmarksPage";

export function MyBookmarks() {
  const queryResult = createUseQuery.useInfiniteFetch("get", "/bookmarks");
  return <BookmarksPage title="All bookmarks" {...queryResult}></BookmarksPage>;
}
