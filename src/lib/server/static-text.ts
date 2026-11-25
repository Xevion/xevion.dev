import publicKey from "./assets/publickey.asc?raw";
import keybaseProof from "./assets/keybase.txt?raw";

export { publicKey, keybaseProof };

export function pgpKeyResponse(): Response {
  return new Response(publicKey, {
    headers: {
      "content-type": "application/pgp-keys",
      "content-disposition": 'attachment; filename="publickey.asc"',
      "cache-control": "public, max-age=86400",
    },
  });
}

export function keybaseResponse(): Response {
  return new Response(keybaseProof, {
    headers: {
      "content-type": "text/plain; charset=utf-8",
      "cache-control": "public, max-age=3600",
    },
  });
}
