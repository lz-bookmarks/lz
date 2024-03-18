import type {
  UseInfiniteQueryResult,
  InfiniteData,
} from "@tanstack/react-query";
import { ListBookmarks } from "../ListBookmarks";
import { v1Components } from "../api";
import { Button, HStack, Heading, Stack, Text } from "@chakra-ui/react";

type Params = UseInfiniteQueryResult<
  InfiniteData<v1Components["schemas"]["ListBookmarkResult"], unknown>
> & {
  title: React.ReactElement[];
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
      <Heading>
        <HStack>
          <Text>LZ - </Text>
          {title}
        </HStack>
      </Heading>
      <Stack>
        <ListBookmarks pages={data.pages} />
        {isFetchingNextPage ? (
          <Text>Loading more...</Text>
        ) : (
          hasNextPage && (
            <Button
              onClick={() => fetchNextPage()}
              disabled={!hasNextPage || isFetchingNextPage}
            >
              Load more
            </Button>
          )
        )}
      </Stack>
    </>
  );
}
