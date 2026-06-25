import { createContext, useContext } from 'react';
import type { Dict } from './zh-CN';

export type Lang = 'zh-CN' | 'en-US';

export interface I18nContextValue {
  t: (key: keyof Dict) => string;
  lang: Lang;
  setLang: (lang: Lang) => void;
}

export const I18nContext = createContext<I18nContextValue>({
  t: (key) => String(key),
  lang: 'zh-CN',
  setLang: () => {},
});

export function useI18n() {
  return useContext(I18nContext);
}
