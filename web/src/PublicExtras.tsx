import { isArticlePage, isCategories, isLinkClasses, isLinkPage, isLinks, isNotePage, isPublicLabels, isSiteInfo, isWallPost, isWallPosts, type Link, type LinkClass, type Note, type WallPost } from './public-contracts';
import { DisplayError } from './api';
import { publicErrorMessage } from './api';
import { t } from '@xcss/web/admin-ui/i18n';
import {useEffect,useRef,useState,type FormEvent,type ReactNode} from 'react';
import {articleExcerpt,request,type ArticleSummary,type Category,type Page,type PublicLabel,type ResponseValidator,type SiteInfo} from './api';
import {ImageLightbox,safeImageUrl} from './MediaPreview';
import {PublicChrome} from './PublicChrome';
import {PublicComments} from './PublicComments';
import {PhotoGrid} from './PhotoGallery';
import {LovePage} from './LovePage';

function safeHttpUrl(value:string|null){if(!value)return null;try{const parsed=new URL(value,window.location.origin);return ['http:','https:'].includes(parsed.protocol)?parsed.href:null;}catch{return null;}}

function Shell({title,subtitle,children}:{title:string;subtitle?:string;children:ReactNode}){
  return <PublicChrome title={title} subtitle={subtitle}>{children}</PublicChrome>;
}

function JourneyPage(){
  const [result,setResult]=useState<Page<ArticleSummary>|null>(null);
  const [page,setPage]=useState(1);
  const [error,setError]=useState('');
  useEffect(()=>{const controller=new AbortController();void request(`/api/v1/articles?page=${page}&size=20`, isArticlePage,{signal:controller.signal}).then(setResult).catch(()=>{if(!controller.signal.aborted)setError(t("游记加载失败", "Unable to load travels"));});return()=>controller.abort();},[page]);
  return <PublicChrome title="xocs" cover="/live/xocs-current-menory.jpg"><div className="journey-page"><header><strong>{t("时间线", "Timeline")}</strong><small>{t("灵魂在路上", "The soul is walking")}</small></header>{error&&<p role="alert">{error}</p>}<div className="journey-grid">{result?.items.map((item,index)=><a href={`/article/${item.id}`} className="journey-item" key={item.id}><span className="journey-image">{safeHttpUrl(item.article_cover)?<img src={safeHttpUrl(item.article_cover)||''} alt="" loading="lazy"/>:<span>xocs</span>}</span><strong>{item.article_title}</strong><small>{item.create_time?.slice(0,16)||t("最近", "Recently")} <span>{result.total-index-(page-1)*20}</span></small></a>)}</div>{result?.items.length===0&&<p className="journey-empty">{t("时光里的故事，正在路上。", "More stories are on their way.")}</p>}{result&&result.total>20&&<div className="pager"><button disabled={page<=1} onClick={()=>setPage(value=>value-1)}>{t("上一页", "Previous page")}</button><span>{t("第 {0} 页", "Page {0}", [page])}</span><button disabled={page*20>=result.total} onClick={()=>setPage(value=>value+1)}>{t("下一页", "Next page")}</button></div>}</div></PublicChrome>;
}

function Wall(){
  const jotting=window.location.pathname==='/jotting';
  const [result,setResult]=useState<Page<Note>|null>(null),[page,setPage]=useState(1);
  const [error,setError]=useState(''),[lightbox,setLightbox]=useState<string|null>(null);
  useEffect(()=>{const controller=new AbortController();setError('');void request(`/api/v1/notes/page?page=${page}`, isNotePage,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted)setResult(value);}).catch(reason=>{if(!controller.signal.aborted)setError(publicErrorMessage(reason,t('内容加载失败','Unable to load content')));});return()=>controller.abort();},[page]);
  return <PublicChrome title={jotting?'xocs':t('微言','Posts')} cover={jotting?'/live/xocs-current-jotting.png':undefined}><div className="wall-layout wall-reading-layout"><div className="wall-list">
    {error&&<p role="alert">{error}</p>}{result?.items.map(item=><article className="wall-card" key={item.id}><small>{item.username||t('站长','Site owner')} · {item.create_time||t('最近','Recently')}</small><p>{item.content}</p>{safeImageUrl(item.image_path)&&<button type="button" className="wall-photo-button" onClick={()=>setLightbox(safeImageUrl(item.image_path))} aria-label={t('放大图片','Enlarge image')}><img src={safeImageUrl(item.image_path)!} alt={t('动态图片','Post image')} loading="lazy"/></button>}</article>)}
    {result?.items.length===0&&<div className="public-empty-state"><h3>{t('还没有记录。','No entries yet.')}</h3><p>{t('新的日常记录正在路上。','New everyday stories are on their way.')}</p></div>}
    {result&&result.total>10&&<nav className="pager" aria-label={t('记录分页','Entry pages')}><button type="button" disabled={page<=1} onClick={()=>setPage(value=>value-1)}>{t('上一页','Previous page')}</button><span>{t('第 {0} 页','Page {0}',[page])}</span><button type="button" disabled={page*10>=result.total} onClick={()=>setPage(value=>value+1)}>{t('下一页','Next page')}</button></nav>}
  </div></div><ImageLightbox src={lightbox} onClose={()=>setLightbox(null)}/></PublicChrome>;
}

function MessagePage(){
  const [items,setItems]=useState<WallPost[]>([]);
  const [draft,setDraft]=useState('');
  const [error,setError]=useState('');
  const [busy,setBusy]=useState(false);
  useEffect(()=>{const controller=new AbortController();void request('/api/v1/tree-hole', isWallPosts,{signal:controller.signal}).then(setItems).catch(()=>{if(!controller.signal.aborted)setError(t("留言加载失败", "Unable to load messages"));});return()=>controller.abort();},[]);
  async function publish(event:FormEvent){
    event.preventDefault();
    if(!draft.trim()||busy)return;
    setBusy(true);setError('');
    try{const saved=await request('/api/v1/tree-hole/guest', isWallPost,{method:'POST',body:JSON.stringify({message:draft.trim()})});setItems(current=>[saved,...current]);setDraft('');}
    catch(reason){setError(publicErrorMessage(reason, t("发送失败", "Unable to send")));}
    finally{setBusy(false);}
  }
  return <PublicChrome plainHeader title={t("留言板", "Message board")}><div className="message-page"><section className="message-hero"><div className="message-barrage" aria-label={t("留言弹幕", "Message wall")}>{items.map((item,index)=><div key={item.id} className="message-barrage-item" style={{animationDelay:`${-(index%10)*2.3}s`,top:`${8+(index%7)*12}%`}}>{safeHttpUrl(item.avatar)&&<img src={safeHttpUrl(item.avatar)||''} alt=""/>}<span>{item.message}</span></div>)}</div><div className="message-form-wrap"><h1>{t("弹幕", "Message wall")}</h1><form onSubmit={event=>void publish(event)}><input aria-label={t("发送弹幕", "Send a message")} maxLength={60} value={draft} onChange={event=>setDraft(event.target.value)} placeholder={t("留下点什么啦～", "Leave a message…")}/><button disabled={busy||!draft.trim()}>{busy?t("发射中…", "Sending…"):t("发射", "Send")}</button></form><p className="message-anonymous">{t("弹幕将匿名发布，无需登录。", "Messages are posted anonymously. No account is needed.")}</p>{error&&<p role="alert">{error}</p>}</div></section><div className="message-comment-wrap"><PublicComments kind="message"/></div></div></PublicChrome>;
}

function TravelPage(){
  const [classes,setClasses]=useState<LinkClass[]>([]);const [selected,setSelected]=useState('');const [page,setPage]=useState(1);
  const [result,setResult]=useState<Page<Link>|null>(null);const [items,setItems]=useState<Link[]>([]);const [lightbox,setLightbox]=useState<string|null>(null);const [error,setError]=useState('');
  const [classesError,setClassesError]=useState(''),[classesBusy,setClassesBusy]=useState(false),[classesAttempt,setClassesAttempt]=useState(0);
  const classesLoading=useRef(false);
  useEffect(()=>{const controller=new AbortController();classesLoading.current=true;setClassesBusy(true);setClassesError('');void request('/api/v1/links/classes?kind=lovePhoto', isLinkClasses,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setClasses(value);setSelected(current=>current||value[0]?.classify||'');}}).catch(()=>{if(!controller.signal.aborted)setClassesError(t("相册分类加载失败", "Unable to load photo categories"));}).finally(()=>{if(!controller.signal.aborted){classesLoading.current=false;setClassesBusy(false);}});return()=>controller.abort();},[classesAttempt]);
  function retryClasses(){if(classesLoading.current)return;classesLoading.current=true;setClassesBusy(true);setClassesAttempt(value=>value+1);}
  const [busy,setBusy]=useState(false),[attempt,setAttempt]=useState(0);
  const loading=useRef(false),loadedPage=useRef(0);
  useEffect(()=>{loadedPage.current=0;setPage(1);setItems([]);setResult(null);},[selected]);
  useEffect(()=>{const controller=new AbortController();loading.current=true;setBusy(true);setError('');const filter=selected?`&classify=${encodeURIComponent(selected)}`:'';void request(`/api/v1/links/page?kind=lovePhoto&page=${page}&size=12${filter}`, isLinkPage,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){loadedPage.current=page;setResult(value);setItems(current=>page===1?value.items:[...current,...value.items.filter(item=>!current.some(existing=>existing.id===item.id))]);}}).catch(()=>{if(!controller.signal.aborted)setError(t("相册加载失败", "Unable to load photos"));}).finally(()=>{if(!controller.signal.aborted){loading.current=false;setBusy(false);}});return()=>controller.abort();},[selected,page,attempt]);
  function loadMore(){if(loading.current)return;loading.current=true;setBusy(true);setPage(loadedPage.current+1);setAttempt(value=>value+1);}
  function choose(value:string){if(value===selected)return;loadedPage.current=0;setSelected(value);setPage(1);setItems([]);setResult(null);}
  return <PublicChrome plainHeader title={t("时光相册", "Photo album")}><div className="travel-page"><section className="travel-banner"><div><h1>{t("时光相册", "Photo album")}</h1><h2>{t("每一张照片都是一次美好的记忆", "Every photo holds a memory")}</h2></div></section><div className="travel-content"><div className="original-tag-panel">{classesError&&<p role="alert">{classesError} <button type="button" disabled={classesBusy} onClick={retryClasses}>{t("重试分类", "Retry categories")}</button></p>}{classes.length>0&&<button className={!selected?'active':''} onClick={()=>choose('')}>{t("全部照片", "All photos")}</button>}{classes.map(item=><button className={selected===item.classify?'active':''} onClick={()=>choose(item.classify)} key={item.classify}>{item.classify} {item.count}</button>)}</div><h2>{selected||t("全部照片", "All photos")}</h2>{error&&<p role="alert">{error} <button type="button" disabled={busy} onClick={loadMore}>{t("重试", "Try again")}</button></p>}<PhotoGrid items={items} onPreview={setLightbox}/>{result&&items.length<result.total?<button className="travel-more" disabled={busy} onClick={loadMore}>{t("下一页", "Next page")}</button>:result&&<p className="travel-end">{t("~~到底啦~~", "~~You have reached the end~~")}</p>}</div></div><ImageLightbox src={lightbox} onClose={()=>setLightbox(null)}/></PublicChrome>;
}


function Favorites({friendOnly=false}:{friendOnly?:boolean}){
  const [tab,setTab]=useState<'friends'|'music'|'favorites'>(friendOnly?'friends':new URLSearchParams(window.location.search).get('tab')==='favorites'?'favorites':'friends');
  const [items,setItems]=useState<Link[]>([]);const [friends,setFriends]=useState<Link[]>([]);const [error,setError]=useState('');
  const [site,setSite]=useState<SiteInfo|null>(null);
  useEffect(()=>{void request('/api/v1/links?kind=favorites', isLinks).then(setItems).catch(()=>setError(t("收藏内容加载失败", "Unable to load favorites")));void request('/api/v1/links?kind=friendUrl', isLinks).then(setFriends).catch(()=>setError(t("友链加载失败", "Unable to load friend links")));void request('/api/v1/site', isSiteInfo).then(setSite).catch(()=>{});},[]);
  const groups=items.reduce<Record<string,Link[]>>((result,item)=>{const name=item.classify||t("我的收藏", "My favorites");(result[name]??=[]).push(item);return result;},{});
  const friendGroups=friends.reduce<Record<string,Link[]>>((result,item)=>{const name=item.classify||t("🥇友情链接", "🥇 Friend links");(result[name]??=[]).push(item);return result;},{});
  return <PublicChrome plainHeader title={friendOnly?t("友人帐", "Friends"):t("百宝箱", "Toolbox")}><div className="favorite-page">
    <section className="favorite-banner"><video autoPlay muted loop playsInline preload="metadata" poster="/legacy/backgroundPicture.jpg" src="/legacy/backgroundVideo.mp4"/><div className="favorite-banner-overlay"><small>{t("记录", "Articles")}</small><h1>{friendOnly?t("友人帐", "Friends"):t("百宝箱", "Toolbox")}</h1><div className="favorite-feature-links"><button className={tab==='friends'?'active':''} onClick={()=>setTab('friends')}><strong>{t("友人帐", "Friends")}</strong><span>{t("看看朋友们的网站", "Explore our friends’ websites")}</span></button><button className={tab==='music'?'active':''} onClick={()=>setTab('music')}><strong>{t("曲乐", "Music")}</strong><span>{t("一曲肝肠断，天涯何处觅知音", "Music brings kindred spirits together")}</span></button><button className={tab==='favorites'?'active':''} onClick={()=>setTab('favorites')}><strong>{t("收藏夹", "Favorites")}</strong><span>{t("将本网站添加到您的收藏夹吧", "Add this site to your favorites")}</span></button></div></div></section>
    <div className="favorite-groups">{error&&<p role="alert">{error}</p>}
      {tab==='friends'&&<div className="friend-page">
        <section className="friend-site-info"><h2>{t("🌸 本站信息", "🌸 Site information")}</h2><div><p><b>{t("网站名称", "Site name")}</b><span>{site?.web_name||'xocs'}</span></p><p><b>{t("网址", "URL")}</b><a href={window.location.origin}>{window.location.origin}</a></p>{site?.avatar&&<p><b>{t("头像", "Avatar")}</b><a href={site.avatar} target="_blank" rel="noopener noreferrer">{t("查看头像", "View avatar")}</a></p>}<p><b>{t("描述", "Description")}</b><span>{site?.web_title||t("有诗意地记录生活", "Record life through poetry")}</span></p></div></section>
        {[t("♥️青出于蓝", "♥️ Inspiration"),t("🥇友情链接", "🥇 Friend links"),...Object.keys(friendGroups).filter(name=>![t("♥️青出于蓝", "♥️ Inspiration"),t("🥇友情链接", "🥇 Friend links")].includes(name))].map(name=><FavoriteGroup key={name} name={name} items={friendGroups[name]||[]}/>)}
      </div>}
      {tab==='music'&&<MusicBody/>}
      {tab==='favorites'&&<>{Object.entries(groups).map(([name,group])=><FavoriteGroup key={name} name={name} items={group}/>)}{items.length===0&&<p className="favorite-empty">{t("还没有收藏的内容。", "No favorites yet.")}</p>}</>}
    </div>
  </div></PublicChrome>;
}

function FavoriteGroup({name,items}:{name:string;items:Link[]}){return <section className="favorite-link-group"><h2>{name}</h2><div className="favorite-link-grid">{items.map(item=>{const href=safeHttpUrl(item.url);const cover=safeHttpUrl(item.cover);return <a key={item.id} className="favorite-link-card" href={href||undefined} target={href?'_blank':undefined} rel="noopener noreferrer">{cover?<img src={cover} alt="" loading="lazy"/>:<span className="favorite-link-placeholder"/>}<span><strong>{item.title||t("未命名", "Untitled")}</strong><small>{item.introduction||''}</small></span></a>;})}</div>{items.length===0&&<p className="favorite-group-empty">{t("这里还没有友链。", "No friend links yet.")}</p>}</section>;}

function MusicPage(){
  return <Shell title={t("音乐盒", "Music player")}><MusicBody/></Shell>;
}
function MusicBody(){
  const [items,setItems]=useState<Link[]>([]);const [selected,setSelected]=useState(0);
  useEffect(()=>{void request('/api/v1/links?kind=funny', isLinks).then(setItems).catch(()=>{});},[]);
  const playable=items.filter(item=>safeHttpUrl(item.url));
  return <div className="music-layout"><div className="wall-card"><h2>{playable[selected]?.title||t("选择一首曲目", "Select a track")}</h2>{playable[selected]?.cover&&safeHttpUrl(playable[selected].cover)&&<img className="music-cover" src={safeHttpUrl(playable[selected].cover)||''} alt=""/>}{playable[selected]&&<audio key={playable[selected].id} controls src={safeHttpUrl(playable[selected].url)||undefined} onEnded={()=>setSelected(index=>Math.min(index+1,playable.length-1))}/>}</div><div className="wall-card"><h2>{t("播放列表", "Playlist")}</h2>{playable.map((item,index)=><button className={index===selected?'selected':''} key={item.id} onClick={()=>setSelected(index)}>{item.title||t("曲目 {0}", "Track {0}", [index+1])}<small>{item.classify}</small></button>)}{playable.length===0&&<p>{t("暂无曲目。", "No tracks yet.")}</p>}</div></div>;
}


function ArticleListCard({item}:{item:ArticleSummary}){
  const cover=safeHttpUrl(item.article_cover);
  const excerpt=articleExcerpt(item.search_snippet||item.excerpt,140);
  return <a className="original-list-card" href={`/article/${item.id}`}><div className="original-list-cover">{cover?<img src={cover} alt="" loading="lazy"/>:<span>{t("遇事不决，可问春风", "Let the spring breeze guide you")}</span>}</div><div className="original-list-body"><small>{t("📅 发布于 ", "📅 Published ")}{item.create_time||t("最近", "Recently")}</small><h3>{item.article_title}</h3><p className="original-list-stats">🔥 {item.view_count}{t(" 热度 📝 ", " views 📝 ")}{item.comment_count}{t(" 条评论 🧡 ", " comments 🧡 ")}{item.like_count}{t(" 点赞", " likes")}</p>{excerpt&&<p className="original-list-excerpt">{excerpt}</p>}<span className="original-list-tags"><span>📁 {item.sort_name||t("记录", "Articles")}</span>{item.label_name&&<span>🏷 {item.label_name}</span>}{item.view_status===0&&<span>{t("🔒 需要密码", "🔒 Password required")}</span>}</span></div></a>;
}

// Keep each result tied to its request, including same-query retries.
function useBrowseData<T>(url:string|null, validate:ResponseValidator<T>, message:string){
  const [attempt,setAttempt]=useState(0);
  const [loaded,setLoaded]=useState<{url:string;attempt:number;value:T|null;error:string}|null>(null);
  useEffect(()=>{
    if(!url)return;
    const controller=new AbortController();
    void request(url,validate,{signal:controller.signal})
      .then(value=>{if(!controller.signal.aborted)setLoaded({url,attempt,value,error:''});})
      .catch(()=>{if(!controller.signal.aborted)setLoaded({url,attempt,value:null,error:message});});
    return()=>controller.abort();
  },[url,validate,message,attempt]);
  const current=loaded?.url===url&&loaded.attempt===attempt?loaded:null;
  return {value:current?.value??null,error:current?.error||'',busy:!!url&&!current,retry:()=>setAttempt(value=>value+1)};
}

function Categories(){
  const params=new URLSearchParams(window.location.search);
  const initial=params.get('recommended')==='1'?-1:Number(params.get('sort_id')||0);
  const initialLabel=Number(params.get('label_id')||0);
  const [selected,setSelected]=useState(Number.isInteger(initial)?initial:0);
  const [label,setLabel]=useState(Number.isInteger(initialLabel)?initialLabel:0);
  const [page,setPage]=useState(1);
  const categoryData=useBrowseData('/api/v1/categories',isCategories,t("分类加载失败", "Unable to load categories"));
  const labelData=useBrowseData(selected>0?`/api/v1/labels?sort_id=${selected}`:null,isPublicLabels,t("标签加载失败", "Unable to load tags"));
  const filter=selected===-1?'&recommended=true':selected>0?`&sort_id=${selected}${label>0?`&label_id=${label}`:''}`:'';
  const articleData=useBrowseData(`/api/v1/articles?page=${page}&size=12${filter}`,isArticlePage,t("文章加载失败", "Unable to load articles"));
  const categories=categoryData.value||[],labels=labelData.value||[],articles=articleData.value;
  function choose(value:number){setSelected(value);setLabel(0);setPage(1);window.history.replaceState(null,'',value===-1?'/sort?recommended=1':value>0?`/sort?sort_id=${value}`:'/sort');}
  function chooseLabel(value:number){setLabel(value);setPage(1);window.history.replaceState(null,'',value>0?`/sort?sort_id=${selected}&label_id=${value}`:`/sort?sort_id=${selected}`);}
  return <Shell title={categories.find(item=>item.id===selected)?.sort_name||t("文章分类", "Article categories")} subtitle={t("就算风吹散了冰雪，想念也会留下来。", "Memories remain after the snow has melted.")}>
    <div className="original-tag-panel original-category-panel"><button className={selected===0?'active':''} onClick={()=>choose(0)}>{t("全部 ", "All ")}<small>{categories.reduce((sum,item)=>sum+item.article_count,0)}</small></button><button className={selected===-1?'active':''} onClick={()=>choose(-1)}>{t("推荐", "Recommended")}</button>{categories.map(item=><button className={selected===item.id?'active':''} key={item.id} onClick={()=>choose(item.id)}>{item.sort_name} <small>{item.article_count}</small></button>)}</div>
    {selected>0&&labels.length>0&&<div className="original-tag-panel original-label-panel"><button className={label===0?'active':''} onClick={()=>chooseLabel(0)}>{t("全部标签 ", "All tags ")}<small>{categories.find(item=>item.id===selected)?.article_count||0}</small></button>{labels.map(item=><button className={label===item.id?'active':''} onClick={()=>chooseLabel(item.id)} key={item.id}>{item.label_name} <small>{item.article_count}</small></button>)}</div>}
    {[categoryData,labelData,articleData].map((data,index)=>data.error&&<p role="alert" key={index}>{data.error} <button type="button" onClick={data.retry}>{t("重试", "Try again")}</button></p>)}{articleData.busy&&<p role="status">{t("正在加载文章…", "Loading articles…")}</p>}<div className="original-article-list"><h2>{t("🍃 发现", "🍃 Discover")}</h2>{articles?.items.map(item=><ArticleListCard item={item} key={item.id}/>)}{articles?.items.length===0&&<div className="original-empty-articles"><span aria-hidden="true">✿</span><h3>{t("还没有文章", "No articles yet")}</h3><p>{selected>0?t("这个分类的故事还在路上。", "Stories for this category are on their way."):t("第一篇记录正在路上。", "The first story is on its way.")}</p><a href="/">{t("回到首页", "Back to home")}</a></div>}{articles&&articles.total>0&&<div className="pager"><button disabled={page<=1} onClick={()=>setPage(page-1)}>{t("上一页", "Previous page")}</button><span>{t("第 {0} 页 · 共 {1} 篇", "Page {0} · {1} articles", [page, articles.total])}</span><button disabled={page*12>=articles.total} onClick={()=>setPage(page+1)}>{t("下一页", "Next page")}</button></div>}</div>
  </Shell>;
}

function Highlight({text,term}:{text:string;term:string}){if(!term)return text;const lower=text.toLocaleLowerCase();const needle=term.toLocaleLowerCase();const parts:ReactNode[]=[];let start=0;let index=lower.indexOf(needle,start);while(index!==-1){parts.push(text.slice(start,index));parts.push(<mark key={index}>{text.slice(index,index+term.length)}</mark>);start=index+term.length;index=lower.indexOf(needle,start);}parts.push(text.slice(start));return <>{parts}</>;}

function SearchPage(){
  const initial=(new URLSearchParams(window.location.search).get('q')||'').trim();const [draft,setDraft]=useState(initial);const [term,setTerm]=useState(initial);const [page,setPage]=useState(1);
  const data=useBrowseData(term?`/api/v1/articles?search=${encodeURIComponent(term)}&page=${page}&size=12`:null,isArticlePage,t("搜索失败", "Search failed"));
  const result=data.value,error=data.error;
  function submit(event:FormEvent){event.preventDefault();const next=draft.trim();if(next===term&&page===1)data.retry();setPage(1);setTerm(next);window.history.replaceState(null,'',`/search?q=${encodeURIComponent(next)}`);}
  return <Shell title={t("搜索文章", "Search articles")} subtitle={t("在文字里寻找熟悉的故事。", "Find familiar stories in these words.")}><form className="search-form" onSubmit={submit}><input aria-label={t("搜索标题或正文", "Search titles or content")} value={draft} maxLength={200} placeholder={t("搜索标题或正文", "Search titles or content")} onChange={event=>setDraft(event.target.value)}/><button type="submit">{t("搜索", "Search")}</button></form>{error&&<p role="alert">{error} <button type="button" onClick={data.retry}>{t("重试", "Try again")}</button></p>}{data.busy&&<p role="status">{t("正在搜索…", "Searching…")}</p>}{term&&result&&<p className="original-search-count">“{term}{t("” 找到 ", "” found ")}{result?.total??0}{t(" 篇文章；标题匹配优先。", " articles; title matches come first.")}</p>}<div className="original-article-list"><h2>{t("🍃 发现", "🍃 Discover")}</h2>{(!term||result?.items.length===0)&&<div className="public-empty-state search-empty-state"><span aria-hidden="true">⌕</span><h3>{term?t('没有找到匹配的文章','No matching articles'):t('输入关键词开始搜索','Enter a keyword to start searching')}</h3><p>{term?t('试试其它关键词，或浏览全部文章。','Try another keyword, or browse all articles.'):t('可以搜索文章标题，也可以寻找正文里的一句话。','Search for an article title or a phrase from its content.')}</p><a href="/sort">{t('浏览全部文章','Browse all articles')} <span aria-hidden="true">↗</span></a></div>}{result?.items.map(item=><a className="original-list-card" href={`/article/${item.id}`} key={item.id}><div className="original-list-cover">{safeHttpUrl(item.article_cover)?<img src={safeHttpUrl(item.article_cover)||''} alt="" loading="lazy"/>:<span>{t("遇事不决，可问春风", "Let the spring breeze guide you")}</span>}</div><div className="original-list-body"><small>{item.search_snippet?t("正文匹配", "Content match"):t("标题匹配", "Title match")} · {item.create_time}</small><h3><Highlight text={item.article_title} term={term}/></h3>{item.search_snippet&&<p className="search-snippet"><Highlight text={item.search_snippet} term={term}/></p>}<span className="original-list-tag">{item.view_status?t("公开", "Public"):t("需要访问密码", "Access password required")}</span></div></a>)}{result&&<div className="pager"><button disabled={page<=1} onClick={()=>setPage(page-1)}>{t("上一页", "Previous page")}</button><span>{t("第 {0} 页", "Page {0}", [page])}</span><button disabled={page*12>=result.total} onClick={()=>setPage(page+1)}>{t("下一页", "Next page")}</button></div>}</div></Shell>;
}

function About(){
  const [site,setSite]=useState<SiteInfo|null>(null);const [step,setStep]=useState(0);const [reply,setReply]=useState<string[]>([]);
  useEffect(()=>{void request('/api/v1/site', isSiteInfo).then(setSite).catch(()=>{});},[]);
  const chapters=[{talk:['Hi, there👋',t("欢迎来到这里。这里记录生活，也收藏一些喜欢的文字。", "Welcome. This is a place for stories from life and words we love.")],choices:[t("然后呢？ 😃", "Tell me more 😃"),t("少废话！ 🙄", "Get to the point 🙄")]},{talk:['😘',t("本站平时用于交流、分享和学习新知识。", "This site is for conversation, sharing, and learning."),t("如果内容涉及侵权，请联系站长处理，谢谢！", "Please contact the site owner about any copyright concerns.")],choices:[t("这个网站有什么用吗？ 😂", "What is this site for? 😂")]},{talk:[t("拥有自己的独立网站难道不酷吗🚀", "Having your own website is pretty cool 🚀"),t("那就摸鱼吧👋", "Time to relax 👋"),t("想说点什么，可以在留言板留下足迹🥝", "Leave a note on the message board 🥝")],choices:[]}];
  function answer(choice:string){setReply(current=>[...current,choice]);if(step===0&&choice.includes(t("少废话", "Get to the point")))setStep(3);else setStep(value=>value+1);}
  return <Shell title={t("关于", "About")}><div className="about-dialog-page"><h1>{t("两只毛驴鸣翠柳", "xocs")}</h1><section className="about-dialog"><h2>{t("与 ", "and ")}{site?.web_name||'xocs'}{t(" 对话中...", " Chatting…")}</h2>{chapters.slice(0,Math.min(step+1,chapters.length)).map((chapter,index)=><div key={index}><div className="about-bubbles">{chapter.talk.map(line=><p className="about-talk" key={line}>{line}</p>)}</div>{reply[index]&&<p className="about-reply">{reply[index]}</p>}</div>)}{step<chapters.length&&<div className="about-choices">{chapters[step].choices.map(choice=><button type="button" key={choice} onClick={()=>answer(choice)}>{choice}</button>)}</div>}{step>=chapters.length&&<p className="about-talk">👋 👋 👋</p>}{site?.notices&&<aside>{site.notices}</aside>}</section></div></Shell>;
}

function LetterPage(){const [open,setOpen]=useState(false);return <PublicChrome plainHeader title={t("信笺", "Letter")}><div className="letter-page"><div className="letter-sakura" aria-hidden="true">✿　❀　✿　❀　✿</div><div className={`letter-envelope${open?' opened':''}`}><div className="letter-envelope-front"><span>To&nbsp; Ming</span><button type="button" onClick={()=>setOpen(true)}>{t("打开信封", "Open letter")}</button></div><div className="letter-paper"><h1>To Ming</h1><p>{t("夜にはいつも寒いよね、でも、手を繋いでいると、暖かくなるよ！", "Cold nights feel warmer when we hold hands.")}</p><p>{t("どんなに寒い夜も、君と二人でいれば、ちっとも寒くない！", "No night feels cold when we are together.")}</p><strong>Hao</strong><button type="button" onClick={()=>setOpen(false)}>{t("合上信封", "Close letter")}</button></div></div></div></PublicChrome>;}

export function PublicExtras(){const path=window.location.pathname;switch(path){case '/weiYan':case '/jotting':return <Wall/>;case '/menory':return <JourneyPage/>;case '/message':return <MessagePage/>;case '/favorite':return <Favorites/>;case '/friend':return <Favorites friendOnly/>;case '/music':return <MusicPage/>;case '/travel':return <TravelPage/>;case '/love':return <LovePage/>;case '/sort':return <Categories/>;case '/search':return <SearchPage/>;case '/about':return <About/>;case '/letter':return <LetterPage/>;default:return null;}}
