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
        return (
          group.bookmarks && (
            <React.Fragment key={i}>
              {group.bookmarks.map(({ bookmark, tags }) => (
                <article key={bookmark.id}>
                  <div>
                    <a href={bookmark.url}>{bookmark.title}</a>
                    <div>{bookmark.description}</div>
                    <ul className="tags">
                      {tags.map((tag) => (
                        <li key={tag.name}>
                          <a href="#" key={tag.name}>
                            {tag.name}
                          </a>
                        </li>
                      ))}
                    </ul>
                    {bookmark.notes && (
                      <blockquote>{bookmark.notes}</blockquote>
                    )}
                  </div>
                </article>
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
