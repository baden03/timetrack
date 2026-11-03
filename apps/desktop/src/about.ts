import pkg from "../package.json";
import { inTauri } from "./api";

export const REPOSITORY_URL = "https://github.com/baden03/timetrack";
export const COPYRIGHT_YEAR = 2026;
export const COPYRIGHT_HOLDER = "Twinpictures";

/** The version of the running app (the build's own version inside Tauri). */
export async function appVersion(): Promise<string> {
  if (inTauri) {
    const { getVersion } = await import("@tauri-apps/api/app");
    return getVersion();
  }
  return pkg.version;
}

/** Opens a page in the default browser. Only the repository is permitted. */
export async function openRepository(): Promise<void> {
  if (inTauri) {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(REPOSITORY_URL);
  } else {
    window.open(REPOSITORY_URL, "_blank", "noopener");
  }
}
