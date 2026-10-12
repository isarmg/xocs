import { formatUtcTime } from './utc-time.ts';
import { isSiteInfo } from './public-contracts';
import { t } from '@xcss/web/admin-ui/i18n';
import {useEffect, useMemo, useState, type ReactNode} from 'react';
import {request, type SiteInfo} from './api';
import {LanguageControl, clearValidation, localizeValidation} from './language';

const links = [
  ['🏡', t("首页", "Home"), '/'],
  ['💖', t("家", "Home and love"), '/love'],
  ['🌈', t("游记", "Travels"), '/menory'],
  ['🏖️', t("随笔", "Journal"), '/jotting'],
  ['✎', t("微言", "Posts"), '/weiYan'],
  ['📒', t("记录", "Articles"), '/sort'],
  ['📸', t("相册", "Photos"), '/travel'],
  ['🧰', t("百宝箱", "Toolbox"), '/favorite'],
  ['📪', t("留言", "Messages"), '/message'],
  ['ℹ', t("关于", "About"), '/about'],
] as const;

function safeCover(value: string | null | undefined) {
  if (!value) return '/live/xocs-current-hero.jpg';
  try {
    const url = new URL(value, window.location.origin);
    return ['http:', 'https:'].includes(url.protocol) ? url.href : '/live/xocs-current-hero.jpg';
  } catch {
    return '/live/xocs-current-hero.jpg';
  }
}
function randomCover(value:string|null|undefined){if(!value)return null;let items:string[]=[];try{const parsed:unknown=JSON.parse(value);if(Array.isArray(parsed))items=parsed.filter((item):item is string=>typeof item==='string');}catch{items=value.split('\n').map(item=>item.trim()).filter(Boolean);}return items.length?items[Math.floor(Math.random()*items.length)]:null;}

export function PublicChrome({site, title, home = false, cover, articleMeta, plainHeader = false, subtitle, children}: {
  site?: SiteInfo | null;
  title?: string;
  home?: boolean;
  cover?: string | null;
  articleMeta?: {author: string | null; date: string | null; views: number; comments: number; likes: number};
  plainHeader?: boolean;
  subtitle?: string;
  children: ReactNode;
}) {
  const [loadedSite, setLoadedSite] = useState<SiteInfo | null>(null);
  const [scrolled, setScrolled] = useState(false);
  const [headerHidden, setHeaderHidden] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [dark, setDark] = useState(() => { try { return window.localStorage.getItem('xocs-theme') === 'dark'; } catch { return false; } });
  useEffect(() => {
    if (site !== undefined) return;
    const controller = new AbortController();
    void request('/api/v1/site', isSiteInfo, {signal: controller.signal}).then(setLoadedSite).catch(() => {});
    return () => controller.abort();
  }, [site]);
  useEffect(() => {
    const scrollPosition = () => Math.max(0, Math.min(window.scrollY, document.documentElement.scrollHeight - window.innerHeight));
    let previous = scrollPosition(), direction = 0, distance = 0, frame = 0;
    const update = () => {
      frame = 0;
      const position = scrollPosition(), delta = position - previous;
      previous = position;
      setScrolled(position > 40);
      if (position <= 48 || menuOpen || document.activeElement?.matches('.public-header :focus-visible')) {
        direction = 0; distance = 0; setHeaderHidden(false); return;
      }
      if (!delta) return;
      const nextDirection = Math.sign(delta);
      distance = nextDirection === direction ? distance + Math.abs(delta) : Math.abs(delta);
      direction = nextDirection;
      if (distance < 8) return;
      distance = 0;
      setHeaderHidden(direction > 0);
    };
    const schedule = () => { if (!frame) frame = window.requestAnimationFrame(update); };
    update();
    window.addEventListener('scroll', schedule, {passive: true});
    return () => { window.removeEventListener('scroll', schedule); window.cancelAnimationFrame(frame); };
  }, [menuOpen]);
  useEffect(() => {
    const close = (event: KeyboardEvent) => { if (event.key === 'Escape') { setMenuOpen(false); } };
    window.addEventListener('keydown', close);
    return () => window.removeEventListener('keydown', close);
  }, []);
  function toggleDark() { setDark(value => { const next = !value; try { window.localStorage.setItem('xocs-theme', next ? 'dark' : 'light'); } catch { /* Storage may be unavailable. */ } return next; }); }
  const themeLabel = dark ? t('切换到浅色模式', 'Switch to light mode') : t('切换到深色模式', 'Switch to dark mode');
  const displayControls = <div className="public-header-controls"><LanguageControl/>
    <button className="theme-toggle" type="button" aria-label={themeLabel} title={themeLabel} aria-pressed={dark} onClick={toggleDark}>
      <svg aria-hidden="true" width="1em" height="1em" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round">
        {dark ? <><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42"/></> : <path d="M20.9 13A9 9 0 1 1 11 3a7 7 0 0 0 9.9 10Z"/>}
      </svg>
    </button>
  </div>;
  const info = site === undefined ? loadedSite : site;
  const fallbackCover=useMemo(()=>randomCover(info?.random_cover),[info?.random_cover]);
  return <div onInvalidCapture={localizeValidation} onInputCapture={clearValidation} onChangeCapture={clearValidation} className={`public-site original-public page-${window.location.pathname.slice(1).replace(/[^a-zA-Z0-9_-]/g,'') || 'home'}${dark ? ' theme-dark' : ''}`}>
    <header className={`public-header${scrolled || menuOpen || plainHeader ? ' entered' : ''}${plainHeader ? ' plain-header' : ''}${headerHidden && !menuOpen ? ' header-hidden' : ''}`}>
      <a className="brand" href="/" aria-label={t("xocs 首页", "xocs home")}>xocs</a>
      <nav className={menuOpen ? 'open' : ''} aria-label={t("主导航", "Main navigation")}><span className="public-mobile-title">{t("欢迎光临", "Welcome")}</span>{links.map(([icon, label, href]) => <a key={href} href={href} aria-current={window.location.pathname === href ? 'page' : undefined}><span aria-hidden="true">{icon}</span> {label}</a>)}</nav>
      {displayControls}
      <button className="public-menu-toggle" type="button" aria-label={menuOpen ? t("关闭导航", "Close navigation") : t("打开导航", "Open navigation")} aria-expanded={menuOpen} onClick={() => setMenuOpen(value => !value)}>{menuOpen ? '✕' : '☰'}</button>
    </header>
    {!plainHeader && <section className={`hero${home ? ' home-hero' : articleMeta ? ' article-hero' : ' compact-hero'}`} style={{backgroundImage: `linear-gradient(#0b2c3e3d,#0b2c3e52),url("${safeCover(cover || info?.background_image || fallbackCover)}")`}}>
      <div className={articleMeta ? 'hero-article-info' : 'hero-center'}><h1>{home ? info?.web_title || t("相信记录的力量", "Believe in the power of stories") : title || 'xocs'}</h1>{home && <p className="hero-poem">{t("相信记录的力量！", "Believe in the power of stories!")}<span className="cursor">|</span></p>}{subtitle && <p className="hero-subtitle">{subtitle}</p>}{articleMeta && <p className="hero-article-meta">👤 {articleMeta.author||t("站长", "Site owner")}　·　📅 {formatUtcTime(articleMeta.date) || t("最近", "Recently")}　·　🔥 {articleMeta.views}{t(" 热度 · 💬 ", " views · 💬 ")}{articleMeta.comments}{t(" 评论 · ❤️ ", " comments · ❤️ ")}{articleMeta.likes}{t(" 点赞", " likes")}</p>}</div>
      {home && <><a className="hero-down" href="#public-content" aria-label={t("浏览文章", "Browse articles")}>⌄</a><div className="hero-wave hero-wave-back" aria-hidden="true"/><div className="hero-wave hero-wave-front" aria-hidden="true"/></>}
    </section>}
    <main id="public-content" className={`public-main${home ? '' : ' extra-main'}`}>{children}</main>
    <footer className="public-footer"><span>{info?.footer || 'xocs'}</span><small>{t("本网站由 xocs 强力支持", "Powered by xocs")}</small></footer>
    {scrolled && <div className="public-tools"><button className="back-top" type="button" onClick={() => window.scrollTo({top: 0, behavior: 'smooth'})} aria-label={t('返回顶部', 'Back to top')}><span aria-hidden="true">↑</span></button></div>}
  </div>;
}
