import { getReviewData } from "../services/reviewService.js";
import { useAsyncResource } from "./useAsyncResource.js";

const initialReviewData = {
  reviewQueue: [],
};

export function useReviewData() {
  return useAsyncResource(getReviewData, initialReviewData);
}
