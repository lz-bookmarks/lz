import { Link } from "react-router-dom";
import { Tag, TagLabel, TagLeftIcon } from "@chakra-ui/react";
import { CiHashtag } from "react-icons/ci";
import { TagContext } from "./TagContext.tsx";
import { useContext } from "react";

interface Params {
  name: string;
}

export function BookmarkTag({ name }: Params) {
  const existingTags = useContext(TagContext);
  return (
    <Tag key={name}>
      <TagLeftIcon as={CiHashtag} />
      <TagLabel>
        {existingTags[name] ? (
          name
        ) : (
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
        )}
      </TagLabel>
    </Tag>
  );
}
