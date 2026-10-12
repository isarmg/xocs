import type { ReactNode } from 'react';
import { HeaderActions, HeaderNavigation } from '@xcss/web/admin-shell';
import { Button } from '@xcss/web/admin-ui';
import { t } from '@xcss/web/admin-ui/i18n';

export const adminGroups = [
  { id: 'overview', label: t('总览', 'Overview'), title: t('总览', 'Overview'), pages: [
    { id: 'dashboard', label: t('总览', 'Overview') },
  ] },
  { id: 'content', label: t('内容', 'Content'), title: t('内容管理', 'Content management'), pages: [
    { id: 'articles', label: t('文章管理', 'Articles') },
    { id: 'categories', label: t('分类管理', 'Categories') },
    { id: 'labels', label: t('标签管理', 'Tags') },
    { id: 'notes', label: t('随笔管理', 'Notes') },
    { id: 'letter', label: t('信笺管理', 'Letter') },
  ] },
  { id: 'community', label: t('社区', 'Community'), title: t('社区管理', 'Community management'), pages: [
    { id: 'comments', label: t('评论管理', 'Comments') },
    { id: 'tree-hole', label: t('留言板管理', 'Message board') },
  ] },
  { id: 'resources', label: t('资源', 'Resources'), title: t('资源管理', 'Resource management'), pages: [
    { id: 'resources', label: t('文件管理', 'Files') },
    { id: 'links', label: t('内容资源', 'Content resources') },
    { id: 'family', label: t('恋爱笔记', 'Love journal') },
  ] },
  { id: 'settings', label: t('设置', 'Settings'), title: t('网站设置', 'Site settings'), pages: [
    { id: 'site', label: t('网站设置', 'Site settings') },
    { id: 'home', label: t('首页栏目', 'Home sections') },
  ] },
] as const;

export function AdminWorkspace({ page, children }: { page: string; children: ReactNode }) {
  const activePage = page === 'new' || page.startsWith('edit/') ? 'articles' : page;
  const group = page === 'account' ? undefined : adminGroups.find(item => item.pages.some(item => item.id === activePage)) ?? adminGroups[1];
  const navigate = (id: string) => { window.location.hash = id; };
  return <div className="xocs-admin-workspace xcss-content-stack">
    <HeaderNavigation label={t('后台导航', 'Administration navigation')}>
      {adminGroups.map(item => <Button key={item.id} aria-pressed={group?.id === item.id}
        onClick={() => navigate(item.pages[0].id)}>{item.label}</Button>)}
    </HeaderNavigation>
    <HeaderActions>
      <a className="xocs-site-link" href="/" aria-label={t('返回网站', 'Back to site')} title={t('返回网站', 'Back to site')}>
        <svg aria-hidden="true" width="1em" height="1em" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round"><path d="m3 10 9-7 9 7M5 9v11h5v-7h4v7h5V9" /></svg>
      </a>
    </HeaderActions>
    {group && group.pages.length > 1 && <nav className="xcss-secondary-navigation" aria-label={group.title}>
      {group.pages.map(item => <Button key={item.id} aria-pressed={activePage === item.id}
        onClick={() => navigate(item.id)}>{item.label}</Button>)}
    </nav>}
    {children}
  </div>;
}
