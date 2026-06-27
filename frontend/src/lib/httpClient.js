import { API_BASE_URL, USE_MOCK_API } from "../config/env.js";
import { getAuthToken } from "../auth/authService.js";

function buildUrl(path) {
  if (/^https?:\/\//i.test(path)) {
    return path;
  }

  return `${API_BASE_URL}${path}`;
}

export async function request(path, options = {}) {
  const { fallback, headers, ...fetchOptions } = options;

  if (USE_MOCK_API && fallback) {
    return fallback();
  }

  try {
    const token = await getAuthToken();
    const response = await fetch(buildUrl(path), {
      ...fetchOptions,
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...headers,
      },
    });

    if (!response.ok) {
      throw new Error(`Request failed with status ${response.status}`);
    }

    if (response.status === 204) {
      return null;
    }

    return response.json();
  } catch (error) {
    if (fallback) {
      return fallback();
    }

    throw error;
  }
}
