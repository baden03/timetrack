import { useState } from "react";
import { t } from "../i18n";
import { displayDate, formatClock, formatHhmm, parseClock, parseDate, parseDuration } from "../time";

/*
 * Date and time fields. While the person types, the text is left alone.
 * On blur it is checked and tidied into the chosen format (`2130` becomes
 * `21:30`); text that cannot be read is marked invalid and kept so it can be
 * fixed. The owner of the form keeps the text and parses it again when the
 * form is submitted, so pressing Enter straight after typing also works.
 */

interface Common {
  /** The text as typed (or as tidied after a blur). */
  value: string;
  onChange: (text: string) => void;
  required?: boolean;
  autoFocus?: boolean;
  "aria-label"?: string;
}

function TextField({
  value,
  onChange,
  tidy,
  placeholder,
  hint,
  ...rest
}: Common & {
  /** The tidied text, or null if the text cannot be read. */
  tidy: (text: string) => string | null;
  placeholder: string;
  hint: string;
}) {
  const [invalid, setInvalid] = useState(false);
  return (
    <input
      type="text"
      inputMode="numeric"
      placeholder={placeholder}
      value={value}
      required={rest.required}
      autoFocus={rest.autoFocus}
      aria-label={rest["aria-label"]}
      aria-invalid={invalid || undefined}
      title={invalid ? hint : undefined}
      onChange={(e) => {
        setInvalid(false);
        onChange(e.target.value);
      }}
      onBlur={() => {
        if (value.trim() === "") return setInvalid(false);
        const tidied = tidy(value);
        setInvalid(tidied === null);
        if (tidied !== null && tidied !== value) onChange(tidied);
      }}
    />
  );
}

/** A date field in the date format chosen in Settings. */
export function DateInput({ format, ...props }: Common & { format: string }) {
  return (
    <TextField
      {...props}
      placeholder={format}
      hint={t("ui.format.date_hint", { format })}
      tidy={(text) => {
        const iso = parseDate(text, format);
        return iso === null ? null : displayDate(iso, format);
      }}
    />
  );
}

/** A duration field: `:45`, `.75` or `1h30` are tidied into `0:45`, `0:45`, `1:30` on blur. */
export function DurationInput(props: Common) {
  return (
    <TextField
      {...props}
      placeholder="1:30"
      hint={t("ui.format.duration_hint")}
      tidy={(text) => {
        const minutes = parseDuration(text);
        return minutes === null ? null : formatHhmm(minutes);
      }}
    />
  );
}

/** A time field in the clock format chosen in Settings (`21:30` or `9:30 PM`). */
export function TimeInput({ clock24h, ...props }: Common & { clock24h: boolean }) {
  return (
    <TextField
      {...props}
      placeholder={clock24h ? "21:30" : "9:30 PM"}
      hint={t("ui.format.time_hint")}
      tidy={(text) => {
        const hhmm = parseClock(text);
        return hhmm === null ? null : formatClock(hhmm, clock24h);
      }}
    />
  );
}
