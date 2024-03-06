import { useParams } from "react-router-dom";
import { createUseQuery } from "./api";
import { BookmarksPage } from "./BookmarksPage";

interface Params {
  tag: string;
}

export function TaggedBookmarks() {
  const { tag } = useParams<Params>();
  const queryResult = createUseQuery.useInfiniteFetch(
    "get",
    "/bookmarks/tagged/{tag}",
    { params: { path: { tag: tag } } },
  );
  return <BookmarksPage {...queryResult}></BookmarksPage>;
}
