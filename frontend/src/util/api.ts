declare global {
  interface EnvApi {
    api: string
}
  interface Window {
    env: EnvApi;
  }
}

const JSON_HEADER = {
  'Content-Type': 'application/json',
};

function urlFor(url) {
  return `${window.env.api}${url}`
}


export async function Get(url) {
  const result = await fetch(urlFor(url), {
    method: 'GET',
    headers: JSON_HEADER,
  });
  return result.json();
}

export async function Put(url, body) {
  const result = await fetch(urlFor(url), {
    method: 'PUT',
    cache: 'no-cache',
    headers: JSON_HEADER,
    body: JSON.stringify(body),
  });
  return result.json();
}

export async function Delete(url) {
  const result = await fetch(urlFor(url), {
    method: 'DELETE',
    cache: 'no-cache',
    headers: JSON_HEADER,
  });
  return result.json();
}

export async function Post(url, body) {
  const result = await fetch(urlFor(url), {
    method: 'POST',
    cache: 'no-cache',
    headers: JSON_HEADER,
    body: JSON.stringify(body),
  });
  return result.json();
}
