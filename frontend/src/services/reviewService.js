import { reviewQueue } from "../data/reviewData.js";
import { request } from "../lib/httpClient.js";

export async function getReviewData() {
  return request("/review", {
    fallback: () => ({
      reviewQueue,
    }),
  });
}
