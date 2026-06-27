import { recentChats } from "../data/dashboardData.js";
import { sidebarItems } from "../data/navigationData.js";
import { request } from "../lib/httpClient.js";

export async function getNavigationData() {
  return request("/api/navigation", {
    fallback: () => ({
      recentChats,
      sidebarItems,
    }),
  });
}
