export function copyId(id: string, taken: readonly string[]): string {
  let number = 2;
  while (taken.includes(`${id}-${number}`)) {
    number += 1;
  }
  return `${id}-${number}`;
}
