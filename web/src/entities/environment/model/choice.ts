export const ENVIRONMENT_COOKIE = "portal_environment";

export const CHOICE_SECONDS = 31_536_000;

export function readChoice(): string | null {
  const pair = document.cookie
    .split(";")
    .map((entry) => entry.trim().split("="))
    .find(([name]) => name === ENVIRONMENT_COOKIE);
  return pair && pair[1] ? decodeURIComponent(pair[1]) : null;
}

export function writeChoice(environment: string) {
  document.cookie = `${ENVIRONMENT_COOKIE}=${encodeURIComponent(environment)}; Path=/; SameSite=Strict; Max-Age=${CHOICE_SECONDS}`;
}

export function clearChoice() {
  document.cookie = `${ENVIRONMENT_COOKIE}=; Path=/; SameSite=Strict; Max-Age=0`;
}
