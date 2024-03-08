import type {
  UseInfiniteQueryResult,
  InfiniteData,
} from "@tanstack/react-query";
import { ListBookmarks } from "../ListBookmarks";
import { v1Components } from "../api";

type Params = UseInfiniteQueryResult<
  InfiniteData<v1Components["schemas"]["ListBookmarkResult"], unknown>
> & {
  title: string;
};

export function BookmarksPage({
  title,
  data,
  error,
  isLoading,
  fetchNextPage,
  isFetchingNextPage,
  hasNextPage,
}: Params) {
  if (isLoading) {
    return <h1>Loading...</h1>;
  }
  if (!data || error) return <div>An error occurred: {error?.message}</div>;
  return (
    <>
      <h1>LZ - {title}</h1>
      <ListBookmarks pages={data.pages} />
      <button
        onClick={() => fetchNextPage()}
        disabled={!hasNextPage || isFetchingNextPage}
      >
        {isFetchingNextPage
          ? "Loading more..."
          : hasNextPage
          ? "Load More"
          : "Nothing more to load"}
      </button>
    </>
  );
}
