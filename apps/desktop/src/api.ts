import { invoke } from "@tauri-apps/api/core";

export type Status = "active" | "archived";
export type Id = string;

export interface Rec<T> {
  id: Id;
  kind: string;
  revision: number;
  deleted: boolean;
  created_at: string;
  updated_at: string;
  payload: T;
}

export interface Client { name: string; status: Status }
export interface Category { name: string; status: Status }
export interface Project {
  client_id: Id;
  name: string;
  description: string;
  billable_default: boolean;
  category_ids: Id[];
  status: Status;
}
export interface Settings {
  language: string;
  timezone: string;
  date_format: string;
  clock_24h: boolean;
  theme: "system" | "light" | "dark";
  entry_window_days: number;
}
export interface TimeEntry {
  project_id: Id;
  category_id: Id | null;
  description: string;
  billable: boolean;
  tags: string[];
  kind: "interval" | "duration";
  work_date: string;
  timezone: string;
  minutes: number;
  start_at: string | null;
  end_at: string | null;
  group_id: string;
  piece_index: number;
  piece_count: number;
  span_start_at: string | null;
  span_end_at: string | null;
}
export interface AppState {
  first_run: boolean;
  identity: { origin: string; author: string; device: string };
  settings: Rec<Settings>;
}

export type Timing =
  | { mode: "interval"; date: string; start: string; end: string }
  | { mode: "duration"; date: string; minutes: number };

export interface NewEntry {
  project_id: Id;
  category_id?: Id | null;
  description: string;
  billable?: boolean;
  tags: string[];
  timing: Timing;
}
export interface EntryPatch {
  project_id?: Id;
  /** Absent: unchanged. null: remove the category. */
  category_id?: Id | null;
  description?: string;
  billable?: boolean;
  tags?: string[];
  timing?: Timing;
}
export interface SavedEntry { pieces: Rec<TimeEntry>[]; overlaps: string[] }
export interface Suggestion { start: string | null; category_id: Id | null }

export interface Timer {
  project_id: Id;
  category_id: Id | null;
  description: string;
  billable: boolean | null;
  tags: string[];
  /** The start to the minute (what the entry will use). */
  start_at: string;
  /** When the timer actually started; the elapsed display counts from here. */
  exact_start_at?: string | null;
  timezone: string;
}
export interface TimerState { timer: Rec<Timer> | null; stale: boolean }
export interface SearchQuery {
  text?: string;
  client_id?: Id;
  project_id?: Id;
  category_id?: Id;
  tag?: string;
  billable?: boolean;
  from?: string;
  to?: string;
}
export interface SearchResult { pieces: Rec<TimeEntry>[]; total_minutes: number }
export interface HistoryEntry {
  record_id: Id;
  revision: number;
  op: "create" | "revise" | "delete";
  at: string;
  before: Record<string, unknown> | null;
  after: Record<string, unknown> | null;
}

export interface ProjectCard {
  project: Rec<Project>;
  client_name: string;
  last_activity: string;
  running: boolean;
}
export interface ProjectOverview {
  today_minutes: number;
  week_minutes: number;
  month_minutes: number;
  total_minutes: number;
}

export class CommandError extends Error {
  constructor(public code: string) {
    super(code);
  }
}

export const inTauri = "__TAURI_INTERNALS__" in window;

/** Development bridge (crates/app/examples/dev_bridge.rs) for plain browsers. */
async function callBridge<T>(command: string, args: object): Promise<T> {
  const response = await fetch("http://127.0.0.1:1421/call", {
    method: "POST",
    body: JSON.stringify({ command, args }),
  });
  const body = await response.json();
  if (!response.ok) throw body;
  return body as T;
}

export async function call<T>(command: string, args: object = {}): Promise<T> {
  try {
    return inTauri ? await invoke<T>("call", { command, args }) : await callBridge<T>(command, args);
  } catch (e) {
    const code = (e as { code?: string })?.code;
    throw new CommandError(code ?? "storage");
  }
}
