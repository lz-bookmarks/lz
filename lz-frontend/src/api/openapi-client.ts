import createClient from "openapi-fetch";
import type { paths } from "./v1";

export default createClient<paths>({
  baseUrl: "/api/v1/",
});
