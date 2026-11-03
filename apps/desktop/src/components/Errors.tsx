import { CommandError } from "../api";
import { tError } from "../i18n";

export function errorText(e: unknown): string {
  return tError(e instanceof CommandError ? e.code : "storage");
}

export function ErrorLine({ message }: { message: string | null }) {
  return message ? (
    <p role="alert" className="error">
      {message}
    </p>
  ) : null;
}
