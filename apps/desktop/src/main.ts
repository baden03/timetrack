import { invoke } from "@tauri-apps/api/core";

// Spike only: proves the TypeScript UI can call the Rust core.
type Ping = { hhmm: string; id: string; keychain_ok: boolean };

async function main() {
  const app = document.getElementById("app")!;
  const ping = await invoke<Ping>("core_ping", { minutes: 7590 });
  app.textContent = `core says 7590 min = ${ping.hhmm}; id ${ping.id}; keychain ${ping.keychain_ok ? "ok" : "failed"}`;
}

main().catch((e) => {
  document.getElementById("app")!.textContent = String(e);
});
