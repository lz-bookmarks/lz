import { createContext } from "react";

export interface TagSet {
  [key: string]: boolean | undefined;
}

const emptyTags: TagSet = {};
export const TagContext = createContext(emptyTags);
