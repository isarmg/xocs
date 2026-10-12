import { t } from '@xcss/web/admin-ui/i18n';
export type ArticleSummary = {id: number; article_title: string; article_cover: string | null; sort_id: number; label_id: number; sort_name: string | null; label_name: string | null; view_count: number; like_count: number; comment_count: number; recommend_status: number; view_status: number; create_time: string | null; excerpt: string | null; search_snippet: string | null};
import {isErrorEnvelope, type ErrorEnvelope} from '@xcss/web/contracts';

export class ApiError extends Error {
  readonly status: number;
  readonly envelope: ErrorEnvelope | null;
  constructor(status: number, envelope: ErrorEnvelope | null) {
    super(publicErrorMessage({status}));
    this.name = 'ApiError';
    this.status = status;
    this.envelope = envelope;
  }
}

/** A successful HTTP response did not prove the requested operation's result. */
export class ApiContractError extends Error {
  readonly status: number;
  readonly reason: 'invalid_json' | 'invalid_response';
  constructor(status: number, reason: 'invalid_json' | 'invalid_response') {
    super(t('服务器响应格式无效，无法确认操作结果。', 'The server response was invalid. The operation result could not be confirmed.'));
    this.name = 'ApiContractError';
    this.status = status;
    this.reason = reason;
  }
}

/** Only locally authored validation text may be shown verbatim. */
export class DisplayError extends Error {}
export function publicErrorMessage(error: unknown, fallback = t("请求失败，请重试。", "Request failed. Please retry.")): string {
  if (error instanceof DisplayError || error instanceof ApiContractError) return error.message;
  const status = error && typeof error === 'object' && 'status' in error ? error.status : undefined;
  if (status === 400) return t("请检查输入内容后重试。", "Check the input and try again.");
  if (status === 401) return t("请检查登录信息，或重新登录后重试。", "Check your credentials or sign in again.");
  if (status === 403) return t("没有执行此操作的权限。", "You do not have permission to perform this action.");
  if (status === 404) return t("内容不存在或已被删除。", "This content was not found or has been deleted.");
  if (status === 409) return t("内容或状态已变化，请刷新后重试。", "The content or state has changed. Refresh and try again.");
  if (status === 413) return t("上传内容过大，请缩小后重试。", "The upload is too large. Reduce its size and try again.");
  if (status === 429) return t("请求过于频繁，请稍后重试。", "Too many requests. Please try again later.");
  if (typeof status === 'number' && status >= 500) return t("服务暂时不可用，请稍后重试。", "The service is temporarily unavailable. Please try again later.");
  return fallback;
}

export type Article = Omit<ArticleSummary, 'excerpt' | 'search_snippet'> & {user_id: number; article_content: string; video_url: string | null; username: string | null; sort_name: string | null; label_name: string | null; comment_status: number; password_required: number; tips: string | null; update_time: string | null};
export type Category = {id: number; sort_name: string; sort_description: string | null; priority: number | null; article_count: number};
export type PublicLabel = {id: number; sort_id: number; label_name: string; label_description: string | null; article_count: number};
export type SiteInfo = {web_name: string | null; web_title: string | null; notices: string | null; footer: string | null; background_image: string | null; avatar: string | null; random_cover: string | null};
export type Page<T> = {items: T[]; total: number; page: number; size: number};
export function articleExcerpt(value: string | null | undefined, limit = 115): string {
  if (!value) return '';
  const plain = value.replace(/!\[[^\]]*\]\([^)]*\)/g, ' ').replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/<[^>]+>/g, ' ').replace(/[#>*_`~|]/g, ' ').replace(/\s+/g, ' ').trim();
  return plain.length > limit ? `${plain.slice(0, limit).trimEnd()}…` : plain;
}
export type ArticleInput = {article_title: string; article_content: string; article_cover: string | null; video_url: string | null; sort_id: number; label_id: number; view_status: boolean; recommend_status: boolean; comment_status: boolean; password: string | null; tips: string | null};

export type ResponseValidator<T> = (value: unknown) => value is T;

export async function request<T>(path: string, validate: ResponseValidator<T>, options: RequestInit = {}, csrf?: string): Promise<T> {
  if (typeof validate !== 'function') throw new TypeError('A response validator is required.');
  const headers = new Headers(options.headers);
  if (options.body !== undefined && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');
  if (csrf && options.method && !['GET','HEAD'].includes(options.method.toUpperCase())) headers.set('X-CSRF-Token', csrf);
  const response = await fetch(path, {...options, headers, credentials: 'same-origin', redirect: 'error'});
  const data: unknown = await response.json().catch(() => {
    if (response.ok) throw new ApiContractError(response.status, 'invalid_json');
    throw new ApiError(response.status, null);
  });
  if (!response.ok) {
    throw new ApiError(response.status, isErrorEnvelope(data) ? data : null);
  }
  if (!validate(data)) throw new ApiContractError(response.status, 'invalid_response');
  return data;
}
