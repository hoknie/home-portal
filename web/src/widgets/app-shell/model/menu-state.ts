export const MENU_KEY = "home-portal.menu-collapsed";

export function readCollapsed(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  try {
    return window.localStorage.getItem(MENU_KEY) === "1";
  } catch {
    return false;
  }
}

export function writeCollapsed(collapsed: boolean) {
  try {
    if (collapsed) {
      window.localStorage.setItem(MENU_KEY, "1");
    } else {
      window.localStorage.removeItem(MENU_KEY);
    }
  } catch {
    return;
  }
}
