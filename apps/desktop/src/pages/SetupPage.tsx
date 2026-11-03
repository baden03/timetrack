import { useState } from "react";
import { call, type Category, type Client, type Project, type Rec, type Status } from "../api";
import type { Data } from "../App";
import { useDialogs } from "../components/Dialogs";
import { ErrorLine, errorText } from "../components/Errors";
import { t } from "../i18n";

type Kind = "client" | "category" | "project";

export function SetupPage({ data, reload }: { data: Data; reload: () => Promise<void> }) {
  const dialogs = useDialogs();
  const [error, setError] = useState<string | null>(null);
  const run = async (fn: () => Promise<unknown>) => {
    try {
      await fn();
      setError(null);
      await reload();
    } catch (e) {
      setError(errorText(e));
    }
  };
  const rename = async (kind: Kind, rec: Rec<{ name: string }>) => {
    const name = await dialogs.prompt(t("ui.setup.rename_prompt"), rec.payload.name);
    if (name !== null) await run(() => call(`rename_${kind}`, { id: rec.id, name }));
  };
  const remove = async (kind: Kind, rec: Rec<{ name: string }>) => {
    if (await dialogs.confirm(t("ui.setup.delete_confirm", { name: rec.payload.name })))
      await run(() => call(`delete_${kind}`, { id: rec.id }));
  };
  const toggle = (kind: Kind, rec: Rec<{ status: Status }>) =>
    run(() =>
      call(`set_${kind}_status`, {
        id: rec.id,
        status: rec.payload.status === "active" ? "archived" : "active",
      }),
    );

  return (
    <div>
      {dialogs.dialog}
      <h1>{t("ui.setup.title")}</h1>
      <ErrorLine message={error} />
      <div className="columns">
        <NamedList
          title={t("ui.setup.clients")}
          kind="client"
          items={data.clients}
          onAdd={(name) => run(() => call("create_client", { name }))}
          onRename={rename}
          onToggle={toggle}
          onDelete={remove}
        />
        <NamedList
          title={t("ui.setup.categories")}
          kind="category"
          items={data.categories}
          onAdd={(name) => run(() => call("create_category", { name }))}
          onRename={rename}
          onToggle={toggle}
          onDelete={remove}
        />
      </div>
      <ProjectsSection data={data} run={run} toggle={toggle} remove={remove} />
    </div>
  );
}

function NamedList<T extends Client | Category>(props: {
  title: string;
  kind: Kind;
  items: Rec<T>[];
  onAdd: (name: string) => Promise<void>;
  onRename: (kind: Kind, rec: Rec<T>) => Promise<void>;
  onToggle: (kind: Kind, rec: Rec<T>) => Promise<void>;
  onDelete: (kind: Kind, rec: Rec<T>) => Promise<void>;
}) {
  const [name, setName] = useState("");
  return (
    <section>
      <h2>{props.title}</h2>
      <form
        className="inline"
        onSubmit={async (e) => {
          e.preventDefault();
          await props.onAdd(name);
          setName("");
        }}
      >
        <input
          aria-label={t("ui.setup.name")}
          placeholder={t("ui.setup.name")}
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button type="submit">{t("ui.setup.add")}</button>
      </form>
      <ul className="plain">
        {props.items.length === 0 && <li className="muted">{t("ui.setup.none")}</li>}
        {props.items.map((rec) => (
          <li key={rec.id} className={rec.payload.status === "archived" ? "archived" : ""}>
            <span>{rec.payload.name}</span>
            <span className="actions">
              <button onClick={() => props.onRename(props.kind, rec)}>{t("ui.setup.rename")}</button>
              <button onClick={() => props.onToggle(props.kind, rec)}>
                {rec.payload.status === "active" ? t("ui.setup.archive") : t("ui.setup.restore")}
              </button>
              <button className="danger" onClick={() => props.onDelete(props.kind, rec)}>
                {t("ui.setup.delete")}
              </button>
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function ProjectsSection(props: {
  data: Data;
  run: (fn: () => Promise<unknown>) => Promise<void>;
  toggle: (kind: Kind, rec: Rec<{ status: Status }>) => Promise<void>;
  remove: (kind: Kind, rec: Rec<{ name: string }>) => Promise<void>;
}) {
  const { data, run } = props;
  const [editing, setEditing] = useState<string | null>(null); // project id or "new"
  const activeClients = data.clients.filter((c) => c.payload.status === "active");
  const clientName = (id: string) => data.clients.find((c) => c.id === id)?.payload.name ?? "";
  const current = data.projects.find((p) => p.id === editing);

  return (
    <section>
      <h2>{t("ui.setup.projects")}</h2>
      <button onClick={() => setEditing("new")} disabled={activeClients.length === 0}>
        {t("ui.setup.add_project")}
      </button>
      {activeClients.length === 0 && <p className="muted">{t("ui.setup.need_client")}</p>}
      {editing && (
        <ProjectForm
          key={editing}
          data={data}
          project={current}
          onCancel={() => setEditing(null)}
          onSave={(input) =>
            run(async () => {
              if (current) await call("update_project", { id: current.id, input });
              else await call("create_project", { input });
              setEditing(null);
            })
          }
        />
      )}
      <ul className="plain">
        {data.projects.length === 0 && <li className="muted">{t("ui.setup.none")}</li>}
        {data.projects.map((rec) => (
          <li key={rec.id} className={rec.payload.status === "archived" ? "archived" : ""}>
            <span>
              {clientName(rec.payload.client_id)} / {rec.payload.name}
            </span>
            <span className="actions">
              <button onClick={() => setEditing(rec.id)}>{t("ui.setup.edit")}</button>
              <button onClick={() => props.toggle("project", rec)}>
                {rec.payload.status === "active" ? t("ui.setup.archive") : t("ui.setup.restore")}
              </button>
              <button className="danger" onClick={() => props.remove("project", rec)}>
                {t("ui.setup.delete")}
              </button>
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function ProjectForm(props: {
  data: Data;
  project?: Rec<Project>;
  onSave: (input: object) => Promise<void>;
  onCancel: () => void;
}) {
  const { data, project } = props;
  const activeClients = data.clients.filter((c) => c.payload.status === "active");
  const [clientId, setClientId] = useState(project?.payload.client_id ?? activeClients[0]?.id ?? "");
  const [name, setName] = useState(project?.payload.name ?? "");
  const [description, setDescription] = useState(project?.payload.description ?? "");
  const [billable, setBillable] = useState(project?.payload.billable_default ?? true);
  const [cats, setCats] = useState<string[]>(project?.payload.category_ids ?? []);
  const categories = data.categories.filter(
    (c) => c.payload.status === "active" || cats.includes(c.id),
  );
  return (
    <form
      className="card"
      onSubmit={(e) => {
        e.preventDefault();
        void props.onSave({
          client_id: clientId,
          name,
          description,
          billable_default: billable,
          category_ids: cats,
        });
      }}
      onKeyDown={(e) => e.key === "Escape" && props.onCancel()}
    >
      <label>
        {t("ui.setup.client")}
        <select value={clientId} onChange={(e) => setClientId(e.target.value)}>
          {data.clients
            .filter((c) => c.payload.status === "active" || c.id === clientId)
            .map((c) => (
              <option key={c.id} value={c.id}>
                {c.payload.name}
              </option>
            ))}
        </select>
      </label>
      <label>
        {t("ui.setup.name")}
        <input autoFocus value={name} onChange={(e) => setName(e.target.value)} />
      </label>
      <label>
        {t("ui.setup.description")}
        <input value={description} onChange={(e) => setDescription(e.target.value)} />
      </label>
      <label className="check">
        <input type="checkbox" checked={billable} onChange={(e) => setBillable(e.target.checked)} />
        {t("ui.setup.billable_default")}
      </label>
      <fieldset>
        <legend>{t("ui.setup.enabled_categories")}</legend>
        {categories.length === 0 && <span className="muted">{t("ui.setup.need_category")}</span>}
        {categories.map((c) => (
          <label key={c.id} className="check">
            <input
              type="checkbox"
              checked={cats.includes(c.id)}
              onChange={(e) =>
                setCats(e.target.checked ? [...cats, c.id] : cats.filter((x) => x !== c.id))
              }
            />
            {c.payload.name}
          </label>
        ))}
      </fieldset>
      <div className="actions">
        <button type="submit">{t("ui.save")}</button>
        <button type="button" onClick={props.onCancel}>
          {t("ui.cancel")}
        </button>
      </div>
    </form>
  );
}
