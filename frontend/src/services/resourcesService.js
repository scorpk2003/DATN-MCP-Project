import { resourceCourses, resources, resourceTypes } from "../data/resourcesData.js";
import { request } from "../lib/httpClient.js";

export async function getResourcesData() {
  return request("/api/resources", {
    fallback: () => ({
      resourceCourses,
      resources,
      resourceTypes,
    }),
  });
}
