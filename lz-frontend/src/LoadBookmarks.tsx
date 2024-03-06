import React from "react";
import { createUseQuery } from "./api";

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
        console.log(group);

        return (
          group.bookmarks && (
            <React.Fragment key={i}>
              <ul>
                {group.bookmarks.map(({ bookmark, tags }) => (
                  <li key={bookmark.id}>
                    <a href={bookmark.url}>{bookmark.title}</a>
                    <div>{bookmark.description}</div>
                    <ul className="tags">
                      {tags.map((tag) => (
                        <li key={tag.name}>{tag.name}</li>
                      ))}
                    </ul>
                    <div>{bookmark.notes}</div>
                  </li>
                ))}
              </ul>
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
