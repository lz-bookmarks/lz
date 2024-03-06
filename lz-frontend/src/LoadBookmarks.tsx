import React from "react";
import { createUseQuery } from "./api";
import { Bookmark } from "./Bookmark";

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
      {data.pages.map((group, i) => {
        return (
          group.bookmarks && (
            <React.Fragment key={i}>
              {group.bookmarks.map((args) => (
                <Bookmark {...args} />
              ))}
            </React.Fragment>
          )
        );
      })}
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
