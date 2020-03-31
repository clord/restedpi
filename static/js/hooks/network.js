import { useState, useMemo, useEffect } from '/react/';

const JSON_HEADER = {
  'Content-Type': 'application/json',
};

export function usePost(url, body) {
  return useFetch(
    `${window.env.api}${url}`,
    'POST',
    'no-cache',
    JSON_HEADER,
    JSON.stringify(body)
  );
}

export function useGet(url) {
  return useFetch(`${window.env.api}${url}`, 'GET', undefined, JSON_HEADER);
}

export async function apiGet(url) {
  const result = await fetch(`${window.env.api}${url}`, {
    method: 'GET',
    headers: JSON_HEADER,
  });
  return result.json();
}

export async function apiPut(url, body) {
  const result = await fetch(`${window.env.api}${url}`, {
    method: 'PUT',
    cache: 'no-cache',
    headers: JSON_HEADER,
    body: JSON.stringify(body),
  });
  return result.json();
}

export async function apiDelete(url, value) {
  const result = await fetch(`${window.env.api}${url}`, {
    method: 'DELETE',
    cache: 'no-cache',
    headers: JSON_HEADER,
  });
  return result.json();
}

export async function apiPost(url, body) {
  const result = await fetch(`${window.env.api}${url}`, {
    method: 'POST',
    cache: 'no-cache',
    headers: JSON_HEADER,
    body: JSON.stringify(body),
  });
  return result.json();
}

export function useFetch(url, method, cache, headers, body) {
  const [response, setResponse] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    const FetchData = async () => {
      try {
        const res = await fetch(url, { method, cache, headers, body });
        const json = await res.json();
        setResponse(json);
      } catch (error) {
        setError(error);
      }
    };
    FetchData();
  }, [url, method, body]);
  return { response, error };
}
