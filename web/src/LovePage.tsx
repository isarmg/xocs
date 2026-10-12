import { formatUtcTime, parseUtcDate } from './utc-time.ts';
import { isFamilies, isLinkClasses, isLinkPage, isNotePage, type Family, type LinkClass, type Note } from './public-contracts';
import { t } from '@xcss/web/admin-ui/i18n';
import { useEffect, useRef, useState } from 'react';
import { publicErrorMessage, request, type Page } from './api';
import { ImageLightbox, safeImageUrl } from './MediaPreview';
import { PhotoGrid, type PhotoItem } from './PhotoGallery';
import { PublicChrome } from './PublicChrome';
import { PublicComments } from './PublicComments';
import { ReferencePetals } from './ReferencePetals';
import { timeTogether } from './love-time.ts';

type Tab = 'notes' | 'photos' | 'wishes';

function dateLabel(value: number) {
  return new Intl.DateTimeFormat(document.documentElement.lang || 'zh-CN', { year: 'numeric', month: 'long', day: 'numeric', timeZone: 'UTC' }).format(value) + ' UTC';
}
function Avatar({ name, src }: { name: string; src: string | null | undefined }) {
  return <span className="love-avatar">{safeImageUrl(src) ? <img src={safeImageUrl(src)!} alt="" />
    : <span aria-hidden="true">{Array.from(name)[0]}</span>}</span>;
}
function Feedback({ message, retry }: { message: string; retry: () => void }) {
  return <div className="public-feedback" role="alert"><p>{message}</p><button type="button" onClick={retry}>{t('重试', 'Try again')}</button></div>;
}

export function LovePage() {
  const [love, setLove] = useState<Family | null>(null);
  const [familyFailure, setFamilyFailure] = useState('');
  const [familyVersion, setFamilyVersion] = useState(0);
  const [tab, setTab] = useState<Tab>('photos');
  const [now, setNow] = useState(Date.now());
  const [notes, setNotes] = useState<Page<Note> | null>(null);
  const [notePage, setNotePage] = useState(1);
  const [noteBusy, setNoteBusy] = useState(false);
  const [noteFailure, setNoteFailure] = useState('');
  const [noteVersion, setNoteVersion] = useState(0);
  const [classes, setClasses] = useState<LinkClass[]>([]);
  const [classFailure, setClassFailure] = useState('');
  const [classify, setClassify] = useState('');
  const [photoPage, setPhotoPage] = useState(1);
  const [photos, setPhotos] = useState<PhotoItem[]>([]);
  const [photoTotal, setPhotoTotal] = useState<number | null>(null);
  const [allPhotoTotal, setAllPhotoTotal] = useState<number | null>(null);
  const [photoBusy, setPhotoBusy] = useState(true);
  const [photoFailure, setPhotoFailure] = useState('');
  const [photoVersion, setPhotoVersion] = useState(0), [photoAttempt, setPhotoAttempt] = useState(0);
  const photoLoading = useRef(false), loadedPhotoPage = useRef(0);
  const [lightbox, setLightbox] = useState<string | null>(null);
  const content = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const controller = new AbortController();
    setFamilyFailure('');
    void request('/api/v1/family', isFamilies, { signal: controller.signal })
      .then(value => { if (!controller.signal.aborted) setLove(value[0] ?? null); })
      .catch(reason => { if (!controller.signal.aborted) setFamilyFailure(publicErrorMessage(reason, t('故事加载失败', 'Unable to load our story'))); });
    return () => controller.abort();
  }, [familyVersion]);
  useEffect(() => {
    const timer = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(timer);
  }, []);
  useEffect(() => {
    if (tab !== 'notes') return;
    const controller = new AbortController();
    setNoteBusy(true); setNoteFailure('');
    void request(`/api/v1/notes/page?page=${notePage}&size=10`, isNotePage, { signal: controller.signal })
      .then(value => { if (!controller.signal.aborted) setNotes(value); })
      .catch(reason => { if (!controller.signal.aborted) setNoteFailure(publicErrorMessage(reason, t('记录加载失败', 'Unable to load entries'))); })
      .finally(() => { if (!controller.signal.aborted) setNoteBusy(false); });
    return () => controller.abort();
  }, [tab, notePage, noteVersion]);
  useEffect(() => {
    const controller = new AbortController(); setClassFailure('');
    void request('/api/v1/links/classes?kind=lovePhoto', isLinkClasses, { signal: controller.signal })
      .then(value => { if (!controller.signal.aborted) setClasses(value); })
      .catch(reason => { if (!controller.signal.aborted) setClassFailure(publicErrorMessage(reason, t('相册分类加载失败', 'Unable to load photo categories'))); });
    return () => controller.abort();
  }, [photoVersion]);
  useEffect(() => {
    if (tab !== 'photos') { photoLoading.current = false; return; }
    const controller = new AbortController(); photoLoading.current = true; setPhotoBusy(true); setPhotoFailure('');
    const filter = classify ? `&classify=${encodeURIComponent(classify)}` : '';
    void request(`/api/v1/links/page?kind=lovePhoto&page=${photoPage}&size=12${filter}`, isLinkPage, { signal: controller.signal })
      .then(value => {
        if (controller.signal.aborted) return;
        loadedPhotoPage.current = photoPage;
        setPhotoTotal(value.total);
        if (!classify) setAllPhotoTotal(value.total);
        setPhotos(current => photoPage === 1 ? value.items : [...current, ...value.items.filter(item => !current.some(existing => existing.id === item.id))]);
      })
      .catch(reason => { if (!controller.signal.aborted) setPhotoFailure(publicErrorMessage(reason, t('相册加载失败', 'Unable to load photos'))); })
      .finally(() => { if (!controller.signal.aborted) { photoLoading.current = false; setPhotoBusy(false); } });
    return () => controller.abort();
  }, [tab, classify, photoPage, photoVersion, photoAttempt]);

  const hisName = love?.man_name || t('他', 'Him'), herName = love?.woman_name || t('她', 'Her');
  const since = parseUtcDate(love?.timing);
  const elapsed = timeTogether(since, now);
  const clock = elapsed ? [
    { key: 'years', value: elapsed.years, label: t('年', elapsed.years === 1 ? 'year' : 'years') },
    { key: 'months', value: elapsed.months, label: t('月', elapsed.months === 1 ? 'month' : 'months') },
    { key: 'days', value: elapsed.days, label: t('日', elapsed.days === 1 ? 'day' : 'days') },
    { key: 'hours', value: elapsed.hours, label: t('时', elapsed.hours === 1 ? 'hour' : 'hours') },
    { key: 'minutes', value: elapsed.minutes, label: t('分', elapsed.minutes === 1 ? 'minute' : 'minutes') },
    { key: 'seconds', value: elapsed.seconds, label: t('秒', elapsed.seconds === 1 ? 'second' : 'seconds') },
  ] : [];
  const countdown = parseUtcDate(love?.countdown_time);
  const target = new Date(countdown), current = new Date(now);
  const left = Number.isFinite(countdown) ? Math.round((Date.UTC(target.getUTCFullYear(), target.getUTCMonth(), target.getUTCDate()) - Date.UTC(current.getUTCFullYear(), current.getUTCMonth(), current.getUTCDate())) / 86400000) : null;
  const tabs: { id: Tab; label: string; icon: string; description: string }[] = [
    { id: 'notes', label: t('点点滴滴', 'Moments'), icon: '/legacy/love-moments.jpg', description: t('☀️ 今朝有酒今朝醉', '☀️ Little moments of joy') },
    { id: 'photos', label: t('时光相册', 'Photos'), icon: '/legacy/love-photos.jpg', description: t('📸 记录美好瞬间', '📸 Keep beautiful memories') },
    { id: 'wishes', label: t('祝福板', 'Wishes'), icon: '/legacy/love-wishes.jpg', description: t('📋 写下你们的祝福', '📋 Leave your wishes') },
  ];
  function chooseClass(value: string) { if (value === classify) return; loadedPhotoPage.current = 0; setClassify(value); setPhotoPage(1); setPhotos([]); setPhotoTotal(null); }
  function loadMorePhotos() { if (photoLoading.current) return; photoLoading.current = true; setPhotoBusy(true); setPhotoPage(loadedPhotoPage.current + 1); setPhotoAttempt(value => value + 1); }
  function showTab(value: Tab) { setTab(value); content.current?.scrollIntoView({ behavior: window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth', block: 'start' }); }

  return <PublicChrome plainHeader title={t('恋爱笔记', 'Love journal')}>
    <ReferencePetals/><div className="love-page">
      <section className="love-hero" style={safeImageUrl(love?.bg_cover) ? { backgroundImage: `url(${JSON.stringify(safeImageUrl(love?.bg_cover))})` } : undefined}>
        <div className="love-hero-inner"><div className="love-pair-card"><div className="love-pair">
          <div><Avatar name={hisName} src={love?.man_cover} /><strong>{hisName}</strong></div>
          <img className="love-heart" src="/legacy/love-heart.svg" alt="" aria-hidden="true"/>
          <div><Avatar name={herName} src={love?.woman_cover} /><strong>{herName}</strong></div>
        </div><p>{Number.isFinite(since) ? <>{t('故事开始于 ', 'Our story began on ')}<time dateTime={love?.timing || undefined}>{dateLabel(since)}</time></> : t('等待第一份属于我们的回忆', 'A place for our first memory together')}</p></div></div>
        <svg className="love-wave" viewBox="0 0 1440 110" preserveAspectRatio="none" aria-hidden="true"><path opacity=".45" d="M0 35C180 100 280 0 440 45S700 95 870 40S1170 0 1440 45V110H0Z"/><path opacity=".7" d="M0 55C150 15 280 95 450 60S730 10 920 65S1210 80 1440 35V110H0Z"/><path d="M0 70C180 100 340 30 530 75S850 90 1060 55S1300 80 1440 65V110H0Z"/></svg>
      </section>
      <div className="love-content">
        {familyFailure && <Feedback message={familyFailure} retry={() => setFamilyVersion(value => value + 1)} />}
        <section className="love-timer">
          <h1>{t('这是我们一起走过的', 'Our journey together')}</h1>
          {elapsed ? <div className="love-clock" aria-label={t('相伴时长', 'Time spent together')}>
            <span className="love-clock-prefix">{t('第', 'Together for')}</span>{clock.map(unit => <div key={unit.key} data-unit={unit.key}><strong>{unit.value}</strong><span>{unit.label}</span></div>)}
          </div> : <div className="love-timer-empty"><strong>{Number.isFinite(since) ? t('新的旅程即将开始', 'A new journey begins soon') : t('从第一份回忆开始', 'Start with your first memory')}</strong><p>{t('留下相识的日期，让时间替我们记住每一个日子。', 'Add the day you met and keep track of each day together.')}</p></div>}
          {left !== null && <p className="love-countdown"><span>{love?.countdown_title || t('下一件值得期待的事', 'Something to look forward to')}：</span><strong>{left > 0 ? t('还有 {0} 天', left === 1 ? '{0} day to go' : '{0} days to go', [left]) : left === 0 ? t('就是今天', 'Today is the day') : t('已过去 {0} 天', left === -1 ? '{0} day ago' : '{0} days ago', [-left])}</strong></p>}
        </section>
        <a className="love-letter" href="/letter"><span>{t('飞车传信', 'A letter for you')}</span></a>
        <details className="love-story-card"><summary>{t('我们的故事', 'Our story')}</summary><p>{love?.family_info || t('把平常的日子写下来，让照片和文字慢慢填满这个家。', 'Make a home for everyday moments, one photo and one story at a time.')}</p><a href="/jotting">{t('看看日常记录', 'Explore our journal')} <span aria-hidden="true">↗</span></a></details>
        <div ref={content} className="love-workspace">
          <nav className="love-tabs" aria-label={t('恋爱笔记内容', 'Love journal content')}>
            {tabs.map(item => <button key={item.id} type="button" className={tab === item.id ? 'active' : ''} aria-pressed={tab === item.id} aria-controls="love-panel" onClick={() => showTab(item.id)}>
              <img className="love-tab-icon" src={item.icon} alt=""/><span><strong>{item.label}</strong><small>{item.description}</small></span>
            </button>)}
          </nav>
          <section id="love-panel" className="love-panel" aria-label={tabs.find(item => item.id === tab)?.label} aria-busy={tab === 'photos' ? photoBusy : tab === 'notes' ? noteBusy : false}>
            {tab === 'photos' && <>
              <div className="original-tag-panel love-photo-filters" aria-label={t('相册分类', 'Photo categories')}><button type="button" aria-pressed={!classify} className={!classify ? 'active' : ''} onClick={() => chooseClass('')}>{t('全部', 'All')} <small>{allPhotoTotal ?? (classes.length ? classes.reduce((sum, item) => sum + item.count, 0) : null)}</small></button>{classes.map(item => <button type="button" key={item.classify} aria-pressed={classify === item.classify} className={classify === item.classify ? 'active' : ''} onClick={() => chooseClass(item.classify)}>{item.classify} <small>{item.count}</small></button>)}</div>
              <div className="love-section-heading"><div><h2>{classify || t('时光相册', 'Photo album')}</h2><p>{t('收藏一起看过的风景。', 'Keep the places and moments we have shared.')}</p></div>{photoTotal !== null && <span>{t('{0} 张照片', photoTotal === 1 ? '{0} photo' : '{0} photos', [photoTotal])}</span>}</div>
              {(photoFailure || classFailure) && <Feedback message={photoFailure || classFailure} retry={() => setPhotoVersion(value => value + 1)} />}
              {photoBusy && !photos.length ? <p className="public-loading" role="status">{t('正在加载照片…', 'Loading photos…')}</p> : (photos.length > 0 || !photoFailure) && <PhotoGrid items={photos} onPreview={setLightbox} />}
              {photoTotal !== null && photos.length < photoTotal && <button type="button" className="travel-more" disabled={photoBusy} onClick={loadMorePhotos}>{photoBusy ? t('加载中…', 'Loading…') : t('加载更多照片', 'Load more photos')}</button>}
            </>}
            {tab === 'notes' && <>
              <div className="love-section-heading"><div><h2>{t('点点滴滴', 'Moments')}</h2><p>{t('记录那些平常又特别的小事。', 'A place for the little things that make our days.')}</p></div><a href="/jotting">{t('浏览更多记录', 'Browse more entries')} <span aria-hidden="true">↗</span></a></div>
              {noteFailure && <Feedback message={noteFailure} retry={() => setNoteVersion(value => value + 1)} />}
              {noteBusy ? <p className="public-loading" role="status">{t('正在加载记录…', 'Loading entries…')}</p> : notes && <div className="love-moments-grid">{notes.items.map(item => <article className="love-moment-card" key={item.id}><small>{item.username || t('站长', 'Site owner')} · {formatUtcTime(item.create_time) || t('最近', 'Recently')}</small><p>{item.content}</p>{safeImageUrl(item.image_path) && <button type="button" className="wall-photo-button" onClick={() => setLightbox(safeImageUrl(item.image_path))} aria-label={t('放大图片', 'Enlarge image')}><img src={safeImageUrl(item.image_path)!} alt={t('随笔图片', 'Note image')} loading="lazy" /></button>}</article>)}{notes.items.length === 0 && <div className="public-empty-state"><span aria-hidden="true">✎</span><h3>{t('还没有记录。', 'No entries yet.')}</h3><p>{t('写下此刻的想法，留住每一个值得纪念的日常。', 'Write a thought and keep a little piece of today.')}</p></div>}</div>}
              {notes && notes.total > notes.size && <nav className="pager" aria-label={t('记录分页', 'Entry pages')}><button type="button" disabled={noteBusy || notePage <= 1} onClick={() => setNotePage(value => value - 1)}>{t('上一页', 'Previous page')}</button><span>{t('第 {0} 页', 'Page {0}', [notePage])}</span><button type="button" disabled={noteBusy || notePage * notes.size >= notes.total} onClick={() => setNotePage(value => value + 1)}>{t('下一页', 'Next page')}</button></nav>}
            </>}
            {tab === 'wishes' && <div className="love-wishes"><PublicComments kind="love" title={t('祝福板', 'Wishes')} /></div>}
          </section>
        </div>
      </div>
    </div>
    <ImageLightbox src={lightbox} onClose={() => setLightbox(null)} />
  </PublicChrome>;
}
