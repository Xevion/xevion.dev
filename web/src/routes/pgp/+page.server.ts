import { PGP_KEY_METADATA } from '$lib/pgp/key-info';
import { readFileSync } from 'fs';
import { join } from 'path';

export const prerender = true;

export const load = () => {
  // Read the PGP key from static directory at build time
  const keyPath = join(process.cwd(), 'static', 'publickey.asc');
  const content = readFileSync(keyPath, 'utf-8');
  
  return {
    key: {
      ...PGP_KEY_METADATA,
      content,
    },
  };
};
