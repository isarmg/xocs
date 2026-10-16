export type CommentSubmission = { request_id: string; content: string; parent_comment_id: number | null; article_id?: number };
// Rust str::trim uses Unicode White_Space. JavaScript trim differs at U+0085
// and U+FEFF; submission checks and acknowledgement comparisons use this contract.
export const normalizeCommentContent = (value: string): string => value.replace(/^\p{White_Space}+|\p{White_Space}+$/gu, '');
const requestId = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
export const submissionKey = (path: string, articleId?: number) => `xocs.comment-submission:${path}:${articleId ?? 0}`;

export function loadSubmission(key: string): { submission: CommentSubmission | null; unreadable: boolean } {
  try {
    const raw = sessionStorage.getItem(key);
    if (raw === null) return { submission: null, unreadable: false };
    const value: unknown = JSON.parse(raw);
    const articleId = value && typeof value === 'object' && 'article_id' in value ? value.article_id : undefined;
    if (value && typeof value === 'object' && 'request_id' in value && typeof value.request_id === 'string' && requestId.test(value.request_id)
      && 'content' in value && typeof value.content === 'string' && normalizeCommentContent(value.content).length > 0
      && 'parent_comment_id' in value && (value.parent_comment_id === null || (typeof value.parent_comment_id === 'number' && Number.isSafeInteger(value.parent_comment_id) && value.parent_comment_id > 0))
      && (articleId === undefined || (typeof articleId === 'number' && Number.isSafeInteger(articleId) && articleId > 0))) {
      return { submission: { request_id: value.request_id, content: value.content, parent_comment_id: value.parent_comment_id, ...(typeof articleId === 'number' ? { article_id: articleId } : {}) }, unreadable: false };
    }
    return { submission: null, unreadable: true };
  } catch { return { submission: null, unreadable: true }; }
}

export function saveSubmission(key: string, submission: CommentSubmission): void {
  sessionStorage.setItem(key, JSON.stringify(submission));
}
export function clearSubmission(key: string): void {
  sessionStorage.removeItem(key);
}
