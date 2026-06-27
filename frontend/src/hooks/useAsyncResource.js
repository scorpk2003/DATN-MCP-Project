import { useEffect, useState } from "react";

export function useAsyncResource(loadResource, initialData = null) {
  const [data, setData] = useState(initialData);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    let isActive = true;

    async function load() {
      setLoading(true);
      setError(null);

      try {
        const nextData = await loadResource();

        if (isActive) {
          setData(nextData);
        }
      } catch (nextError) {
        if (isActive) {
          setError(nextError);
        }
      } finally {
        if (isActive) {
          setLoading(false);
        }
      }
    }

    load();

    return () => {
      isActive = false;
    };
  }, [loadResource, reloadKey]);

  return {
    data,
    loading,
    error,
    reload: () => setReloadKey((key) => key + 1),
  };
}
