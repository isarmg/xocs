import { t } from '@xcss/web/admin-ui/i18n';
import { useEffect, useRef, useState, type FormEvent } from 'react';
import { ApiError, publicErrorMessage, request, type Page } from './api';
import { isPublicComment, isPublicCommentPage, type PublicComment } from './public-contracts';
import { clearSubmission, loadSubmission, normalizeCommentContent, saveSubmission, submissionKey, type CommentSubmission } from './comment-submission';
import { safeImageUrl } from './MediaPreview';

type CommentKind = 'article' | 'message' | 'love';
const commentPath = (kind: CommentKind) => kind === 'article' ? '/api/v1/comments' : kind === 'love' ? '/api/v1/love-comments' : '/api/v1/message-comments';
const commenter = (item: PublicComment) => item.username || t('匿名访客', 'Anonymous guest');

function CommentEntry({ item, onReply }: { item: PublicComment; onReply: (item: PublicComment) => void }) {
  const avatar = safeImageUrl(item.avatar);
  return <article className="public-comment"><div className="public-comment-head">
    {avatar && <img src={avatar} alt="" />}<div><strong>{commenter(item)}</strong><small>{item.create_time || t('最近', 'Recently')}</small></div>
  </div><p>{item.parent_username && <span className="comment-mention">@{item.parent_username} </span>}{item.comment_content}</p>
    <div className="comment-actions"><button type="button" onClick={() => onReply(item)}>{t('回复', 'Reply')}</button></div>
  </article>;
}

function CommentReplies({ root, refresh, kind, onReply }: { root: PublicComment; refresh: number; kind: CommentKind; onReply: (item: PublicComment) => void }) {
  const [page, setPage] = useState(1);
  const [result, setResult] = useState<Page<PublicComment> | null>(null);
  const [items, setItems] = useState<PublicComment[]>([]);
  const [failure, setFailure] = useState('');
  const [busy, setBusy] = useState(false), [attempt, setAttempt] = useState(0);
  const loading = useRef(false), loadedPage = useRef(0);
  useEffect(() => { loadedPage.current = 0; setPage(1); setItems([]); setResult(null); }, [root.id, root.reply_count, refresh, kind]);
  useEffect(() => {
    if (!root.reply_count) { loading.current = false; setBusy(false); setItems([]); setResult(null); return; }
    const controller = new AbortController(); loading.current = true; setBusy(true); setFailure('');
    void request(`${commentPath(kind)}/${root.id}/replies?page=${page}&size=5`, isPublicCommentPage, { signal: controller.signal })
      .then(value => { if (!controller.signal.aborted) { loadedPage.current = page; setResult(value); setItems(current => page === 1 ? value.items : [...current, ...value.items.filter(item => !current.some(existing => existing.id === item.id))]); } })
      .catch(reason => { if (!controller.signal.aborted) setFailure(publicErrorMessage(reason, t('回复加载失败', 'Unable to load replies'))); })
      .finally(() => { if (!controller.signal.aborted) { loading.current = false; setBusy(false); } });
    return () => controller.abort();
  }, [root.id, root.reply_count, page, refresh, kind, attempt]);
  function loadMore() {
    if (loading.current) return;
    loading.current = true; setBusy(true); setPage(loadedPage.current + 1); setAttempt(value => value + 1);
  }
  if (!root.reply_count) return null;
  return <div className="comment-replies">{failure && <p role="alert">{failure} <button type="button" disabled={busy} onClick={loadMore}>{t('重试', 'Try again')}</button></p>}
    {items.map(item => <CommentEntry key={item.id} item={item} onReply={onReply} />)}
    {result && items.length < result.total && <button type="button" className="comment-more" disabled={busy} onClick={loadMore}>{t('展开剩余 {0} 条回复', 'Show {0} more replies', [result.total - items.length])}</button>}
  </div>;
}

type CommentProps = { articleId?: number; kind?: CommentKind; title?: string };
export function PublicComments(props: CommentProps) {
  return <CommentThread key={`${props.kind ?? 'article'}:${props.articleId ?? 0}`} {...props} />;
}

export function CommentThread({ articleId, kind = 'article', title }: CommentProps) {
  const path = commentPath(kind);
  const storageKey = submissionKey(path, articleId);
  const [saved] = useState(() => loadSubmission(storageKey));
  const [pending, setPending] = useState<CommentSubmission | null>(saved.submission);
  const [storageFailure, setStorageFailure] = useState(saved.unreadable);
  const submitting = useRef(false);
  const pendingRef = useRef(saved.submission);
  const [comments, setComments] = useState<Page<PublicComment> | null>(null);
  const [draft, setDraft] = useState(saved.submission?.content ?? ''), [replyTo, setReplyTo] = useState<PublicComment | null>(null);
  const [failure, setFailure] = useState(''), [feedback, setFeedback] = useState('');
  const [loadFailure, setLoadFailure] = useState(''), [loading, setLoading] = useState(true), [loadAttempt, setLoadAttempt] = useState(0);
  const [refresh, setRefresh] = useState(0), [page, setPage] = useState(1), [busy, setBusy] = useState(false);
  useEffect(() => {
    const controller = new AbortController(); const filter = kind === 'article' ? `&article_id=${articleId}` : '';
    setLoading(true); setLoadFailure('');
    void request(`${path}?page=${page}&size=10${filter}`, isPublicCommentPage, { signal: controller.signal })
      .then(value => { if (!controller.signal.aborted) setComments(value); })
      .catch(reason => { if (!controller.signal.aborted) setLoadFailure(publicErrorMessage(reason, t('评论加载失败', 'Unable to load comments'))); })
      .finally(() => { if (!controller.signal.aborted) setLoading(false); });
    return () => controller.abort();
  }, [articleId, kind, path, page, refresh, loadAttempt]);
  async function post(event: FormEvent) {
    event.preventDefault();
    if (submitting.current || storageFailure || (!pendingRef.current && !normalizeCommentContent(draft))) return;
    submitting.current = true; setBusy(true); setFailure(''); setFeedback('');
    const retry = pendingRef.current !== null;
    try {
      const submission = pendingRef.current ?? { request_id: crypto.randomUUID(), ...(kind === 'article' ? { article_id: articleId } : {}), content: normalizeCommentContent(draft), parent_comment_id: replyTo?.id ?? null };
      // Persist before sending. An unknown result keeps this exact identity and
      // payload across retries and reloads in this tab; edits cannot replace it.
      try { saveSubmission(storageKey, submission); }
      catch { setStorageFailure(true); return; }
      pendingRef.current = submission; setPending(submission);
      await request(path, (value): value is PublicComment => isPublicComment(value) && value.comment_content === normalizeCommentContent(submission.content), { method: 'POST', body: JSON.stringify(submission) });
      try { clearSubmission(storageKey); } catch { /* Replaying a saved receipt remains safe. */ }
      pendingRef.current = null; setPending(null);
      setDraft(''); setReplyTo(null); setPage(1); setRefresh(value => value + 1);
      setFeedback(t('评论已发表。', 'Your comment has been posted.'));
    } catch (reason) {
      const rejected = !retry && reason instanceof ApiError && reason.envelope !== null
        && [400, 401, 403, 404, 409, 413, 429].includes(reason.status);
      if (rejected) {
        try { clearSubmission(storageKey); } catch { /* The rejected identity cannot duplicate a write. */ }
        pendingRef.current = null; setPending(null);
        setFailure(publicErrorMessage(reason));
      } else {
        setFailure(t('评论提交结果尚未确认。请确认原提交；重试会使用同一请求，不会重复发表。', 'Your comment submission is not yet confirmed. Confirm the original submission; retrying uses the same request and will not post it twice.'));
      }
    } finally { submitting.current = false; setBusy(false); }
  }
  function abandon() {
    if (busy || !window.confirm(t('这条评论可能已经发表。请先检查评论列表；确定放弃确认原提交并清空草稿？', 'This comment may already be posted. Check the comments first. Stop confirming this submission and clear its draft?'))) return;
    try { clearSubmission(storageKey); }
    catch { setStorageFailure(true); return; }
    pendingRef.current = null; setPending(null); setDraft(''); setReplyTo(null); setFailure(''); setStorageFailure(false);
  }
  const chooseReply = (item: PublicComment) => { if (!busy && !pending && !storageFailure) setReplyTo(item); };
  const visibleComments = comments?.page === page ? comments : null;
  return <section className="article-comments"><h3>{title || t('评论', 'Comments')} | {t('{0} 条评论', '{0} comments', [comments?.total ?? 0])}</h3>
    {failure && <p role="alert">{failure}</p>}
    {loadFailure && <p role="alert">{loadFailure} <button type="button" disabled={loading} onClick={() => setLoadAttempt(value => value + 1)}>{t('重试', 'Try again')}</button></p>}
    {loading && <p role="status">{t('评论加载中…', 'Loading comments…')}</p>}
    {storageFailure && <p role="alert">{t('无法读取或保存待确认的提交。请先检查评论，再清除待确认状态；当前不会发送新评论。', 'The pending submission could not be read or saved. Check the comments before clearing it. No new comment will be sent.')}</p>}
    {pending && !failure && <p role="status">{t('有一条评论等待确认，请继续确认原提交。', 'A comment is awaiting confirmation. Continue confirming the original submission.')}</p>}
    {visibleComments?.items.map(item => <div className="comment-floor" key={item.id}><CommentEntry item={item} onReply={chooseReply} /><CommentReplies root={item} refresh={refresh} kind={kind} onReply={chooseReply} /></div>)}
    {visibleComments?.items.length === 0 && <p className="muted">{t('还没有评论。', 'No comments yet.')}</p>}
    {comments && comments.total > 10 && <nav className="pager" aria-label={t('评论分页', 'Comment pages')}><button type="button" disabled={busy || page <= 1} onClick={() => setPage(value => value - 1)}>{t('上一页', 'Previous page')}</button><span>{t('第 {0} 页', 'Page {0}', [page])}</span><button type="button" disabled={busy || page * 10 >= comments.total} onClick={() => setPage(value => value + 1)}>{t('下一页', 'Next page')}</button></nav>}
    <form className="comment-form" onSubmit={event => void post(event)}>
      <p className="comment-identity">{t('无需登录，以匿名访客身份发表评论。', 'Post a comment as an anonymous guest. No account is needed.')}</p>
      {replyTo && <button type="button" className="text-action" disabled={busy || !!pending} onClick={() => setReplyTo(null)}>{t('回复 {0} · 取消', 'Reply to {0} · Cancel', [commenter(replyTo)])}</button>}
      <textarea disabled={busy || !!pending || storageFailure} aria-label={t('写下你的想法', 'Write your thoughts')} required maxLength={1024} value={draft} onChange={event => setDraft(event.target.value)} placeholder={t('写下你的想法', 'Write your thoughts')} />
      <button type="submit" disabled={busy || storageFailure || (!pending && !normalizeCommentContent(draft))}>{busy ? t('发表中…', 'Posting…') : pending ? t('确认原提交', 'Confirm original submission') : t('匿名发表评论', 'Post anonymous comment')}</button>
      {(pending || storageFailure) && <button type="button" disabled={busy} onClick={abandon}>{t('清除待确认状态', 'Clear pending submission')}</button>}
      {feedback && <p role="status">{feedback}</p>}
    </form>
  </section>;
}
