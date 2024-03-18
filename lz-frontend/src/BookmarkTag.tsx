import { Link } from "react-router-dom";
import { Tag, TagLabel, TagLeftIcon } from "@chakra-ui/react";
import { CiHashtag } from "react-icons/ci";

export interface TagSet {
  [key: string]: boolean | undefined;
}

interface Params {
  name: string;
  existingTags: TagSet;
}

export function BookmarkTag({ name, existingTags }: Params) {
  return (
    <Tag key={name}>
      <TagLeftIcon as={CiHashtag} />
      <TagLabel>
        <Link
          to={{
            pathname: `/tag/${Object.keys({
              ...existingTags,
              [name]: true,
            }).join(" ")}`,
          }}
        >
          {name}
        </Link>
      </TagLabel>
    </Tag>
  );
}
