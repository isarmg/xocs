import { useId, useRef, type InputHTMLAttributes, type RefObject } from 'react';
import { Button } from '@xcss/web/admin-ui';
import { t } from '@xcss/web/admin-ui/i18n';

type FilePickerProps = Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'value' | 'children'> & {
  label: string;
  filename?: string;
  inputRef?: RefObject<HTMLInputElement | null>;
};

/** Keep the native file dialog and change event behind a shared, painted button. */
export function FilePicker({ label, filename = '', inputRef, disabled, id, ...props }: FilePickerProps) {
  const generatedId = useId();
  const localRef = useRef<HTMLInputElement | null>(null);
  const fieldRef = inputRef ?? localRef;
  const inputId = id ?? generatedId;
  const filenameId = `${inputId}-filename`;
  return <div className="xcss-form-field">
    <label htmlFor={inputId}>{label}</label>
    <div className="xocs-file-picker">
      <input {...props} id={inputId} ref={fieldRef} type="file" hidden disabled={disabled} />
      <Button disabled={disabled} aria-label={t('选择文件：{0}', 'Choose file: {0}', [label])}
        aria-describedby={filenameId} onClick={() => fieldRef.current?.click()}>
        {t('选择文件', 'Choose file')}
      </Button>
      <span className="xocs-file-picker-name" id={filenameId} aria-live="polite">
        {filename || t('未选择文件', 'No file chosen')}
      </span>
    </div>
  </div>;
}
