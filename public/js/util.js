import { useState, useEffect } from './depend/preact.hooks.js';

/**
 * Post a body to a url
 */
export function usePost(url, body) {
    return useFetch(url, {
	    method: 'POST',
        cache: 'no-cache',
        headers: {
        	'Content-Type': 'application/json'
        },
        body: JSON.stringify(body)
	});
}

export function useFetch(url, options) {
  const [response, setResponse] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    const FetchData = async () => {
      try {
        const res = await fetch(url, options);
        const json = await res.json();
        setResponse(json);
      } catch (error) {
        setError(error);
      }
    };
    FetchData();
  }, []);
  return { response, error };
};
