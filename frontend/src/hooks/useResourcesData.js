import { getResourcesData } from "../services/resourcesService.js";
import { useAsyncResource } from "./useAsyncResource.js";

const initialResourcesData = {
  resourceCourses: [],
  resources: [],
  resourceTypes: [],
};

export function useResourcesData() {
  return useAsyncResource(getResourcesData, initialResourcesData);
}
