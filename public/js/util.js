import { useState, useMemo, useEffect } from './depend/preact.hooks.js';
import produce from './depend/immer.module.js'

const JSON_HEADER = {
  'Content-Type': 'application/json'
}

export function usePost(url, body) {
  return useFetch(url, "POST", "no-cache", JSON_HEADER, JSON.stringify(body));
}

export function useGet(url) {
  return useFetch(url, "GET", undefined, JSON_HEADER);
}

function useFetch(url, method, cache, headers, body) {
  const [response, setResponse] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    const FetchData = async () => {
      try {
        const res = await fetch(url, {method, cache, headers, body});
        const json = await res.json();
        setResponse(json);
      } catch (error) {
        setError(error);
      }
    };
    FetchData();
  }, [url, method, body]);
  return { response, error };
};
