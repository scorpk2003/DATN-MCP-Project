import { getNavigationData } from "../services/navigationService.js";
import { useAsyncResource } from "./useAsyncResource.js";

const initialNavigationData = {
  recentChats: [],
  sidebarItems: [],
};

export function useNavigationData() {
  return useAsyncResource(getNavigationData, initialNavigationData);
}
