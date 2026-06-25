/**
 * v0.4.17: stable country-flag icon via the flag-icons SVG/CSS library.
 *
 * The old countryFlag() emitted Unicode Regional-Indicator emoji (🇯🇵/🇭🇰),
 * which render reliably on Firefox but degrade to letters / blanks on
 * Edge + Chrome + Windows depending on the system Emoji font. flag-icons
 * ships its own SVGs bundled by Vite, so the flag renders the same
 * everywhere and works offline (no CDN).
 *
 * Structure: an outer `.country-flag-pill` (sizing/border) wraps an inner
 * `.fi.fi-xx` span. The two-layer structure keeps flag-icons' own `.fi`
 * sizing from leaking into the surrounding layout.
 */
interface Props {
  /** 2-letter ISO-3166 alpha-2 code, e.g. "JP", "hk". */
  code?: string | null;
}

const ISO_ALPHA2 = /^[A-Za-z]{2}$/;

/** A country flag rendered from a 2-letter ISO code, or "--" when unknown. */
export function CountryFlag({ code }: Props) {
  if (!code || !ISO_ALPHA2.test(code)) {
    return <span className="country-flag-pill"><span>--</span></span>;
  }
  const lower = code.toLowerCase();
  const upper = code.toUpperCase();
  return (
    <span className="country-flag-pill" title={upper}>
      <span className={`fi fi-${lower}`} />
    </span>
  );
}
