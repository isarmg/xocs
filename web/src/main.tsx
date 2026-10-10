import { uploadFilename } from './upload-filename.ts';
import { isArticle, isArticleAccess, isArticlePage, isCategories, isCategory, isEditorLabels, isHomeSections, isNews, isNewsEntry, isPublicLabels, isSiteInfo, isSiteStats, isWallPosts, type NewsEntry } from './public-contracts';
import { publicErrorMessage } from './api';
import { t } from '@xcss/web/admin-ui/i18n';
import {StrictMode, useEffect, useMemo, useRef, useState} from 'react';
import {createRoot} from 'react-dom/client';
import {createXcssAdminApplication, InstanceHeaderActions, useAdminApplication, AccountPage} from '@xcss/web/admin-shell';
import {Button, EmptyState, ErrorState, FormField, LoadingState, PageHeader, Select, Table, TextField} from '@xcss/web/admin-ui';
import '@xcss/web/design-tokens/tokens.css';
import '@xcss/web/design-tokens/tokens.dark.css';
import '@xcss/web/admin-ui/styles.css';
import '@xcss/web/design-tokens/reset.css';
import '@xcss/web/design-tokens/accessibility.css';
import '@xcss/web/web-fonts/fonts.css';
import {startAfterFonts} from '@xcss/web/web-fonts';
import './style.css';
import './public-original.css';
import './public-layout.css';
import './admin.css';
import { AdminWorkspace } from './AdminLayout';
import {articleExcerpt, request, type Article, type ArticleInput, type ArticleSummary, type Category, type Page, type PublicLabel, type SiteInfo} from './api';
import {clearValidation, languageHref, localizeValidation, preserveLinkLanguage} from './language';
import {CommentsPage, Dashboard, FamilyPage, LabelsPage, LinksPage, NotesPage, ResourcesPage, SitePage, TreeHolePage, UsersPage} from './AdminExtras';
import {PublicExtras} from './PublicExtras';
import {HomeSectionsPage,type HomeSection} from './HomeSections';
import {PublicChrome} from './PublicChrome';
import {PublicComments} from './PublicComments';
import {ArticleNews} from './ArticleNews';
import {ImageLightbox} from './MediaPreview';
import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';

const markdown = new MarkdownIt({html: false, linkify: true, breaks: true});
function safeMediaUrl(value:string|null){if(!value)return null;try{const url=new URL(value,window.location.origin);return ['http:','https:'].includes(url.protocol)?url.href:null;}catch{return null;}}

function ReadingArticle({article}: {article: Article}) {
  const readingRef=useRef<HTMLElement|null>(null);
  const [progress,setProgress]=useState(0);
  const [active,setActive]=useState('');
  const [copyrightOpen,setCopyrightOpen]=useState(false);
  const [lightbox,setLightbox]=useState<string|null>(null);
  const [copyStatus,setCopyStatus]=useState('');
  const rendered=useMemo(()=>{const holder=document.createElement('div');holder.innerHTML=DOMPurify.sanitize(markdown.render(article.article_content));const headings=Array.from(holder.querySelectorAll('h2,h3,h4')).map((node,index)=>{const id=`section-${index+1}`;node.id=id;return {id,title:node.textContent?.trim()||t("第 {0} 节", "Section {0}", [index+1]),level:Number(node.tagName[1])};});holder.querySelectorAll('pre').forEach(pre=>{const button=document.createElement('button');button.type='button';button.className='article-copy-code';button.textContent=t("复制代码", "Copy code");pre.prepend(button);});holder.querySelectorAll('img').forEach(img=>{img.setAttribute('role','button');img.setAttribute('tabindex','0');img.setAttribute('aria-label',t("查看图片：{0}", "View image: {0}", [img.alt||t("文章插图", "Article illustration")]));});return {html:holder.innerHTML,headings};},[article.article_content]);
  async function articleAction(target:EventTarget|null){if(!(target instanceof Element))return;const button=target.closest('.article-copy-code');if(button){const code=button.parentElement?.querySelector('code')?.textContent||'';try{await navigator.clipboard.writeText(code);setCopyStatus(t("代码已复制", "Code copied"));window.setTimeout(()=>setCopyStatus(''),2000);}catch{setCopyStatus(t("复制失败，请手动选中代码", "Copy failed. Select the code manually."));}return;}const img=target.closest('img');if(img&&img.closest('.article-markdown'))setLightbox(safeMediaUrl(img.getAttribute('src')));}
  useEffect(()=>{let frame=0;function update(){cancelAnimationFrame(frame);frame=requestAnimationFrame(()=>{const card=readingRef.current;if(!card)return;const start=card.getBoundingClientRect().top+window.scrollY;const end=Math.max(start+1,start+card.offsetHeight-window.innerHeight);setProgress(Math.round(Math.min(100,Math.max(0,(window.scrollY-start)/(end-start)*100))));let current='';for(const heading of rendered.headings){const target=document.getElementById(heading.id);if(target&&target.getBoundingClientRect().top<=150)current=heading.id;}setActive(current);});}update();window.addEventListener('scroll',update,{passive:true});window.addEventListener('resize',update);return()=>{cancelAnimationFrame(frame);window.removeEventListener('scroll',update);window.removeEventListener('resize',update);};},[rendered]);
  const outline=<ol>{rendered.headings.map(heading=><li key={heading.id} className={`toc-level-${heading.level}`}><a className={active===heading.id?'active':''} href={`#${heading.id}`}>{heading.title}</a></li>)}</ol>;
  return <><div className="reading-progress" role="progressbar" aria-label={t("阅读进度", "Reading progress")} aria-valuemin={0} aria-valuemax={100} aria-valuenow={progress}><span style={{width:`${progress}%`}}/></div><div className="reading-layout"><article className="reading-card reading-article" ref={readingRef}><a href="/">{t("← 返回文章", "← Back to articles")}</a>{safeMediaUrl(article.video_url)&&<video className="article-video" controls preload="metadata" poster={safeMediaUrl(article.article_cover)||undefined} src={safeMediaUrl(article.video_url)||undefined}/>}{article.password_required===0&&<ArticleNews articleId={article.id}/>}<div className="article-markdown" onClick={event=>void articleAction(event.target)} onKeyDown={event=>{if((event.key==='Enter'||event.key===' ')&&event.target instanceof HTMLImageElement){event.preventDefault();void articleAction(event.target);}}} dangerouslySetInnerHTML={{__html:rendered.html}} />{copyStatus&&<span className="article-copy-status" role="status">{copyStatus}</span>}<div className="article-end-info"><p>{t("文章最后更新于 ", "Last updated ")}{article.update_time||article.create_time||t("最近", "Recently")}</p><a href={`/sort?sort_id=${article.sort_id}&label_id=${article.label_id}`}>{article.sort_name||t("分类", "Category")} ▶ {article.label_name||t("标签", "Tags")}</a><blockquote>{t("作者：", "Author: ")}{article.username||t("站长", "Site owner")}<br/>{t("版权与许可请详阅 ", "For copyright and licensing, see ")}<button type="button" onClick={()=>setCopyrightOpen(true)}>{t("版权声明", "Copyright notice")}</button></blockquote></div>{article.comment_status===1&&article.password_required===0&&<PublicComments articleId={article.id}/>}</article>{rendered.headings.length>0&&<aside className="reading-toc" aria-label={t("文章目录", "Article contents")}><nav><h3>{t("目录", "Contents")}</h3>{outline}</nav><details><summary>{t("文章目录 · ", "Contents · ")}{progress}%</summary>{outline}</details></aside>}</div>{copyrightOpen&&<div className="article-copyright-backdrop" role="presentation" onClick={()=>setCopyrightOpen(false)}><section role="dialog" aria-modal="true" aria-label={t("版权声明", "Copyright notice")} onClick={event=>event.stopPropagation()}><button type="button" aria-label={t("关闭版权声明", "Close copyright notice")} onClick={()=>setCopyrightOpen(false)}>×</button><h2>{t("版权声明", "Copyright notice")}</h2><p>{t("文章、图片和视频的著作权归各自原作者所有。转载或使用内容时，请遵守原作者的许可要求；本站原创内容采用署名－非商业性使用－相同方式共享 4.0 国际许可协议。", "Articles, images, and videos belong to their respective authors. Follow their license terms when using or republishing content. Original content on this site is licensed under CC BY-NC-SA 4.0 International.")}</p><p><a href="https://creativecommons.org/licenses/by-nc-sa/4.0/" target="_blank" rel="noopener noreferrer">{t("查看许可协议", "View license")}</a></p></section></div>}<ImageLightbox src={lightbox} onClose={()=>setLightbox(null)}/></>;
}

function isDelete(value: unknown): value is {deleted: boolean} {
  return typeof value === 'object' && value !== null && 'deleted' in value && value.deleted === true;
}

function AdminPages() {
  const [page, setPage] = useState(() => window.location.hash.slice(1) || 'articles');
  useEffect(() => { const update = () => setPage(window.location.hash.slice(1) || 'articles'); window.addEventListener('hashchange', update); return () => window.removeEventListener('hashchange', update); }, []);
  function renderPage() {
    if (page === 'account') return <AccountPage />;
    if (page === 'dashboard') return <Dashboard />;
    if (page === 'users') return <UsersPage />;
    if (page === 'comments') return <CommentsPage />;
    if (page === 'notes') return <NotesPage />;
    if (page === 'site') return <SitePage />;
    if (page === 'home') return <HomeSectionsPage />;
    if (page === 'labels') return <LabelsPage />;
    if (page === 'resources') return <ResourcesPage />;
    if (page === 'links') return <LinksPage />;
    if (page === 'tree-hole') return <TreeHolePage />;
    if (page === 'family') return <FamilyPage />;
    if (page === 'categories') return <CategoryManager />;
    if (page.startsWith('edit/')) return <ArticleEditor key={page} id={Number(page.slice(5))} />;
    if (page === 'new') return <ArticleEditor key="new" />;
    return <ArticleManager />;
  }
  return <AdminWorkspace page={page}>{renderPage()}</AdminWorkspace>;
}

const AdminApp = createXcssAdminApplication({
  product: {name: 'xocs'},
  navigation: [],
  loginLandingHref: languageHref('/admin#articles'),
  routes: <div style={{display:'contents'}} onInvalidCapture={localizeValidation} onInputCapture={clearValidation} onChangeCapture={clearValidation}><AdminPages /></div>,
});

function ArticleManager() {
  const {client, notify} = useAdminApplication();
  const [loaded, setLoaded] = useState<{page:number;refresh:number;data:Page<ArticleSummary>} | null>(null);
  const [failure, setFailure] = useState('');
  const [busy, setBusy] = useState(false);
  const [refresh, setRefresh] = useState(0);
  const [page, setPage] = useState(1);
  const result=loaded?.page===page&&loaded.refresh===refresh?loaded.data:null;
  useEffect(() => {
    const controller = new AbortController();
    setBusy(true);
    setFailure('');
    void client.request(`/api/v1/content/articles?page=${page}&size=15`, isArticlePage, {signal: controller.signal})
      .then(data => {
        if (controller.signal.aborted) return;
        const lastPage = Math.max(1, Math.ceil(data.total / data.size));
        if (page > lastPage) { setLoaded(null); setPage(lastPage); return; }
        setLoaded({page,refresh,data});
      })
      .catch(() => { if (!controller.signal.aborted) setFailure(t("文章加载失败", "Unable to load articles")); })
      .finally(() => { if (!controller.signal.aborted) setBusy(false); });
    return () => controller.abort();
  }, [client, refresh, page]);
  async function remove(id: number) {
    if (!window.confirm(t("确定删除这篇文章？", "Delete this article?"))) return;
    try {
      await client.request(`/api/v1/content/articles/${id}`, isDelete, {method: 'DELETE'});
      notify(t("文章已删除", "Article deleted")); setLoaded(null); setRefresh(value => value+1);
    } catch { setFailure(t("删除失败，请刷新后重试", "Delete failed. Refresh and try again.")); }
  }
  return <section className="xcss-content-stack">
    <InstanceHeaderActions create={() => {window.location.hash='new';}} createLabel={t("新增文章", "Add article")} refresh={() => setRefresh(value => value+1)} refreshing={busy} />
    <PageHeader><div><h1>{t("文章管理", "Articles")}</h1><p>{t("管理公开文章与草稿。", "Manage published articles and drafts.")}</p></div></PageHeader>
    {failure && <ErrorState onRetry={() => setRefresh(value => value+1)}>{failure}</ErrorState>}
    {busy && !result ? <LoadingState /> : result?.items.length ? <Table aria-label={t("文章列表", "Article list")}><thead><tr><th>{t("标题", "Title")}</th><th>{t("状态", "Status")}</th><th>{t("浏览", "Views")}</th><th>{t("创建时间", "Created at")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{result.items.map(item => <tr key={item.id}><td><strong>{item.article_title}</strong><small className="muted">#{item.id}</small></td><td><span className={item.view_status ? 'badge success' : 'badge'}>{item.view_status ? t("公开", "Public") : t("加密", "Password protected")}</span></td><td>{item.view_count}</td><td>{item.create_time || '—'}</td><td><div className="xcss-actions"><Button onClick={() => {window.location.hash=`edit/${item.id}`;}}>{t("编辑", "Edit")}</Button><Button className="xcss-danger" onClick={() => void remove(item.id)}>{t("删除", "Delete")}</Button></div></td></tr>)}</tbody></Table> : result&&<EmptyState>{t("还没有文章。点击“新增文章”开始创作。", "No articles yet. Select “Add article” to begin writing.")}</EmptyState>}
    {result&&(result.total>15||page>1)&&<nav className="pager" aria-label={t("分页", "Pagination")}><Button disabled={busy||page<=1} onClick={() => setPage(page-1)}>{t("上一页", "Previous page")}</Button><span>{t("第 {0} 页 · 共 {1} 篇", "Page {0} · {1} articles", [page, result.total])}</span><Button disabled={busy||page*15>=result.total} onClick={() => setPage(page+1)}>{t("下一页", "Next page")}</Button></nav>}
  </section>;
}

const emptyArticle: ArticleInput = {article_title:'',article_content:'',article_cover:null,video_url:null,sort_id:0,label_id:0,view_status:true,recommend_status:false,comment_status:true,password:null,tips:null};
function ArticleEditor({id}: {id?: number}) {
  const {client, notify} = useAdminApplication();
  const [article, setArticle] = useState<ArticleInput>(emptyArticle);
  const [categories, setCategories] = useState<Category[]>([]);
  const [labels, setLabels] = useState<{id:number;sort_id:number;label_name:string}[]>([]);
  const [saving, setSaving] = useState(false);
  const [preview,setPreview]=useState(false);
  const [uploading,setUploading]=useState(false);
  const uploadRequest=useRef<AbortController|null>(null),saveRequest=useRef<AbortController|null>(null);
  useEffect(()=>{setUploading(false);setSaving(false);return()=>{uploadRequest.current?.abort();saveRequest.current?.abort();uploadRequest.current=null;saveRequest.current=null;};},[id]);
  const [failure, setFailure] = useState('');
  const previewHtml=useMemo(()=>DOMPurify.sanitize(markdown.render(article.article_content)),[article.article_content]);
  const [loadedId,setLoadedId]=useState<number|null|undefined>(id?null:undefined);
  const [loadFailure,setLoadFailure]=useState(''),[loadAttempt,setLoadAttempt]=useState(0);
  const articleReady=loadedId===id;
  useEffect(() => {
    const controller = new AbortController();
    void request('/api/v1/categories', isCategories, {signal: controller.signal}).then(value => {if (!controller.signal.aborted) setCategories(value);}).catch(() => {if(!controller.signal.aborted)setFailure(t("分类加载失败", "Unable to load categories"));});
    void client.request('/api/v1/content/labels', isEditorLabels, {signal:controller.signal}).then(value => {if (!controller.signal.aborted) setLabels(value);}).catch(() => {if(!controller.signal.aborted)setFailure(t("标签加载失败", "Unable to load tags"));});
    return () => controller.abort();
  }, [client]);
  useEffect(() => {
    const controller = new AbortController();setLoadFailure('');setFailure('');
    if (!id) {setArticle(emptyArticle);setLoadedId(undefined);return()=>controller.abort();}
    setLoadedId(null);
    void client.request(`/api/v1/content/articles/${id}`, isArticle, {signal: controller.signal}).then(value => {
      if (controller.signal.aborted)return;
      setArticle({article_title:value.article_title,article_content:value.article_content,article_cover:value.article_cover,video_url:value.video_url,sort_id:value.sort_id,label_id:value.label_id,view_status:Boolean(value.view_status),recommend_status:Boolean(value.recommend_status),comment_status:Boolean(value.comment_status),password:null,tips:value.tips});setLoadedId(id);
    }).catch(() => {if(!controller.signal.aborted)setLoadFailure(t("文章加载失败", "Unable to load article"));});
    return () => controller.abort();
  }, [client,id,loadAttempt]);
  function change<K extends keyof ArticleInput>(key: K, value: ArticleInput[K]) {setArticle(current => ({...current,[key]:value}));}
  async function uploadPicture(file:File|undefined,target:'cover'|'content'){
    if(!articleReady||!file||uploadRequest.current||saveRequest.current)return;
    if(file.size>10*1024*1024||!['image/png','image/jpeg','image/gif','image/webp'].includes(file.type)){setFailure(t("请选择不超过 10 MB 的 PNG、JPEG、GIF 或 WebP 图片", "Choose a PNG, JPEG, GIF, or WebP image up to 10 MB"));return;}
    const controller=new AbortController();uploadRequest.current=controller;setUploading(true);setFailure('');
    try{const result=await client.request('/api/v1/content/upload',(value):value is {path:string}=>typeof value==='object'&&value!==null&&'path'in value&&typeof value.path==='string',{method:'POST',signal:controller.signal,headers:{'Content-Type':file.type,'X-File-Name':uploadFilename(file.name)},body:file});if(controller.signal.aborted||uploadRequest.current!==controller)return;if(target==='cover')change('article_cover',result.path);else setArticle(current=>({...current,article_content:`${current.article_content}\n\n![${file.name.replace(/[\[\]]/g,'')}](${result.path})\n`}));notify(t("图片已上传", "Image uploaded"));}catch(reason){if(!controller.signal.aborted)setFailure(publicErrorMessage(reason, t("图片上传失败", "Image upload failed")));}finally{if(uploadRequest.current===controller){uploadRequest.current=null;setUploading(false);}}
  }
  async function save(event: React.FormEvent) {
    event.preventDefault();
    if(!articleReady||uploadRequest.current||saveRequest.current)return;
    const controller=new AbortController();saveRequest.current=controller;setSaving(true);setFailure('');
    try {
      await client.request(id ? `/api/v1/content/articles/${id}` : '/api/v1/content/articles', isArticle, {method:id?'PUT':'POST',body:JSON.stringify(article),signal:controller.signal});
      if(controller.signal.aborted||saveRequest.current!==controller)return;
      notify(t("文章已保存", "Article saved")); window.location.hash='articles';
    } catch { if(!controller.signal.aborted)setFailure(t("文章保存失败，请检查内容后重试", "Unable to save article. Check the content and retry.")); }
    finally {if(saveRequest.current===controller){saveRequest.current=null;setSaving(false);}}
  }
  return <section className="xcss-content-stack"><PageHeader><div><h1>{id ? t("编辑文章", "Edit article") : t("新增文章", "Add article")}</h1><p>{t("支持 Markdown 正文，发布后可从网站访问。", "Write in Markdown. Published articles are available on the site.")}</p></div></PageHeader>
    {failure && <ErrorState>{failure}</ErrorState>}
    {!articleReady?<>{loadFailure?<ErrorState onRetry={()=>setLoadAttempt(value=>value+1)}>{loadFailure}</ErrorState>:<LoadingState/>}<Button onClick={()=>{window.location.hash='articles';}}>{t("取消", "Cancel")}</Button></>:<>
    <form className="editor-form" onSubmit={event => void save(event)}>
      <fieldset className="xocs-fieldset" disabled={saving}>
      <FormField label={t("标题", "Title")}><TextField required maxLength={120} value={article.article_title} onChange={event => change('article_title',event.target.value)} /></FormField>
      <div className="form-grid"><FormField label={t("分类", "Category")}><Select required value={article.sort_id} onChange={event => {change('sort_id',Number(event.target.value));change('label_id',0);}}><option value={0}>{t("选择分类", "Select a category")}</option>{categories.map(item => <option key={item.id} value={item.id}>{item.sort_name}</option>)}</Select></FormField><FormField label={t("标签", "Tags")}><Select required value={article.label_id} onChange={event => change('label_id',Number(event.target.value))}><option value={0}>{t("选择标签", "Select a tag")}</option>{labels.filter(item=>item.sort_id===article.sort_id).map(item=><option key={item.id} value={item.id}>{item.label_name}</option>)}</Select></FormField></div>
      <div className="form-grid"><FormField label={t("封面图片地址", "Cover image URL")}><TextField value={article.article_cover || ''} onChange={event => change('article_cover',event.target.value || null)} /></FormField><FormField label={t("上传封面图片", "Upload cover image")}><input type="file" accept="image/png,image/jpeg,image/gif,image/webp" disabled={uploading||saving} onChange={event=>{const file=event.target.files?.[0];event.target.value='';void uploadPicture(file,'cover');}}/></FormField></div>
      <FormField label={t("Markdown 正文", "Markdown content")}><textarea className="xcss-input editor-textarea" required value={article.article_content} onChange={event => change('article_content',event.target.value)} /></FormField>
      <div className="xcss-actions"><Button type="button" onClick={()=>setPreview(value=>!value)}>{preview?t("收起预览", "Hide preview"):t("预览正文", "Preview content")}</Button><FormField label={t("插入正文图片", "Insert content image")}><input type="file" accept="image/png,image/jpeg,image/gif,image/webp" disabled={uploading||saving} onChange={event=>{const file=event.target.files?.[0];event.target.value='';void uploadPicture(file,'content');}}/></FormField></div>
      {preview&&<div className="editor-markdown-preview article-markdown" dangerouslySetInnerHTML={{__html:previewHtml}}/>}
      <div className="form-grid"><FormField label={t("视频地址", "Video URL")}><TextField value={article.video_url || ''} onChange={event => change('video_url',event.target.value || null)} /></FormField>{!article.view_status&&<FormField label={t("文章访问密码", "Article access password")}><TextField type="password" minLength={12} placeholder={id?t("留空则保留原密码", "Leave empty to keep the current password"):t("至少 12 字节", "At least 12 bytes")} value={article.password || ''} onChange={event => change('password',event.target.value || null)} /></FormField>}</div>
      <div className="check-row"><label><input type="checkbox" checked={article.view_status} onChange={event => {change('view_status',event.target.checked);if(event.target.checked)change('password',null);}} />{t(" 公开访问（关闭后需设置密码）", " Public access (a password is required when disabled)")}</label><label><input type="checkbox" checked={article.recommend_status} onChange={event => change('recommend_status',event.target.checked)} />{t(" 推荐", " Recommended")}</label><label><input type="checkbox" checked={article.comment_status} onChange={event => change('comment_status',event.target.checked)} />{t(" 允许评论", " Allow comments")}</label></div>
      <div className="xcss-actions"><Button disabled={saving} onClick={() => {uploadRequest.current?.abort();uploadRequest.current=null;setUploading(false);window.location.hash='articles';}}>{t("取消", "Cancel")}</Button><Button type="submit" disabled={saving||uploading}>{uploading?t("图片上传中…", "Uploading image…"):saving?t("保存中…", "Saving…"):t("保存文章", "Save article")}</Button></div>
      </fieldset>
    </form>
    {id&&<ArticleNewsEditor id={id}/>}
    </>}
  </section>;
}

function ArticleNewsEditor({id}:{id:number}){
  const {client,notify}=useAdminApplication();
  const [entries,setEntries]=useState<NewsEntry[]>([]);const [content,setContent]=useState('');const [date,setDate]=useState('');const [failure,setFailure]=useState('');
  const [saving,setSaving]=useState(false);const savePending=useRef(false);
  useEffect(()=>{const controller=new AbortController();void client.request(`/api/v1/content/articles/${id}/news`,isNews,{signal:controller.signal}).then(setEntries).catch(()=>{if(!controller.signal.aborted)setFailure(t("文章进展加载失败", "Unable to load article updates"));});return()=>controller.abort();},[client,id]);
  async function publish(event:React.FormEvent){
    event.preventDefault();if(savePending.current)return;
    savePending.current=true;setSaving(true);
    try{
      const saved=await client.request(`/api/v1/content/articles/${id}/news`,isNewsEntry,{method:'POST',body:JSON.stringify({content,create_time:date ? new Date(date).toISOString() : null})});
      setEntries(current=>[saved,...current].sort((a,b)=>(b.create_time||'').localeCompare(a.create_time||'')));
      setContent('');setDate('');setFailure('');notify(t("文章进展已发布", "Article update published"));
    }catch(reason){setFailure(publicErrorMessage(reason, t("发布失败", "Unable to publish")));}
    finally{savePending.current=false;setSaving(false);}
  }
  async function remove(entryId:number){if(!window.confirm(t("确定删除这条文章进展？", "Delete this article update?")))return;try{await client.request(`/api/v1/content/articles/${id}/news/${entryId}`,isDelete,{method:'DELETE'});setEntries(current=>current.filter(item=>item.id!==entryId));notify(t("文章进展已删除", "Article update deleted"));}catch(reason){setFailure(publicErrorMessage(reason, t("删除失败", "Unable to delete")));}}
  return <section className="editor-form"><h2>{t("最新进展", "Recent updates")}</h2>{failure&&<ErrorState>{failure}</ErrorState>}<form className="editor-form" onSubmit={event=>void publish(event)}><fieldset className="xocs-fieldset" disabled={saving}><FormField label={t("进展内容", "Update content")}><textarea className="xcss-input" required maxLength={1024} value={content} onChange={event=>setContent(event.target.value)}/></FormField><FormField label={t("进展时间", "Update time")}><TextField type="datetime-local" value={date} onChange={event=>setDate(event.target.value)}/></FormField><Button type="submit" disabled={saving}>{saving?t("发布中…", "Publishing…"):t("发布进展", "Publish update")}</Button></fieldset></form>{entries.length?<Table aria-label={t("文章进展", "Article updates")}><thead><tr><th>{t("时间", "Time")}</th><th>{t("内容", "Content")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{entries.map(item=><tr key={item.id}><td>{item.create_time||'—'}</td><td className="comment-cell">{item.content}</td><td><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></td></tr>)}</tbody></Table>:<EmptyState>{t("暂无进展", "No updates yet")}</EmptyState>}</section>;
}

function CategoryManager() {
  const {client, notify} = useAdminApplication();
  const [categories, setCategories] = useState<Category[]>([]);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState(0);
  const [editing, setEditing] = useState<number | null>(null);
  const [failure, setFailure] = useState('');
  const [refresh, setRefresh] = useState(0);
  const [busy, setBusy] = useState(true);
  const [saving,setSaving]=useState(false);const savePending=useRef(false);
  useEffect(() => {
    const controller = new AbortController();setBusy(true);setFailure('');
    void client.request('/api/v1/categories',isCategories,{signal:controller.signal})
      .then(value => {if (!controller.signal.aborted) setCategories(value);})
      .catch(() => {if (!controller.signal.aborted) setFailure(t('分类加载失败','Unable to load categories'));})
      .finally(() => {if (!controller.signal.aborted) setBusy(false);});
    return () => controller.abort();
  },[client,refresh]);
  function edit(item:Category){setEditing(item.id);setName(item.sort_name);setDescription(item.sort_description||'');setPriority(item.priority??0);}
  function reset(){setEditing(null);setName('');setDescription('');setPriority(0);}
  async function save(event: React.FormEvent) {
    event.preventDefault();if(savePending.current)return;
    savePending.current=true;setSaving(true);
    try {
      await client.request(editing?`/api/v1/content/categories/${editing}`:'/api/v1/content/categories',isCategory,{method:editing?'PUT':'POST',body:JSON.stringify({name,description:description||null,priority})});
      reset();setRefresh(value=>value+1);notify(t("分类已保存", "Category saved"));setFailure('');
    } catch(reason) {setFailure(publicErrorMessage(reason, t("分类保存失败", "Unable to save category")));}
    finally{savePending.current=false;setSaving(false);}
  }
  async function remove(id:number){if(!window.confirm(t("确定删除这个分类？", "Delete this category?")))return;try{await client.request(`/api/v1/content/categories/${id}`,isDelete,{method:'DELETE'});setRefresh(value=>value+1);notify(t("分类已删除", "Category deleted"));}catch(reason){setFailure(publicErrorMessage(reason, t("分类删除失败", "Unable to delete category")));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("分类管理", "Categories")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setRefresh(value=>value+1)}>{failure}</ErrorState>}<fieldset className="xocs-fieldset" disabled={saving}><form className="inline-form" onSubmit={event=>void save(event)}><FormField label={t("分类名称", "Category name")}><TextField required maxLength={64} value={name} onChange={event=>setName(event.target.value)} /></FormField><FormField label={t("分类描述", "Category description")}><TextField maxLength={200} value={description} onChange={event=>setDescription(event.target.value)}/></FormField><FormField label={t("优先级", "Priority")}><TextField type="number" value={priority} onChange={event=>setPriority(Number(event.target.value)||0)}/></FormField><Button type="submit" disabled={saving}>{saving?t("保存中…", "Saving…"):editing?t("保存修改", "Save changes"):t("创建分类", "Create category")}</Button>{editing&&<Button type="button" onClick={reset}>{t("取消编辑", "Cancel editing")}</Button>}</form>{categories.length?<Table aria-label={t("分类列表", "Category list")}><thead><tr><th>{t("名称", "Name")}</th><th>{t("描述", "Description")}</th><th>{t("排序", "Order")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{categories.map(item=><tr key={item.id}><td>{item.sort_name}</td><td>{item.sort_description||'—'}</td><td>{item.priority??0}</td><td><div className="xcss-actions"><Button onClick={()=>edit(item)}>{t("编辑", "Edit")}</Button><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></div></td></tr>)}</tbody></Table>:busy?<LoadingState />:!failure&&<EmptyState>{t("暂无分类", "No categories yet")}</EmptyState>}</fieldset></section>;
}

function HomeArticleCard({item}:{item:ArticleSummary}){
  const excerpt=articleExcerpt(item.excerpt,75);
  return <a className="article-card" href={`/article/${item.id}`}>
    <div className="article-card-cover">{safeMediaUrl(item.article_cover)?<img src={safeMediaUrl(item.article_cover)||''} alt="" loading="lazy"/>:<span>{t("遇事不决，可问春风", "Let the spring breeze guide you")}</span>}</div>
    <div className="article-card-body"><small>{t("📅 发布于 ", "📅 Published ")}{item.create_time||t("最近", "Recently")}</small><h3>{item.article_title}</h3><p className="article-card-stats">🔥 {item.view_count}{t(" 热度 💬 ", " views 💬 ")}{item.comment_count}{t(" 评论 🧡 ", " comments 🧡 ")}{item.like_count}{t(" 点赞", " likes")}</p>{excerpt&&<p className="article-card-excerpt">{excerpt}</p>}<span className="article-card-tags">{item.sort_name&&<span>📁 {item.sort_name}</span>}{item.label_name&&<span>🏷 {item.label_name}</span>}{item.view_status===0&&<span>{t("🔒 密码文章", "🔒 Password protected")}</span>}</span></div>
  </a>;
}

function noticeList(value:string|null|undefined):string[]{if(!value)return [];try{const parsed:unknown=JSON.parse(value);if(Array.isArray(parsed))return parsed.filter((item):item is string=>typeof item==='string');}catch{/* Plain text notice. */}return value.split('\n').map(item=>item.trim()).filter(Boolean);}
function noticeText(value:string|null|undefined){return noticeList(value).filter(item=>!['推送标题：','推送封面：','推送链接：'].some(prefix=>item.startsWith(prefix))).join(' · ')||t("欢迎光临", "Welcome");}
function pushNotice(value:string|null|undefined){const lines=noticeList(value);const title=lines.find(item=>item.startsWith('推送标题：'))?.slice(5);const cover=lines.find(item=>item.startsWith('推送封面：'))?.slice(5);const link=lines.find(item=>item.startsWith('推送链接：'))?.slice(5);return title&&link?{title,cover,link}:null;}

function HomeAside({site,categories,recommendedArticles,labels,messages,articleTotal,viewTotal}:{site:SiteInfo|null;categories:Category[];recommendedArticles:ArticleSummary[];labels:PublicLabel[];messages:{message:string}[];articleTotal:number;viewTotal:number}){
  return <aside className="home-aside" aria-label={t("网站侧栏", "Site sidebar")}>
    <section className="aside-profile"><div className="aside-avatar"><img src={safeMediaUrl(site?.avatar||null)||'/legacy/avatar.jpg'} alt={t("站长头像", "Site owner avatar")}/></div><h2>{site?.web_name||'xocs'}</h2><div className="aside-stats"><span>{t("📖 文章", "📖 Articles")}<strong>{articleTotal}</strong></span><span>{t("📒 分类", "📒 Categories")}<strong>{categories.length}</strong></span><span>{t("🔥 访问量", "🔥 Views")}<strong>{viewTotal}</strong></span></div></section>
    <form className="aside-search" action="/search" method="get"><label htmlFor="home-search">{t("搜索", "Search")}</label><div><input id="home-search" name="q" maxLength={200} placeholder={t("搜索文章", "Search articles")}/><button type="submit" aria-label={t("搜索", "Search")}>⌕</button></div></form>
    {recommendedArticles.length>0&&<section className="aside-featured"><h2>{t("✧ 推荐位", "✧ Featured")}</h2>{recommendedArticles.slice(0,2).map(item=><a key={item.id} href={`/article/${item.id}`}>{safeMediaUrl(item.article_cover)&&<img src={safeMediaUrl(item.article_cover)||''} alt="" loading="lazy"/>}<strong>{item.article_title}</strong></a>)}</section>}
    {recommendedArticles.length>0&&<section className="aside-recommended"><h2>{t("🔥 推荐文章", "🔥 Recommended articles")}</h2>{recommendedArticles.map(item=><a href={`/article/${item.id}`} key={item.id}>{safeMediaUrl(item.article_cover)?<img src={safeMediaUrl(item.article_cover)||''} alt="" loading="lazy"/>:<span className="aside-recommend-cover">{t("诗意生活", "A poetic life")}</span>}<span>{item.article_title}<small>◷ {item.create_time||t("最近", "Recently")}</small></span></a>)}</section>}
    {labels.length>0&&<section className="aside-labels"><h2>{t("🏷 标签", "🏷 Tags")}</h2><div>{labels.slice(0,28).map(item=><a key={item.id} href={`/sort?sort_id=${item.sort_id}&label_id=${item.id}`}>{item.label_name}</a>)}</div></section>}
    {messages.length>0&&<section className="aside-latest"><h2>{t("💬 最新弹幕", "💬 Recent messages")}</h2><div>{messages.slice(0,9).map((item,index)=><p key={index}>{item.message}</p>)}</div></section>}
    {categories.length>0&&<section className="aside-categories" aria-label={t("分类速览", "Category overview")}>{categories.map((item,index)=><a key={item.id} href={`/sort?sort_id=${item.id}`} style={{background:['linear-gradient(110deg,#358bff,#15c6ff)','linear-gradient(110deg,#18c9a7,#1eebeb)','linear-gradient(110deg,#ff6655,#ffbf37)','linear-gradient(110deg,#8c72e9,#d3a4f4)'][index%4]}}><small>{t("速览", "Overview")}</small><strong>{item.sort_name}</strong><span>{item.sort_description||t("查看更多文章 →", "More articles →")}</span></a>)}</section>}
  </aside>;
}

function Site() {
  const [site,setSite] = useState<SiteInfo|null>(null);
  const [stats,setStats] = useState<{article_count:number;view_count:number}|null>(null);
  const [sections,setSections] = useState<HomeSection[]>([]);
  const [sectionItems,setSectionItems] = useState<Record<number,Page<ArticleSummary>>>({});
  const [article,setArticle] = useState<Article|null>(null);
  const [categories,setCategories] = useState<Category[]>([]);
  const [recommendedArticles,setRecommendedArticles] = useState<ArticleSummary[]>([]);
  const [labels,setLabels] = useState<PublicLabel[]>([]);
  const [messages,setMessages] = useState<{message:string}[]>([]);
  const [failure,setFailure] = useState('');
  const [needsPassword,setNeedsPassword] = useState(false);
  const [accessTips,setAccessTips] = useState<string|null>(null);
  const [password,setPassword] = useState('');
  const [pushOpen,setPushOpen]=useState(false);
  const articleId = /^\/article\/(\d+)$/.exec(window.location.pathname)?.[1];
  const push=useMemo(()=>pushNotice(site?.notices),[site?.notices]);
  useEffect(()=>{if(articleId||!push||!safeMediaUrl(push.link))return;const key=`xocs-push:${push.link}`;try{if(window.localStorage.getItem(key)==='seen')return;}catch{/* Storage may be disabled. */}const timer=window.setTimeout(()=>{setPushOpen(true);try{window.localStorage.setItem(key,'seen');}catch{/* Storage may be disabled. */}},2000);return()=>window.clearTimeout(timer);},[articleId,push]);
  useEffect(() => {
    const controller = new AbortController();
    void request('/api/v1/site', isSiteInfo,{signal:controller.signal}).then(setSite).catch(()=>setFailure(t("网站信息加载失败", "Unable to load site information")));
    void request('/api/v1/site/stats', isSiteStats,{signal:controller.signal}).then(setStats).catch(()=>{});
    void request('/api/v1/categories', isCategories,{signal:controller.signal}).then(setCategories).catch(()=>{});
    if (!articleId) void request('/api/v1/articles?size=5&recommended=true', isArticlePage,{signal:controller.signal}).then(result=>setRecommendedArticles(result.items)).catch(()=>{});
    if (!articleId) {void request('/api/v1/labels', isPublicLabels,{signal:controller.signal}).then(setLabels).catch(()=>{});void request('/api/v1/tree-hole', isWallPosts,{signal:controller.signal}).then(setMessages).catch(()=>{});}
    if (articleId) void request(`/api/v1/articles/${articleId}`, isArticle,{signal:controller.signal}).then(setArticle).catch(()=>{if(controller.signal.aborted)return;void request(`/api/v1/articles/${articleId}/access`, isArticleAccess,{signal:controller.signal}).then(access=>{if(access.password_required){setNeedsPassword(true);setAccessTips(access.tips);}else setFailure(t("文章加载失败", "Unable to load articles"));}).catch(()=>{if(!controller.signal.aborted)setFailure(t("文章不存在", "Article not found"));});});
    else void request('/api/v1/home-sections', isHomeSections,{signal:controller.signal}).then(setSections).catch(()=>setFailure(t("首页栏目加载失败", "Unable to load home sections")));
    return () => controller.abort();
  },[articleId]);
  useEffect(()=>{if(articleId||!sections.length)return;const controller=new AbortController();void Promise.all(sections.map(async section=>{const filter=section.kind==='recommended'?'&recommended=true':section.kind==='category'?`&sort_id=${section.sort_id}`:'';const result=await request(`/api/v1/articles?size=6${filter}`, isArticlePage,{signal:controller.signal});return [section.id,result] as const;})).then(results=>{if(!controller.signal.aborted)setSectionItems(Object.fromEntries(results));}).catch(()=>{if(!controller.signal.aborted)setFailure(t("首页文章加载失败", "Unable to load home articles"));});return()=>controller.abort();},[articleId,sections]);
  async function unlock(event: React.FormEvent) {
    event.preventDefault();
    try {const found = await request(`/api/v1/articles/${articleId}/unlock`, isArticle,{method:'POST',body:JSON.stringify({password})});setArticle(found);setNeedsPassword(false);setFailure('');setPassword('');}
    catch {setFailure(t("访问密码错误或文章不可用", "Incorrect password or article unavailable"));}
  }
  const latest=sections.find(section=>section.kind==='latest');
  return <PublicChrome site={site} home={!articleId} title={article?.article_title||t("文章", "Articles")} cover={article?.article_cover} articleMeta={article?{author:article.username,date:article.create_time,views:article.view_count,comments:article.comment_count,likes:article.like_count}:undefined}>
    {failure&&<p role="alert">{failure}</p>}
    {articleId?article?<ReadingArticle article={article}/>:needsPassword?<form className="reading-card password-form" onSubmit={event=>void unlock(event)}><h2>{t("这篇文章需要访问密码", "This article requires an access password")}</h2>{accessTips&&<p>{accessTips}</p>}<label>{t("访问密码", "Access password")}<input type="password" required value={password} onChange={event=>setPassword(event.target.value)} /></label><button type="submit">{t("阅读文章", "Read article")}</button></form>:failure?null:<p>{t("正在加载文章…", "Loading article…")}</p>:
    <div className="home-layout"><HomeAside site={site} categories={categories} recommendedArticles={recommendedArticles} labels={labels} messages={messages} articleTotal={stats?.article_count??(latest?sectionItems[latest.id]?.total||0:0)} viewTotal={stats?.view_count||0}/><div className="home-content"><div className="announcement"><span aria-hidden="true">🔊</span><p>{noticeText(site?.notices)}</p></div>{sections.map(section=>{const page=sectionItems[section.id];const more=section.kind==='category'?`/sort?sort_id=${section.sort_id}`:section.kind==='recommended'?'/sort?recommended=1':'/sort';return <section className="home-section" key={section.id} aria-label={section.title}><div className="section-heading"><h2><span className="section-squares" aria-hidden="true"/>{section.title}</h2><a href={more}><span className="more-chevrons" aria-hidden="true">❯❯</span> {t("更多", "More")}</a></div><div className="article-grid">{page?.items.map(item=><HomeArticleCard item={item} key={item.id}/>)}</div>{page&&page.items.length===0&&<div className="public-empty-state home-empty-section"><span aria-hidden="true">✎</span><h3>{t('这个栏目还没有文章。','No articles in this section yet.')}</h3><p>{t('浏览其它分类，或到留言页分享你的想法。','Browse other categories, or leave a thought on the message wall.')}</p><div><a href="/sort">{t('浏览文章','Browse articles')} <span aria-hidden="true">↗</span></a><a href="/message">{t('给我留言','Leave a message')} <span aria-hidden="true">↗</span></a></div></div>}</section>;})}</div></div>}{pushOpen&&push&&safeMediaUrl(push.link)&&<div className="push-backdrop" role="presentation" onClick={()=>setPushOpen(false)}><section className="push-dialog" role="dialog" aria-modal="true" aria-label={t("每日推荐", "Daily recommendation")} onClick={event=>event.stopPropagation()}><button className="push-close" type="button" aria-label={t("关闭每日推荐", "Close daily recommendation")} onClick={()=>setPushOpen(false)}>×</button><h2>{t("每日推荐", "Daily recommendation")}</h2><h3>{push.title}</h3>{safeMediaUrl(push.cover||null)&&<img src={safeMediaUrl(push.cover||null)||''} alt={t("推荐封面", "Recommended cover")}/>}<a href={safeMediaUrl(push.link)||undefined} target="_blank" rel="noopener noreferrer" onClick={()=>setPushOpen(false)}>{t("立即前往 →", "Visit now →")}</a></section></div>}
  </PublicChrome>;
}

function NotFoundPage(){return <PublicChrome title={t('页面不存在','Page not found')}><section className="public-empty-state"><h2>{t('页面不存在','Page not found')}</h2><a href="/">{t('返回首页','Back to home')}</a></section></PublicChrome>;}

function renderApplication() {
  createRoot(document.getElementById('root')!).render(<StrictMode><div style={{display:'contents'}} onClickCapture={preserveLinkLanguage}>{window.location.pathname.startsWith('/admin')?<div className="xocs-admin-app"><AdminApp /></div>:['/weiYan','/jotting','/menory','/message','/favorite','/friend','/music','/travel','/love','/sort','/search','/about','/letter'].includes(window.location.pathname)?<PublicExtras/>:window.location.pathname==='/'||/^\/article\/[^/]+$/.test(window.location.pathname)?<Site />:<NotFoundPage/>}</div></StrictMode>);
}

if (window.location.pathname.startsWith('/admin')) void startAfterFonts(renderApplication);
else renderApplication();
