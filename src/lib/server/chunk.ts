/** D1 caps a statement at 100 bound parameters, so id lists are queried in slices. */
export const D1_ID_CHUNK = 90;

export function chunk<T>(items: T[], size = D1_ID_CHUNK): T[][] {
  const out: T[][] = [];
  for (let i = 0; i < items.length; i += size)
    out.push(items.slice(i, i + size));
  return out;
}
