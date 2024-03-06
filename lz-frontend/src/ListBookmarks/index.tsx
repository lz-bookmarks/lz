import React from "react";
import type { components } from "../api/v1.d.ts";
import { Bookmark } from "../Bookmark";

type ListBookmarksArgs = {
  pages: Array<components["schemas"]["ListBookmarkResult"]>;
};

export function ListBookmarks({ pages }: ListBookmarksArgs) {
  return pages.map((group, i) => {
    return (
      group.bookmarks && (
        <React.Fragment key={i}>
          {group.bookmarks.map((args) => (
            <Bookmark key={args.bookmark.id} {...args} />
          ))}
        </React.Fragment>
      )
    );
  });
}
