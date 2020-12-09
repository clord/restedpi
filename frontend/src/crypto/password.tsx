
export function toHexString(byteArray: Uint8Array) {
  let result = ''
  for (let i = 0; i < byteArray.length; i += 1) {
    const b = byteArray[i];
    result += ('0' + b.toString(16)).slice(-2)
  }
  return result
}

export function toByteArray(hexString: string) {
  const result = [];
  for (let i = 0; i < hexString.length; i += 2) {
    result.push(parseInt(hexString.substr(i, 2), 16));
  }
  return result;
}

/**
 * Returns PBKDF2 derived key from supplied password together with version and salt
 */
export async function passwordHash(password: string, iterations=1e6): Promise<string> {
	const crypto = window.crypto;
	const pw = new TextEncoder().encode(password.normalize('NFKC'));
	const pwKey = await crypto.subtle.importKey('raw', pw, 'PBKDF2', false, ['deriveBits']);
	const salt = crypto.getRandomValues(new Uint8Array(16));
  const key = await crypto.subtle.deriveBits({
    name: 'PBKDF2',
    hash: 'SHA-256',
    salt,
    iterations,
  }, pwKey, 256);

	return [
		salt,
		Uint8Array.from([1]),
		new Uint8Array(key),
	].map(toHexString).join("_")
}
