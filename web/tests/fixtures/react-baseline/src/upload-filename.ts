// X-File-Name carries percent-encoded UTF-8; its metadata limit is 200 bytes.
export function uploadFilename(name: string): string {
  const encoder = new TextEncoder();
  let prefix = '';
  let size = 0;
  for (const character of name) {
    const bytes = encoder.encode(character).length;
    if (size + bytes > 200) break;
    prefix += character;
    size += bytes;
  }
  return encodeURIComponent(prefix);
}
