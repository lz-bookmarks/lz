import type { components } from "../api/v1.d.ts";
import { Bookmark } from "../Bookmark";
import { Stack } from "@chakra-ui/react";

type ListBookmarksArgs = {
  pages: Array<components["schemas"]["ListBookmarkResult"]>;
};

export function ListBookmarks({ pages }: ListBookmarksArgs) {
  return pages.map((group, i) => {
    return (
      group.bookmarks && (
        <Stack key={i}>
          {group.bookmarks.map((args) => (
            <Bookmark key={args.bookmark.id} {...args} />
          ))}
        </Stack>
      )
    );
  });
}
