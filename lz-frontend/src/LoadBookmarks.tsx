import React from "react";
import { createUseQuery } from "./api";
import { ListBookmarks } from "./ListBookmarks";

export function LoadBookmarks() {
  const {
    data,
    error,
    isLoading,
    fetchNextPage,
    isFetchingNextPage,
    hasNextPage,
  } = createUseQuery.useInfiniteFetch("get", "/bookmarks");
  if (isLoading) {
    return <h1>Loading...</h1>;
  }
  if (!data || error) return <div>An error occurred: {error}</div>;
  return (
    <>
      <h1>LZ - bookmarks</h1>
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
