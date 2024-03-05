// A single-function dynamic hook that hooks react-query up to
// openapi-typescript. From
// https://gist.github.com/AshMW2724/7c7d248c35db3a894376686025e2df67,
// with modifications.

import { paths } from "./v1";
import {
  UseQueryOptions,
  UseQueryResult,
  useQuery,
} from "@tanstack/react-query";
import { FetchOptions, FetchResponse } from "openapi-fetch";
import type {
  FilterKeys,
  PathsWithMethod,
  HasRequiredKeys,
  HttpMethod,
} from "openapi-typescript-helpers";
import api from "./openapi-client";

type QueryOptions = { queryOpts?: Partial<UseQueryOptions> };

type CreateUseQuery<Paths extends {}> = {
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
};

export const createUseQuery: CreateUseQuery<paths> = {
  // @ts-expect-error It does return the correct type
  useFetch(method, url, ...init) {
    const options = init[0];
    return useQuery({
      queryKey: [url, options?.body, options?.params],
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
};
