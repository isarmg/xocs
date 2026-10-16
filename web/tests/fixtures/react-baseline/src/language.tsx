import { getLocale, languageLabel, switchLanguage, t, validationMessage } from '@xcss/web/admin-ui/i18n';
import type { FormEvent, MouseEvent } from 'react';

export function LanguageControl() {
  return <button type="button" aria-label={languageLabel()} title={languageLabel()} onClick={switchLanguage}>
    <svg aria-hidden="true" width="1em" height="1em" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7"><path d="M3 5h12M9 3v2M6 5c0 5 4 9 8 11M13 5c0 5-4 9-9 12m10 4 4-10 4 10m-6.5-4h5" /></svg>
  </button>;
}

/** Carry an explicit choice across local pages when storage is unavailable. */
export function languageHref(href: string): string {
  const url = new URL(href, window.location.href);
  if (url.origin !== window.location.origin || !['http:', 'https:'].includes(url.protocol)) return href;
  if (new URL(window.location.href).searchParams.has('lang')) url.searchParams.set('lang', getLocale());
  return url.pathname + url.search + url.hash;
}

export function preserveLinkLanguage(event: MouseEvent<HTMLElement>) {
  if (!(event.target instanceof Element)) return;
  const link = event.target.closest<HTMLAnchorElement>('a[href]');
  if (!link || link.hasAttribute('download') || link.getAttribute('href')?.startsWith('#')) return;
  link.href = languageHref(link.href);
}

export function localizeValidation(event: FormEvent<HTMLElement>) {
  const input = event.target;
  if (!(input instanceof HTMLInputElement || input instanceof HTMLSelectElement || input instanceof HTMLTextAreaElement) || input.validity.customError) return;
  const message = input instanceof HTMLTextAreaElement
    ? input.validity.valueMissing ? t('请填写此项。', 'Please fill out this field.')
      : input.validity.tooLong ? t('输入内容过长。', 'The value is too long.')
      : input.validity.tooShort ? t('输入内容过短。', 'The value is too short.')
      : t('请检查输入内容。', 'Please check this value.')
    : validationMessage(input);
  input.setCustomValidity(message);
}
export function clearValidation(event: FormEvent<HTMLElement>) {
  const input = event.target;
  if (input instanceof HTMLInputElement || input instanceof HTMLSelectElement || input instanceof HTMLTextAreaElement) input.setCustomValidity('');
}
