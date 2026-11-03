import { useState } from "react";
import { PALETTE, themedColor, useDark } from "../colors";
import { t } from "../i18n";

/** A small filled circle in a client's or project's color, adjusted to the theme. */
export function ColorDot({ color }: { color: string | null | undefined }) {
  const dark = useDark();
  if (!color) return null;
  return <span className="dot-color" aria-hidden="true" style={{ background: themedColor(color, dark) }} />;
}

/** The choice of a color, or none: a row of swatches. */
export function ColorField({
  value,
  onChange,
  legend,
}: {
  value: string | null | undefined;
  onChange: (color: string | null) => void;
  legend: string;
}) {
  const dark = useDark();
  const [name] = useState(() => `color-${Math.random().toString(36).slice(2)}`);
  return (
    <fieldset className="colors">
      <legend>{legend}</legend>
      <label className="swatch none">
        <input type="radio" name={name} checked={!value} onChange={() => onChange(null)} />
        <span>{t("ui.color.none")}</span>
      </label>
      {PALETTE.map((c) => (
        <label key={c.key} className="swatch" title={t(`color.${c.key}`)}>
          <input
            type="radio"
            name={name}
            checked={value === c.hex}
            onChange={() => onChange(c.hex)}
            aria-label={t(`color.${c.key}`)}
          />
          <span className="chip" style={{ background: themedColor(c.hex, dark) }} />
        </label>
      ))}
    </fieldset>
  );
}
