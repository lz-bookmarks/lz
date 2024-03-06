import invariant from "tiny-invariant";
import { useParams } from "react-router-dom";
import { createUseQuery } from "./api";
import { BookmarksPage } from "./BookmarksPage";

type Params = {
  tag: string;
};

export function TaggedBookmarks() {
  const { tag } = useParams<Params>();
  invariant(tag);

  const queryResult = createUseQuery.useInfiniteFetch(
    "get",
    "/bookmarks/tagged/{tag}",
    { params: { path: { tag: tag! } } },
  );
  return <BookmarksPage {...queryResult}></BookmarksPage>;
}
