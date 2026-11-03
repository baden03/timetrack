import { inTauri } from "./api";
import { t } from "./i18n";

/** Asks where to save a backup. Returns a path, or null if cancelled. */
type AskPath = (message: string, initial: string) => Promise<string | null>;

export async function chooseBackupSavePath(defaultName: string, ask: AskPath): Promise<string | null> {
  if (inTauri) {
    const { save } = await import("@tauri-apps/plugin-dialog");
    return save({ defaultPath: defaultName, filters: [{ name: "TimeTrack backup", extensions: ["sqlite"] }] });
  }
  // A plain browser (development only) has no file dialog.
  return ask(t("ui.backup.path_prompt"), `/tmp/${defaultName}`);
}

/** Asks which backup to restore. Returns a path, or null if cancelled. */
export async function chooseBackupOpenPath(ask: AskPath): Promise<string | null> {
  if (inTauri) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const picked = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "TimeTrack backup", extensions: ["sqlite"] }],
    });
    return typeof picked === "string" ? picked : null;
  }
  return ask(t("ui.backup.path_prompt"), "/tmp/");
}
