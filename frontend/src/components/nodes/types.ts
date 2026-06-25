import type { useI18n } from '../../i18n/context';

/** The i18n t() function type, shared across node components. */
export type Tfn = ReturnType<typeof useI18n>['t'];
