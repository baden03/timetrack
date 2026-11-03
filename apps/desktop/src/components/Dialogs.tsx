import { useEffect, useRef, useState, type ReactNode } from "react";
import { t } from "../i18n";

type Request =
  | { kind: "confirm"; message: string; danger: boolean; resolve: (v: boolean) => void }
  | { kind: "prompt"; message: string; initial: string; resolve: (v: string | null) => void };

/**
 * In-app replacements for confirm() and prompt(), which embedded macOS web
 * views do not reliably show.
 */
export function useDialogs(): {
  confirm: (message: string, danger?: boolean) => Promise<boolean>;
  prompt: (message: string, initial: string) => Promise<string | null>;
  dialog: ReactNode;
} {
  const [request, setRequest] = useState<Request | null>(null);
  return {
    confirm: (message, danger = true) =>
      new Promise((resolve) => setRequest({ kind: "confirm", message, danger, resolve })),
    prompt: (message, initial) =>
      new Promise((resolve) => setRequest({ kind: "prompt", message, initial, resolve })),
    dialog: request && <DialogView request={request} close={() => setRequest(null)} />,
  };
}

function DialogView({ request, close }: { request: Request; close: () => void }) {
  const ref = useRef<HTMLDialogElement>(null);
  const [text, setText] = useState(request.kind === "prompt" ? request.initial : "");
  useEffect(() => {
    ref.current?.showModal();
  }, []);

  const finish = (ok: boolean) => {
    ref.current?.close(); // returns focus to whatever opened the dialog
    if (request.kind === "confirm") request.resolve(ok);
    else request.resolve(ok ? text : null);
    close();
  };

  return (
    <dialog ref={ref} className="dialog" onCancel={(e) => { e.preventDefault(); finish(false); }}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          finish(true);
        }}
      >
        <p>{request.message}</p>
        {request.kind === "prompt" && (
          <input autoFocus value={text} onChange={(e) => setText(e.target.value)} />
        )}
        <div className="actions">
          <button
            type="submit"
            autoFocus={request.kind === "confirm"}
            className={request.kind === "confirm" && request.danger ? "danger" : ""}
          >
            {t("ui.ok")}
          </button>
          <button type="button" onClick={() => finish(false)}>
            {t("ui.cancel")}
          </button>
        </div>
      </form>
    </dialog>
  );
}
