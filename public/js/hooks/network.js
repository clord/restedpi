import { useState, useMemo, useEffect } from '/static/js/depend/preact.hooks.js';
// import produce from '/static/js/depend/immer.module.js'

const JSON_HEADER = {
  'Content-Type': 'application/json'
}

export function usePost(url, body) {
    return useFetch(`${window.env.api}${url}`, "POST", "no-cache", JSON_HEADER, JSON.stringify(body));
}

export function useGet(url) {
    return useFetch(`${window.env.api}${url}`, "GET", undefined, JSON_HEADER);
}

// Raw fetch, which does not prefix with environment prefix for api
export function useFetch(url, method, cache, headers, body) {
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
