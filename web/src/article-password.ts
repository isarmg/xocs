import { t } from '@xcss/web/admin-ui/i18n';

// The administrator password primitive and article API count UTF-8 bytes.
export function articlePasswordError(value:string|null, allowEmpty:boolean):string|null {
  if (!value) return allowEmpty ? null : t('请设置文章访问密码。','Set an article access password.');
  const bytes = new TextEncoder().encode(value).length;
  if (bytes < 12 || bytes > 1024 || /[\u0000-\u001f\u007f\ud800-\udfff]/u.test(value)) {
    return t('密码须为 12–1024 个 UTF-8 字节，不能包含 ASCII 控制字符。','Use 12–1024 UTF-8 bytes without ASCII control characters.');
  }
  return null;
}
