import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/react';
import { CountryFlag } from './CountryFlag';

/** Convenience: render and return the outer pill element. */
function pill(code?: string | null) {
  const { container } = render(<CountryFlag code={code} />);
  const el = container.querySelector('.country-flag-pill');
  if (!el) throw new Error('pill wrapper not rendered');
  return el;
}

describe('CountryFlag — valid 2-letter codes', () => {
  it('lowercase "jp" → fi-jp', () => {
    expect(pill('jp').querySelector('.fi')?.className).toContain('fi-jp');
  });

  it('uppercase "HK" → fi-hk, title = HK', () => {
    const el = pill('HK');
    expect(el.querySelector('.fi')?.className).toContain('fi-hk');
    expect(el.getAttribute('title')).toBe('HK');
  });

  it('"US" → fi-us', () => {
    expect(pill('US').querySelector('.fi')?.className).toContain('fi-us');
  });
});

describe('CountryFlag — invalid codes render "--"', () => {
  it.each([
    ['null', null],
    ['undefined', undefined],
    ['empty', ''],
    ['three letters "USA"', 'USA'],
    ['digits "12"', '12'],
    ['one letter "U"', 'U'],
  ])('renders "--" for %s', (_label, code) => {
    expect(pill(code).textContent).toBe('--');
  });
});

describe('CountryFlag — no emoji', () => {
  it('never renders regional-indicator emoji code points', () => {
    const { container } = render(
      <>
        <CountryFlag code="JP" />
        <CountryFlag code="HK" />
        <CountryFlag code={null} />
      </>,
    );
    // Regional indicators are U+1F1E6..U+1F1FF. No rendered text may contain them.
    const text = container.textContent || '';
    for (let cp = 0x1f1e6; cp <= 0x1f1ff; cp += 1) {
      expect(text).not.toContain(String.fromCodePoint(cp));
    }
  });
});

describe('CountryFlag — structure', () => {
  it('wraps the .fi span in a .country-flag-pill (two layers)', () => {
    const el = pill('US');
    expect(el.classList.contains('country-flag-pill')).toBe(true);
    const fi = el.querySelector('.fi.fi-us');
    expect(fi).not.toBeNull();
    // the flag is the pill's child, not a sibling — pill + fi is two layers
    expect(fi?.parentElement).toBe(el);
  });
});
