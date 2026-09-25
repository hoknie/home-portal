export const ID_LIMIT = 63;

const CYRILLIC: Record<string, string> = {
  а: "a",
  б: "b",
  в: "v",
  г: "g",
  д: "d",
  е: "e",
  ё: "e",
  ж: "zh",
  з: "z",
  и: "i",
  й: "y",
  к: "k",
  л: "l",
  м: "m",
  н: "n",
  о: "o",
  п: "p",
  р: "r",
  с: "s",
  т: "t",
  у: "u",
  ф: "f",
  х: "kh",
  ц: "ts",
  ч: "ch",
  ш: "sh",
  щ: "shch",
  ъ: "",
  ы: "y",
  ь: "",
  э: "e",
  ю: "yu",
  я: "ya",
};

function trimmed(value: string) {
  return value.replace(/^[^a-z]+/, "").replace(/-+$/, "");
}

export function slugOf(name: string) {
  const latin = [...name.toLocaleLowerCase("ru")].map((letter) => CYRILLIC[letter] ?? letter).join("");
  const dashed = trimmed(latin.replace(/[^a-z0-9]+/g, "-"));
  return trimmed(dashed.slice(0, ID_LIMIT));
}

export function uniqueId(base: string, taken: Iterable<string>) {
  if (base === "") {
    return "";
  }
  const used = new Set(taken);
  if (!used.has(base)) {
    return base;
  }
  for (let number = 2; ; number += 1) {
    const suffix = `-${number}`;
    const candidate = `${trimmed(base.slice(0, ID_LIMIT - suffix.length))}${suffix}`;
    if (!used.has(candidate)) {
      return candidate;
    }
  }
}
