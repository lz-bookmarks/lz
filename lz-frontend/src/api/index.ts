// A single-function dynamic hook that hooks react-query up to
// openapi-typescript. From
// https://gist.github.com/AshMW2724/7c7d248c35db3a894376686025e2df67,
// with modifications.

import { paths } from "./v1";
import { useQuery, useInfiniteQuery } from "@tanstack/react-query";
import type {
  UseQueryOptions,
  UseInfiniteQueryOptions,
  UseQueryResult,
  UseInfiniteQueryResult,
  InfiniteData,
} from "@tanstack/react-query";
import { FetchOptions, FetchResponse } from "openapi-fetch";
import type {
  FilterKeys,
  PathsWithMethod,
  HasRequiredKeys,
  HttpMethod,
  SuccessResponse,
  ResponseObjectMap,
  MediaType,
} from "openapi-typescript-helpers";
import api from "./openapi-client";

type QueryOptions = { queryOpts?: Partial<UseQueryOptions> };
type InfiniteQueryOptions = { queryOpts?: Partial<UseInfiniteQueryOptions> };

type CreateUseQuery<Paths extends object> = {
  useFetch<T extends HttpMethod, P extends PathsWithMethod<Paths, T>>(
    method: T,
    url: P,
    ...init: HasRequiredKeys<
      FetchOptions<FilterKeys<Paths[P], T>>
    > extends never
      ? [((FetchOptions<FilterKeys<Paths[P], T>> & QueryOptions) | undefined)?]
      : [FetchOptions<FilterKeys<Paths[P], T>> & QueryOptions]
  ): UseQueryResult<
    FetchResponse<
      T extends keyof Paths[P] ? Paths[P][T] : unknown,
      FetchOptions<FilterKeys<Paths[P], T>> & QueryOptions
    >["data"],
    FetchResponse<
      T extends keyof Paths[P] ? Paths[P][T] : unknown,
      FetchOptions<FilterKeys<Paths[P], T>> & QueryOptions
    >["error"]
  >;
  useInfiniteFetch<T extends HttpMethod, P extends PathsWithMethod<Paths, T>>(
    method: T,
    url: P,
    ...init: HasRequiredKeys<
      FetchOptions<FilterKeys<Paths[P], T>>
    > extends never
      ? [
          (
            | (FetchOptions<FilterKeys<Paths[P], T>> & InfiniteQueryOptions)
            | undefined
          )?,
        ]
      : [FetchOptions<FilterKeys<Paths[P], T>> & InfiniteQueryOptions]
  ): UseInfiniteQueryResult<
    InfiniteData<
      FilterKeys<
        SuccessResponse<
          ResponseObjectMap<T extends keyof Paths[P] ? Paths[P][T] : unknown>
        >,
        MediaType
      >
    >,
    FetchResponse<
      InfiniteData<T extends keyof Paths[P] ? Paths[P][T] : unknown>,
      FetchOptions<FilterKeys<Paths[P], T>> & InfiniteQueryOptions
    >["error"]
  >;
};

export const createUseQuery: CreateUseQuery<paths> = {
  // @ts-expect-error It does return the correct type
  useFetch(method, url, ...init) {
    const options = init[0];
    return useQuery({
      queryKey: [url, options],
      queryFn: async ({ signal }) => {
        // @ts-expect-error All good, we know this method exists
        const { data, error } = await api[method.toUpperCase()](url, {
          ...options,
          signal,
        });
        if (data) return data;
        throw new Error(error);
      },
      ...options?.queryOpts,
    });
  },
  // @ts-expect-error It does return the correct type
  useInfiniteFetch(method, url, ...init) {
    const options = init[0];
    return useInfiniteQuery({
      queryKey: [url, options],
      queryFn: async ({ pageParam, signal }) => {
        // @ts-expect-error All good, we know this method exists
        const { data, error } = await api[method.toUpperCase()](url, {
          params: { query: { cursor: pageParam } },
          ...options,
          signal,
        });
        if (data) return data;
        throw new Error(error);
      },
      initialPageParam: null,
      getNextPageParam: (lastPage) => lastPage.nextCursor,
      ...options?.queryOpts,
    });
  },
};
