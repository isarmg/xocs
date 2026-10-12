import { isCommentPage, isResourcePage, isList, isLabel, isSite, isStatistics, isDeleted, isLink, isLinksPage, isTreePage, isNotesPage, isNote, isFamily, isUploadedImage, isAdminLetter, type AdminLetter, type AdminSite, type Note, type Resource, type Label, type Statistics, type Link, type Family } from './admin-contracts.ts';
import { isCategories } from './public-contracts.ts';
import { formatUtcTime } from './utc-time.ts';
import { uploadFilename } from './upload-filename.ts';
import { publicErrorMessage } from './api';
import { t } from '@xcss/web/admin-ui/i18n';
import {useEffect,useRef,useState,type FormEvent} from 'react';
import {useAdminApplication} from '@xcss/web/admin-shell';
import {Button,EmptyState,ErrorState,FormField,LoadingState,PageHeader,Select,Table,TextField} from '@xcss/web/admin-ui';
import type {Category,Page,SiteInfo} from './api';
import {ImageLightbox,safeImageUrl} from './MediaPreview';
import {adminGroups} from './AdminLayout';
import {FilePicker} from './FilePicker.tsx';

function AdminPager({data,page,onPage,disabled=false}:{data:Page<unknown>|null;page:number;onPage:(page:number)=>void;disabled?:boolean}){if(!data||(data.total<=data.size&&page<=1))return null;return <nav className="pager" aria-label={t("分页", "Pagination")}><Button disabled={disabled||page<=1} onClick={()=>onPage(page-1)}>{t("上一页", "Previous page")}</Button><span>{t("第 {0} 页 · 共 {1} 条", "Page {0} · {1} items", [page, data.total])}</span><Button disabled={disabled||page*data.size>=data.total} onClick={()=>onPage(page+1)}>{t("下一页", "Next page")}</Button></nav>;}

// A new selection also identifies a refresh, so old rows disappear before effects run.
function useAdminPage<T>(client:ReturnType<typeof useAdminApplication>['client'],path:string,validate:(value:unknown)=>value is Page<T>,loadError:string){
  const [selection,setSelection]=useState({page:1});
  const [result,setResult]=useState<{client:typeof client;selection:typeof selection;data:Page<T>|null;failure:string}|null>(null);
  const [actionFailure,setFailure]=useState('');
  const current=result?.client===client&&result.selection===selection?result:null;
  const {page}=selection;
  function reload(){setSelection(current=>({...current}));}
  function setCurrentPage(page:number){setSelection({page});}
  useEffect(()=>{
    const controller=new AbortController();
    void client.request(`${path}?page=${page}&size=20`,(value):value is Page<T>=>validate(value)&&value.page===page&&value.size===20,{signal:controller.signal}).then(value=>{
      if(controller.signal.aborted)return;
      const lastPage=Math.max(1,Math.ceil(value.total/value.size));
      if(page>lastPage){setSelection(current=>current===selection?{page:lastPage}:current);return;}
      setResult({client,selection,data:value,failure:''});
    }).catch(()=>{
      if(!controller.signal.aborted)setResult({client,selection,data:null,failure:loadError});
    });
    return()=>controller.abort();
  },[client,path,validate,loadError,selection]);
  return {data:current?.data??null,loadFailure:current?.failure??'',failure:actionFailure,setFailure,currentPage:page,setCurrentPage,reload};
}

export function Dashboard(){
  const {client}=useAdminApplication();
  const [stats,setStats]=useState<Statistics|null>(null);
  const [failure,setFailure]=useState('');
  const [version,setVersion]=useState(0);
  useEffect(()=>{
    const controller=new AbortController();
    setFailure('');
    void client.request('/api/v1/content/statistics',isStatistics,{signal:controller.signal})
      .then(value=>{if(!controller.signal.aborted)setStats(value);})
      .catch(()=>{if(!controller.signal.aborted)setFailure(t("统计信息加载失败", "Unable to load statistics"));});
    return()=>controller.abort();
  },[client,version]);
  const metrics=stats?[[t('文章','Articles'),stats.articles],[t('评论','Comments'),stats.comments],
    [t('用户','Users'),stats.members],[t('分类','Categories'),stats.categories]] as const:[];
  return <section className="xcss-content-stack">
    <PageHeader><div><h1>{t('总览','Overview')}</h1><p>{t('网站内容与社区状态。','Site content and community status.')}</p></div></PageHeader>
    {failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}
    {stats?<Table className="xcss-statistics-table" aria-label={t('网站统计','Site statistics')}>
      <thead><tr><th scope="col">{t('指标','Metric')}</th><th scope="col">{t('数量','Count')}</th></tr></thead>
      <tbody>{metrics.map(([label,value])=><tr key={label}><th scope="row">{label}</th><td>{value.toLocaleString()}</td></tr>)}</tbody>
    </Table>:!failure&&<LoadingState />}
    <div className="xocs-dashboard-sections">
      {adminGroups.filter(group=>group.id!=='overview').map(group=><section className="xcss-content-panel" key={group.id}>
        <h2>{group.title}</h2><nav aria-label={group.title}>
          {group.pages.map(item=><a key={item.id} href={`/admin#${item.id}`}>{item.label}</a>)}
        </nav>
      </section>)}
    </div>
  </section>;
}

export function CommentsPage(){
  const {client,notify}=useAdminApplication();const {data,failure,loadFailure,setFailure,currentPage,setCurrentPage,reload}=useAdminPage(client,'/api/v1/content/comments',isCommentPage,t("评论加载失败", "Unable to load comments"));
  async function remove(id:number){if(!window.confirm(t("删除这条评论？如果是顶层评论，会同时删除该楼层的全部回复。", "Delete this comment? If it is a top-level comment, all replies in its thread will also be deleted.")))return;setFailure('');try{await client.request(`/api/v1/content/comments/${id}`,isDeleted,{method:'DELETE'});notify(t("评论已删除", "Comment deleted"));reload();}catch{setFailure(t("删除评论失败", "Unable to delete comment"));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("评论管理", "Comments")}</h1></div></PageHeader>{failure&&<ErrorState>{failure}</ErrorState>}{loadFailure&&<ErrorState onRetry={reload}>{loadFailure}</ErrorState>}{data?.items.length?<Table aria-label={t("评论列表", "Comment list")}><thead><tr><th>{t("来源", "Source")}</th><th>{t("用户", "Users")}</th><th>{t("内容", "Content")}</th><th>{t("时间", "Time")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(comment=><tr key={comment.id}><td>{comment.comment_type==='message'?t("留言板", "Message board"):comment.comment_type==='love'?<a href="/love" target="_blank" rel="noopener noreferrer">{t("祝福板", "Wishes")}</a>:<a href={`/article/${comment.source}`} target="_blank" rel="noopener noreferrer">{t("文章 #", "Article #")}{comment.source}</a>}</td><td>{comment.username||comment.user_id||t("匿名访客", "Anonymous guest")}</td><td className="comment-cell">{comment.comment_content}</td><td>{formatUtcTime(comment.create_time)||'—'}</td><td><Button className="xcss-danger" onClick={()=>void remove(comment.id)}>{t("删除", "Delete")}</Button></td></tr>)}</tbody></Table>:data?<EmptyState>{data.total===0?t("暂无评论", "No comments yet"):t("本页暂无评论", "No comments on this page")}</EmptyState>:!loadFailure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage}/></section>;
}

export function SitePage(){
  const {client,notify}=useAdminApplication();
  const [site,setSite]=useState<AdminSite|null>(null);
  const [failure,setFailure]=useState('');
  const [saving,setSaving]=useState(false);
  const [version,setVersion]=useState(0);
  useEffect(()=>{
    const controller=new AbortController();
    setFailure('');
    void client.request('/api/v1/content/site',isSite,{signal:controller.signal})
      .then(value=>{if(!controller.signal.aborted)setSite(value);})
      .catch(()=>{if(!controller.signal.aborted)setFailure(t('站点设置加载失败','Unable to load site settings'));});
    return()=>controller.abort();
  },[client,version]);
  async function save(event:FormEvent){
    event.preventDefault();if(!site)return;
    if(!site.author_name.trim()||Array.from(site.author_name.trim()).length>100){setFailure(t('站长显示名须为 1–100 个字符','The site owner display name must contain 1–100 characters'));return;}
    setSaving(true);setFailure('');
    try{
      const updated=await client.request('/api/v1/content/site',isSite,{method:'PUT',body:JSON.stringify(site)});
      setSite(updated);notify(t('站点设置已保存','Site settings saved'));
    }catch(reason){setFailure(publicErrorMessage(reason,t('保存失败','Unable to save')));}
    finally{setSaving(false);}
  }
  function change(key:keyof AdminSite,value:string){setSite(current=>current?{...current,[key]:value}:current);}
  return <section className="xcss-content-stack">
    <PageHeader><div><h1>{t('网站设置','Site settings')}</h1></div></PageHeader>
    {failure&&<ErrorState onRetry={site?undefined:()=>setVersion(value=>value+1)}>{failure}</ErrorState>}
    {site?<form className="xocs-settings-form" onSubmit={event=>void save(event)}>
      <fieldset className="xocs-fieldset" disabled={saving} aria-label={t('网站设置','Site settings')}>
        <section className="xcss-content-panel">
          <h2>{t('基本信息','Basic information')}</h2>
          <div className="form-grid">
            <FormField label={t('网站名称','Site name')}><TextField required value={site.web_name||''} onChange={event=>change('web_name',event.target.value)}/></FormField>
            <FormField label={t('网站标题','Site title')}><TextField value={site.web_title||''} onChange={event=>change('web_title',event.target.value)}/></FormField>
            <FormField label={t('站长显示名','Site owner display name')}><TextField required maxLength={200} value={site.author_name} onChange={event=>change('author_name',event.target.value)}/></FormField>
          </div>
          <p className="xocs-field-help">{t('用于文章和随笔署名；管理员登录名与密码在右上角头像的账号设置中修改。','Used as the author of articles and notes. Change administrator sign-in credentials through the account settings avatar.')}</p>
          <FormField label={t('公告','Notices')}>
            <textarea className="xcss-input" aria-describedby="site-notices-help" value={site.notices||''} onChange={event=>change('notices',event.target.value)}/>
          </FormField>
          <p id="site-notices-help" className="xocs-field-help">{t('使用 JSON 字符串数组或每行一条公告。','Use a JSON string array or one notice per line.')}</p>
          <details><summary>{t('推送格式','Promotion format')}</summary><p className="xocs-field-help">{t('推送条目使用这些前缀：','Use these prefixes for promotion entries:')} <code>推送标题：</code> <code>推送封面：</code> <code>推送链接：</code></p></details>
        </section>
        <section className="xcss-content-panel">
          <h2>{t('外观','Appearance')}</h2>
          <div className="form-grid">
            <FormField label={t('背景图片','Background image')}><TextField value={site.background_image||''} onChange={event=>change('background_image',event.target.value)}/></FormField>
            <FormField label={t('站长头像','Site owner avatar')}><TextField value={site.avatar||''} onChange={event=>change('avatar',event.target.value)}/></FormField>
          </div>
        </section>
        <section className="xcss-content-panel">
          <h2>{t('随机封面','Random covers')}</h2>
          <p className="xocs-field-help">{t('使用 JSON 图片地址数组或每行一个图片地址。','Use a JSON array of image URLs or one image URL per line.')}</p>
          <FormField label={t('随机封面','Random covers')}><textarea className="xcss-input" value={site.random_cover||''} onChange={event=>change('random_cover',event.target.value)}/></FormField>
        </section>
      </fieldset>
      <div className="xcss-actions"><Button type="submit" disabled={saving}>{saving?t('保存中…','Saving…'):t('保存设置','Save settings')}</Button></div>
    </form>:!failure&&<LoadingState />}
  </section>;
}

export function LabelsPage(){
  const {client,notify}=useAdminApplication();const [labels,setLabels]=useState<Label[]>([]);const [name,setName]=useState('');const [description,setDescription]=useState('');const [sortId,setSortId]=useState(0);const [editing,setEditing]=useState<number|null>(null);const [failure,setFailure]=useState('');const [version,setVersion]=useState(0);
  const [saving,setSaving]=useState(false);const savePending=useRef(false);
  const [categoryVersion,setCategoryVersion]=useState(0);
  const [categoryResult,setCategoryResult]=useState<{client:typeof client;version:number;data:Category[]|null;failure:string}|null>(null);
  const currentCategories=categoryResult?.client===client&&categoryResult.version===categoryVersion?categoryResult:null;
  const categories=currentCategories?.data??null;
  const canSave=categories?.some(category=>category.id===sortId)??false;
  useEffect(()=>{const controller=new AbortController();void client.request('/api/v1/content/labels',isList,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setLabels(value);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("标签加载失败", "Unable to load tags"));});return()=>controller.abort();},[client,version]);
  useEffect(()=>{
    const controller=new AbortController();
    void client.request('/api/v1/categories',isCategories,{signal:controller.signal}).then(value=>{
      if(!controller.signal.aborted)setCategoryResult({client,version:categoryVersion,data:value,failure:''});
    }).catch(()=>{
      if(!controller.signal.aborted)setCategoryResult({client,version:categoryVersion,data:null,failure:t("分类加载失败", "Unable to load categories")});
    });
    return()=>controller.abort();
  },[client,categoryVersion]);
  function edit(item:Label){setEditing(item.id);setSortId(item.sort_id);setName(item.label_name);setDescription(item.label_description||'');}
  function reset(){setEditing(null);setSortId(0);setName('');setDescription('');}
  async function save(event:FormEvent){
    event.preventDefault();if(savePending.current||!canSave)return;
    savePending.current=true;setSaving(true);
    try{
      await client.request(editing?`/api/v1/content/labels/${editing}`:'/api/v1/content/labels',isLabel,{method:editing?'PUT':'POST',body:JSON.stringify({sort_id:sortId,name,description:description||null})});
      reset();setVersion(v=>v+1);notify(t("标签已保存", "Tag saved"));setFailure('');
    }catch(reason){setFailure(editing&&typeof reason==='object'&&reason!==null&&'status'in reason&&reason.status===409?t("标签仍被文章使用，不能更改分类。请先修改文章的分类和标签。", "This tag is still used by articles. Update their categories and tags before moving this tag."):publicErrorMessage(reason, t("标签保存失败", "Unable to save tag")));}
    finally{savePending.current=false;setSaving(false);}
  }
  async function remove(id:number){if(!window.confirm(t("确定删除这个标签？", "Delete this tag?")))return;try{await client.request(`/api/v1/content/labels/${id}`,isDeleted,{method:'DELETE'});setVersion(v=>v+1);notify(t("标签已删除", "Tag deleted"));}catch(reason){setFailure(publicErrorMessage(reason, t("标签删除失败", "Unable to delete tag")));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("标签管理", "Tags")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}{currentCategories?.failure&&<ErrorState onRetry={()=>setCategoryVersion(value=>value+1)}>{currentCategories.failure}</ErrorState>}{!currentCategories&&<LoadingState>{t("正在加载分类…", "Loading categories…")}</LoadingState>}{categories?.length===0&&<p>{t("请先创建分类，再添加标签。", "Create a category before adding tags.")} <a href="/admin#categories">{t("分类管理", "Categories")}</a></p>}<fieldset className="xocs-fieldset" disabled={saving}><form className="inline-form" onSubmit={event=>void save(event)}><FormField label={t("分类", "Category")}><Select required disabled={!categories} value={sortId} onChange={event=>setSortId(Number(event.target.value))}><option value={0}>{t("选择分类", "Select a category")}</option>{categories?.map(item=><option key={item.id} value={item.id}>{item.sort_name}</option>)}</Select></FormField><FormField label={t("标签名称", "Tag name")}><TextField value={name} required maxLength={32} onChange={event=>setName(event.target.value)}/></FormField><FormField label={t("标签描述", "Tag description")}><TextField value={description} maxLength={200} onChange={event=>setDescription(event.target.value)}/></FormField><Button type="submit" disabled={saving||!canSave}>{saving?t("保存中…", "Saving…"):editing?t("保存修改", "Save changes"):t("新增标签", "Add tag")}</Button>{editing&&<Button type="button" onClick={reset}>{t("取消编辑", "Cancel editing")}</Button>}</form>{labels.length?<Table aria-label={t("标签列表", "Tag list")}><thead><tr><th>ID</th><th>{t("标签", "Tags")}</th><th>{t("描述", "Description")}</th><th>{t("分类", "Category")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{labels.map(item=><tr key={item.id}><td>{item.id}</td><td>{item.label_name}</td><td>{item.label_description||'—'}</td><td>{categories?.find(category=>category.id===item.sort_id)?.sort_name||item.sort_id}</td><td><div className="xcss-actions"><Button onClick={()=>edit(item)}>{t("编辑", "Edit")}</Button><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></div></td></tr>)}</tbody></Table>:<EmptyState>{t("暂无标签", "No tags yet")}</EmptyState>}</fieldset></section>;
}

export function ResourcesPage(){
  const [saving,setSaving]=useState(false);const savePending=useRef(false);const fileInput=useRef<HTMLInputElement|null>(null);
  const [deleting,setDeleting]=useState<number|null>(null);const deletePending=useRef<AbortController|null>(null);
  const {client,notify}=useAdminApplication();const {data,failure,loadFailure,setFailure,currentPage,setCurrentPage,reload}=useAdminPage(client,'/api/v1/content/resources',isResourcePage,t("文件列表加载失败", "Unable to load files"));const [file,setFile]=useState<File|null>(null);const [uploaded,setUploaded]=useState('');
  async function upload(event:FormEvent){event.preventDefault();if(savePending.current||deletePending.current)return;if(!file)return;if(file.size>10*1024*1024){setFailure(t("图片不能超过 10 MB", "Images must not exceed 10 MB"));return;}savePending.current=true;setSaving(true);setFailure('');try{const result=await client.request('/api/v1/content/upload',isUploadedImage,{method:'POST',headers:{'Content-Type':file.type||'application/octet-stream','X-File-Name':uploadFilename(file.name)},body:file});setUploaded(result.path);setFile(null);if(fileInput.current)fileInput.current.value='';reload();notify(t("图片已上传", "Image uploaded"));setFailure('');}catch(reason){setFailure(publicErrorMessage(reason, t("上传失败", "Upload failed")));}finally{savePending.current=false;setSaving(false);}}
  useEffect(()=>()=>{deletePending.current?.abort();deletePending.current=null;},[client]);
  async function remove(item:Resource){
    if(!item.path||savePending.current||deletePending.current||!window.confirm(t('确定删除这个文件及其图片数据？','Delete this file and its image data?')))return;
    const id=item.id;
    const controller=new AbortController();deletePending.current=controller;setDeleting(id);setFailure('');
    try{
      await client.request(`/api/v1/content/resources/${id}`,isDeleted,{method:'DELETE',signal:controller.signal,body:JSON.stringify({path:item.path})});
      if(controller.signal.aborted)return;
      reload();notify(t('文件已删除','File deleted'));
    }catch(reason){if(!controller.signal.aborted)setFailure(typeof reason==='object'&&reason!==null&&'status'in reason&&reason.status===409?t('文件仍被引用或记录已变化。请刷新列表，移除引用后再重试。','This file is referenced or its record has changed. Refresh the list and remove its references before retrying.'):publicErrorMessage(reason,t('删除文件失败','Unable to delete file')));}
    finally{if(deletePending.current===controller){deletePending.current=null;setDeleting(null);}}
  }
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("文件管理", "Files")}</h1><p>{t("上传图片后复制地址，用于文章封面或内容资源。", "Upload an image and copy its URL for article covers or content resources.")}</p></div></PageHeader>{failure&&<ErrorState onRetry={reload}>{failure}</ErrorState>}{loadFailure&&<ErrorState onRetry={reload}>{loadFailure}</ErrorState>}<fieldset className="xocs-fieldset" disabled={saving||deleting!==null}><form className="inline-form" onSubmit={event=>void upload(event)}><FilePicker label={t("上传图片（PNG、JPEG、GIF、WebP）", "Upload image (PNG, JPEG, GIF, WebP)")} inputRef={fileInput} filename={file?.name} accept="image/png,image/jpeg,image/gif,image/webp" onChange={event=>setFile(event.target.files?.[0]||null)}/><Button type="submit" disabled={!file||saving}>{saving?t("上传中…", "Uploading…"):t("上传", "Upload")}</Button></form></fieldset>{uploaded&&<p>{t("新图片地址：", "New image URL: ")}<code>{uploaded}</code></p>}{data?.items.length?<Table aria-label={t("文件列表", "File list")}><thead><tr><th>{t("名称", "Name")}</th><th>{t("类型", "Type")}</th><th>{t("大小", "Size")}</th><th>{t("位置", "Location")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}><td>{item.original_name||t("文件 {0}", "File {0}", [item.id])}</td><td>{item.resource_type||'—'}</td><td>{item.size??'—'}</td><td>{item.path&&/^https?:\/\/|^\/media\//.test(item.path)?<a href={item.path} target="_blank" rel="noopener noreferrer">{t("打开", "Open")}</a>:'—'}</td><td><Button className="xcss-danger" disabled={!item.path||saving||deleting!==null} title={!item.path?t('文件地址缺失，无法确认删除对象','The file address is missing, so the deletion target cannot be verified.'):undefined} onClick={()=>void remove(item)}>{deleting===item.id?t('删除中…','Deleting…'):t('删除','Delete')}</Button></td></tr>)}</tbody></Table>:data?<EmptyState>{data.total===0?t("暂无文件", "No files yet"):t("本页暂无文件", "No files on this page")}</EmptyState>:!loadFailure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage} disabled={saving||deleting!==null}/></section>;
}

const emptyLink={title:'',classify:'',cover:'',url:'',introduction:'',link_type:'favorites',status:true};
export function LinksPage(){
  const [saving,setSaving]=useState(false);const savePending=useRef(false);
  const {client,notify}=useAdminApplication();const [form,setForm]=useState(emptyLink);const [editing,setEditing]=useState<number|null>(null);const [failure,setFailure]=useState('');const [selection,setSelection]=useState({kind:'',status:-1,page:1});
  const [result,setResult]=useState<{client:typeof client;selection:typeof selection;data:Page<Link>|null;failure:string}|null>(null);
  const {kind,status,page}=selection;
  function reload(){setSelection(current=>({...current}));}
  function setPage(page:number){setSelection(current=>({...current,page}));}
  const current=result?.client===client&&result.selection===selection?result:null;
  const data=current?.data??null;
  const loadFailure=current?.failure??'';
  useEffect(()=>{
    const controller=new AbortController();
    const query=new URLSearchParams({page:String(page),size:'20'});
    if(kind)query.set('kind',kind);
    if(status>=0)query.set('status',String(status));
    setFailure('');
    void client.request(`/api/v1/content/links?${query}`,(value):value is Page<Link>=>isLinksPage(value)&&value.page===page&&value.size===20,{signal:controller.signal}).then(value=>{
      if(controller.signal.aborted)return;
      const lastPage=Math.max(1,Math.ceil(value.total/value.size));
      if(page>lastPage){setSelection(current=>current===selection?{...current,page:lastPage}:current);return;}
      setResult({client,selection,data:value,failure:''});
    }).catch(()=>{
      if(!controller.signal.aborted)setResult({client,selection,data:null,failure:t("资源列表加载失败", "Unable to load resources")});
    });
    return()=>controller.abort();
  },[client,selection]);
  function edit(item:Link){setEditing(item.id);setForm({title:item.title||'',classify:item.classify||'',cover:item.cover||'',url:item.url||'',introduction:item.introduction||'',link_type:item.link_type||'favorites',status:item.status===1});}
  function change<K extends keyof typeof emptyLink>(key:K,value:(typeof emptyLink)[K]){setForm(current=>({...current,[key]:value}));}
  async function save(event:FormEvent){event.preventDefault();if(savePending.current)return;savePending.current=true;setSaving(true);setFailure('');try{await client.request(editing?`/api/v1/content/links/${editing}`:'/api/v1/content/links',isLink,{method:editing?'PUT':'POST',body:JSON.stringify({...form,classify:form.classify||null,cover:form.cover||null,url:form.url||null,introduction:form.introduction||null})});setForm(emptyLink);setEditing(null);reload();notify(t("资源已保存", "Resource saved"));setFailure('');}catch(reason){setFailure(publicErrorMessage(reason, t("保存资源失败", "Unable to save resource")));}finally{savePending.current=false;setSaving(false);}}
  async function remove(id:number){if(!window.confirm(t("确定删除这条资源？", "Delete this resource?")))return;try{await client.request(`/api/v1/content/links/${id}`,isDeleted,{method:'DELETE'});reload();notify(t("资源已删除", "Resource deleted"));}catch{setFailure(t("删除资源失败", "Unable to delete resource"));}}
  async function approve(id:number){try{await client.request(`/api/v1/content/links/${id}/status`,isLink,{method:'PUT',body:JSON.stringify({active:true})});reload();notify(t("友链已审核通过", "Friend link approved"));}catch(reason){setFailure(publicErrorMessage(reason, t("审核失败", "Review failed")));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("内容资源", "Content resources")}</h1><p>{t("维护友链、收藏、照片和音乐条目。", "Manage friend links, favorites, photos, and music.")}</p></div></PageHeader>{(failure||loadFailure)&&<ErrorState onRetry={reload}>{failure||loadFailure}</ErrorState>}<div className="inline-form"><FormField label={t("类型筛选", "Filter by type")}><Select value={kind} onChange={event=>{const kind=event.target.value;setSelection(current=>({...current,kind,page:1}));}}><option value="">{t("全部", "All")}</option><option value="friendUrl">{t("友链", "Friend links")}</option><option value="favorites">{t("收藏", "Favorites")}</option><option value="lovePhoto">{t("照片", "Photos")}</option><option value="funny">{t("音乐与趣味", "Music and fun")}</option></Select></FormField><FormField label={t("状态筛选", "Filter by status")}><Select value={status} onChange={event=>{const status=Number(event.target.value);setSelection(current=>({...current,status,page:1}));}}><option value={-1}>{t("全部", "All")}</option><option value={0}>{t("待审核/隐藏", "Pending review / hidden")}</option><option value={1}>{t("已公开", "Published")}</option></Select></FormField></div><fieldset className="xocs-fieldset" disabled={saving}><form className="editor-form" onSubmit={event=>void save(event)}><h2>{editing?t("编辑资源", "Edit resource"):t("新增资源", "Add resource")}</h2><div className="form-grid"><FormField label={t("类型", "Type")}><Select value={form.link_type} onChange={event=>change('link_type',event.target.value)}><option value="favorites">{t("收藏", "Favorites")}</option><option value="friendUrl">{t("友链", "Friend links")}</option><option value="lovePhoto">{t("照片", "Photos")}</option><option value="funny">{t("音乐与趣味", "Music and fun")}</option></Select></FormField><FormField label={t("分类", "Category")}><TextField maxLength={32} value={form.classify} onChange={event=>change('classify',event.target.value)}/></FormField></div><FormField label={t("标题", "Title")}><TextField required maxLength={64} value={form.title} onChange={event=>change('title',event.target.value)}/></FormField><FormField label={t("图片地址", "Image URL")}><TextField value={form.cover} onChange={event=>change('cover',event.target.value)}/></FormField><FormField label={t("目标地址", "Target URL")}><TextField value={form.url} onChange={event=>change('url',event.target.value)}/></FormField><FormField label={t("简介", "Introduction")}><TextField value={form.introduction} onChange={event=>change('introduction',event.target.value)}/></FormField><label><input type="checkbox" checked={form.status} onChange={event=>change('status',event.target.checked)}/>{t(" 公开显示", " Show publicly")}</label><div className="xcss-actions"><Button type="submit" disabled={saving}>{saving?t("保存中…", "Saving…"):editing?t("保存修改", "Save changes"):t("新增资源", "Add resource")}</Button>{editing&&<Button onClick={()=>{setEditing(null);setForm(emptyLink);}}>{t("取消编辑", "Cancel editing")}</Button>}</div></form>{data?.items.length?<Table aria-label={t("内容资源列表", "Content resource list")}><thead><tr><th>{t("标题", "Title")}</th><th>{t("类型", "Type")}</th><th>{t("状态", "Status")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}><td>{item.title||'—'}</td><td>{item.link_type||'—'}</td><td>{item.status?t("公开", "Public"):t("待审核/隐藏", "Pending review / hidden")}</td><td><div className="xcss-actions"><Button onClick={()=>edit(item)}>{t("编辑", "Edit")}</Button>{item.link_type==='friendUrl'&&!item.status&&<Button onClick={()=>void approve(item.id)}>{t("审核通过", "Approve")}</Button>}<Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></div></td></tr>)}</tbody></Table>:data?<EmptyState>{data.total===0?t("暂无内容资源", "No content resources yet"):t("本页暂无内容资源", "No resources on this page")}</EmptyState>:!loadFailure&&<LoadingState />}<AdminPager data={data} page={page} onPage={setPage}/></fieldset></section>;
}

export function TreeHolePage(){
  const {client,notify}=useAdminApplication();const {data,failure,loadFailure,setFailure,currentPage,setCurrentPage,reload}=useAdminPage(client,'/api/v1/content/tree-hole',isTreePage,t("留言加载失败", "Unable to load messages"));const [lightbox,setLightbox]=useState<string|null>(null);
  async function remove(id:number){if(!window.confirm(t("确定删除这条留言？", "Delete this message?")))return;setFailure('');try{await client.request(`/api/v1/content/tree-hole/${id}`,isDeleted,{method:'DELETE'});reload();notify(t("留言已删除", "Message deleted"));}catch{setFailure(t("删除留言失败", "Unable to delete message"));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("留言板管理", "Message board")}</h1></div></PageHeader>{failure&&<ErrorState>{failure}</ErrorState>}{loadFailure&&<ErrorState onRetry={reload}>{loadFailure}</ErrorState>}{data?.items.length?<Table aria-label={t("留言列表", "Message list")}><thead><tr><th>ID</th><th>{t("作者", "Author")}</th><th>{t("内容", "Content")}</th><th>{t("图片", "Image")}</th><th>{t("时间", "Time")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}><td>{item.id}</td><td>{item.username||(item.user_id?t("用户 {0}", "User {0}", [item.user_id]):t("匿名访客", "Anonymous guest"))}</td><td>{item.message||t("图片留言", "Image message")}</td><td>{safeImageUrl(item.image_path)&&<button type="button" className="moderation-photo" onClick={()=>setLightbox(safeImageUrl(item.image_path))}><img src={safeImageUrl(item.image_path)||''} alt={t("查看留言图片", "View message image")}/></button>}</td><td>{formatUtcTime(item.create_time)||'—'}</td><td><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></td></tr>)}</tbody></Table>:data?<EmptyState>{data.total===0?t("暂无留言", "No messages yet"):t("本页暂无留言", "No messages on this page")}</EmptyState>:!loadFailure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage}/><ImageLightbox src={lightbox} onClose={()=>setLightbox(null)}/></section>;
}

export function NotesPage(){
  const {client,notify}=useAdminApplication();
  const {data,failure,loadFailure,setFailure,currentPage,setCurrentPage,reload}=useAdminPage(client,'/api/v1/content/notes',isNotesPage,t('随笔加载失败','Unable to load notes'));
  const [draft,setDraft]=useState(''),[isPublic,setIsPublic]=useState(true),[imagePath,setImagePath]=useState<string|null>(null);
  const [editing,setEditing]=useState<Note|null>(null),[lightbox,setLightbox]=useState<string|null>(null),[filename,setFilename]=useState('');
  const [saving,setSaving]=useState(false),[uploading,setUploading]=useState(false);
  const savePending=useRef<AbortController|null>(null),uploadPending=useRef<AbortController|null>(null);
  const busy=saving||uploading;
  const dirty=draft!==(editing?.content??'')||imagePath!==(editing?.image_path??null)||isPublic!==(editing?editing.is_public===1:true);
  useEffect(()=>()=>{savePending.current?.abort();uploadPending.current?.abort();savePending.current=null;uploadPending.current=null;},[client]);
  function reset(){uploadPending.current?.abort();uploadPending.current=null;setUploading(false);setEditing(null);setDraft('');setImagePath(null);setFilename('');setIsPublic(true);}
  function edit(item:Note){
    if(busy||(dirty&&!window.confirm(t('放弃当前未保存的随笔修改？','Discard the current unsaved note changes?'))))return;
    setEditing(item);setDraft(item.content);setImagePath(item.image_path);setIsPublic(item.is_public===1);setFilename('');setFailure('');
  }
  async function uploadImage(file:File|undefined){
    if(!file||savePending.current||uploadPending.current)return;
    if(file.size>10*1024*1024||!['image/png','image/jpeg','image/gif','image/webp'].includes(file.type)){setFailure(t('请选择不超过 10 MB 的 PNG、JPEG、GIF 或 WebP 图片','Choose a PNG, JPEG, GIF, or WebP image up to 10 MB'));return;}
    const controller=new AbortController();uploadPending.current=controller;setUploading(true);setFailure('');
    try{
      const result=await client.request('/api/v1/content/upload',isUploadedImage,{method:'POST',signal:controller.signal,headers:{'Content-Type':file.type,'X-File-Name':uploadFilename(file.name)},body:file});
      if(controller.signal.aborted||uploadPending.current!==controller)return;
      setImagePath(result.path);setFilename(file.name);notify(t('图片已上传','Image uploaded'));
    }catch(reason){if(!controller.signal.aborted)setFailure(publicErrorMessage(reason,t('图片上传失败','Image upload failed')));}
    finally{if(uploadPending.current===controller){uploadPending.current=null;setUploading(false);}}
  }
  async function save(event:FormEvent){
    event.preventDefault();if(savePending.current||uploadPending.current)return;
    if(!draft.trim()||Array.from(draft.trim()).length>1024){setFailure(t('随笔正文须为 1–1024 个字符','Note content must contain 1–1024 characters'));return;}
    const controller=new AbortController();savePending.current=controller;setSaving(true);setFailure('');
    const input={content:draft,image_path:imagePath,is_public:isPublic};
    try{
      await client.request(editing?`/api/v1/content/notes/${editing.id}`:'/api/v1/content/notes',
        (value):value is Note=>isNote(value)&&(!editing||value.id===editing.id)&&value.content===input.content.trim()&&value.image_path===input.image_path&&value.is_public===Number(input.is_public),
        {method:editing?'PUT':'POST',signal:controller.signal,body:JSON.stringify(input)});
      if(controller.signal.aborted)return;
      reset();setCurrentPage(1);reload();notify(t('随笔已保存','Note saved'));
    }catch(reason){if(!controller.signal.aborted)setFailure(publicErrorMessage(reason,t('保存随笔失败','Unable to save note')));}
    finally{if(savePending.current===controller){savePending.current=null;setSaving(false);}}
  }
  async function toggle(item:Note){
    if(savePending.current||uploadPending.current)return;
    const controller=new AbortController();savePending.current=controller;setSaving(true);setFailure('');const visible=item.is_public!==1;
    try{
      const saved=await client.request(`/api/v1/content/notes/${item.id}`,(value):value is Note=>isNote(value)&&value.id===item.id&&value.is_public===Number(visible)&&value.content===item.content.trim()&&value.image_path===item.image_path,
        {method:'PUT',signal:controller.signal,body:JSON.stringify({content:item.content,image_path:item.image_path,is_public:visible})});
      if(controller.signal.aborted)return;
      if(editing?.id===item.id){setEditing(saved);setIsPublic(visible);}
      reload();notify(visible?t('随笔已公开','Note published'):t('随笔已隐藏','Note hidden'));
    }catch(reason){if(!controller.signal.aborted)setFailure(publicErrorMessage(reason,t('修改随笔状态失败','Unable to change note visibility')));}
    finally{if(savePending.current===controller){savePending.current=null;setSaving(false);}}
  }
  async function remove(id:number){
    if(savePending.current||uploadPending.current||!window.confirm(t('确定删除这条随笔？','Delete this note?')))return;
    const controller=new AbortController();savePending.current=controller;setSaving(true);setFailure('');
    try{
      await client.request(`/api/v1/content/notes/${id}`,isDeleted,{method:'DELETE',signal:controller.signal});
      if(controller.signal.aborted)return;
      if(editing?.id===id)reset();reload();notify(t('随笔已删除','Note deleted'));
    }catch(reason){if(!controller.signal.aborted)setFailure(publicErrorMessage(reason,t('删除随笔失败','Unable to delete note')));}
    finally{if(savePending.current===controller){savePending.current=null;setSaving(false);}}
  }
  return <section className="xcss-content-stack">
    <PageHeader><div><h1>{t('随笔管理','Notes')}</h1><p>{t('维护随笔及恋爱日常中的内容。','Manage notes shown in the journal and everyday moments.')}</p></div></PageHeader>
    {failure&&<ErrorState>{failure}</ErrorState>}{loadFailure&&<ErrorState onRetry={reload}>{loadFailure}</ErrorState>}
    <form className="editor-form" onSubmit={event=>void save(event)}><fieldset className="xocs-fieldset" disabled={saving}>
      <h2>{editing?t('编辑随笔','Edit note'):t('新增随笔','Add note')}</h2>
      <FormField label={t('随笔正文','Note content')}><textarea className="xcss-input" aria-label={t('随笔正文','Note content')} maxLength={2048} required value={draft} onChange={event=>setDraft(event.target.value)}/></FormField>
      <FilePicker label={t('随笔图片','Note image')} filename={filename} disabled={busy} accept="image/png,image/jpeg,image/gif,image/webp" onChange={event=>{const file=event.target.files?.[0];event.target.value='';void uploadImage(file);}}/>
      {imagePath&&<div><button type="button" className="moderation-photo" onClick={()=>setLightbox(safeImageUrl(imagePath))}><img src={imagePath} alt={t('查看随笔图片','View note image')}/></button><Button disabled={busy} onClick={()=>{setImagePath(null);setFilename('');}}>{t('移除图片','Remove image')}</Button></div>}
      <label><input type="checkbox" checked={isPublic} onChange={event=>setIsPublic(event.target.checked)}/>{t(' 公开可见',' Publicly visible')}</label>
      <div className="xcss-actions"><Button type="submit" disabled={busy}>{uploading?t('图片上传中…','Uploading image…'):saving?t('保存中…','Saving…'):editing?t('保存修改','Save changes'):t('发布','Publish')}</Button>
        {(editing||dirty||uploading)&&<Button disabled={saving} onClick={()=>{if(!dirty||window.confirm(t('放弃当前未保存的随笔修改？','Discard the current unsaved note changes?')))reset();}}>{t('取消编辑','Cancel editing')}</Button>}</div>
    </fieldset></form>
    {data?.items.length?<Table aria-label={t('随笔列表','Note list')}><thead><tr><th>{t('作者','Author')}</th><th>{t('内容','Content')}</th><th>{t('图片','Image')}</th><th>{t('状态','Status')}</th><th>{t('时间','Time')}</th><th>{t('操作','Actions')}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}>
      <td>{item.username||t('站长','Site owner')}</td><td className="comment-cell">{item.content}</td><td>{safeImageUrl(item.image_path)&&<button type="button" className="moderation-photo" onClick={()=>setLightbox(safeImageUrl(item.image_path))}><img src={safeImageUrl(item.image_path)!} alt={t('查看随笔图片','View note image')}/></button>}</td><td>{item.is_public?t('公开','Public'):t('私密','Private')}</td><td>{formatUtcTime(item.create_time)||'—'}</td>
      <td><div className="xcss-actions"><Button disabled={busy} onClick={()=>edit(item)}>{t('编辑','Edit')}</Button><Button disabled={busy||(editing?.id===item.id&&dirty)} onClick={()=>void toggle(item)}>{item.is_public?t('隐藏','Hide'):t('公开','Publish')}</Button><Button className="xcss-danger" disabled={busy} onClick={()=>void remove(item.id)}>{t('删除','Delete')}</Button></div></td>
    </tr>)}</tbody></Table>:data?<EmptyState>{data.total===0?t('暂无随笔','No notes yet'):t('本页暂无随笔','No notes on this page')}</EmptyState>:!loadFailure&&<LoadingState/>}
    <AdminPager data={data} page={currentPage} onPage={setCurrentPage} disabled={busy}/><ImageLightbox src={lightbox} onClose={()=>setLightbox(null)}/>
  </section>;
}

const emptyLetter:AdminLetter={recipient:'',sender:'',content:'',is_public:false};
export function LetterPage(){
  const {client,notify}=useAdminApplication();
  const [form,setForm]=useState<AdminLetter>(emptyLetter),[original,setOriginal]=useState<AdminLetter|null>(null);
  const [failure,setFailure]=useState(''),[version,setVersion]=useState(0),[saving,setSaving]=useState(false);
  const pending=useRef<AbortController|null>(null);
  useEffect(()=>{
    const controller=new AbortController();setOriginal(null);setFailure('');
    void client.request('/api/v1/content/letter',isAdminLetter,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setForm(value);setOriginal(value);}}).catch(reason=>{if(!controller.signal.aborted)setFailure(publicErrorMessage(reason,t('信笺加载失败','Unable to load letter settings')));});
    return()=>{controller.abort();pending.current?.abort();pending.current=null;};
  },[client,version]);
  const dirty=original&&Object.keys(emptyLetter).some(key=>form[key as keyof AdminLetter]!==original[key as keyof AdminLetter]);
  async function save(event:FormEvent){
    event.preventDefault();if(!original||pending.current)return;
    const counts=[Array.from(form.recipient.trim()).length,Array.from(form.sender.trim()).length,Array.from(form.content.trim()).length];
    if(counts[0]<1||counts[0]>100||counts[1]<1||counts[1]>100||counts[2]<1||counts[2]>16384){setFailure(t('收信人与署名须为 1–100 个字符，正文须为 1–16384 个字符。','Recipient and sender must contain 1–100 characters; content must contain 1–16384 characters.'));return;}
    const controller=new AbortController();pending.current=controller;setSaving(true);setFailure('');
    try{
      const saved=await client.request('/api/v1/content/letter',isAdminLetter,{method:'PUT',signal:controller.signal,body:JSON.stringify(form)});
      if(controller.signal.aborted)return;
      setForm(saved);setOriginal(saved);notify(t('信笺已保存','Letter saved'));
    }catch(reason){if(!controller.signal.aborted)setFailure(publicErrorMessage(reason,t('保存信笺失败','Unable to save letter')));}
    finally{if(pending.current===controller){pending.current=null;setSaving(false);}}
  }
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t('信笺管理','Letter')}</h1><p>{t('维护公开信笺的收信人、署名和正文。','Manage the recipient, sender, and content of the public letter.')}</p></div></PageHeader>
    {failure&&<ErrorState onRetry={original?undefined:()=>setVersion(value=>value+1)}>{failure}</ErrorState>}
    {original?<form className="editor-form" onSubmit={event=>void save(event)}><fieldset className="xocs-fieldset" disabled={saving}>
      <FormField label={t('收信人','Recipient')}><TextField required maxLength={200} value={form.recipient} onChange={event=>setForm(current=>({...current,recipient:event.target.value}))}/></FormField>
      <FormField label={t('署名','Sender')}><TextField required maxLength={200} value={form.sender} onChange={event=>setForm(current=>({...current,sender:event.target.value}))}/></FormField>
      <FormField label={t('信笺正文','Letter content')}><textarea className="xcss-input" aria-label={t('信笺正文','Letter content')} required maxLength={32768} value={form.content} onChange={event=>setForm(current=>({...current,content:event.target.value}))}/></FormField>
      <label><input type="checkbox" checked={form.is_public} onChange={event=>setForm(current=>({...current,is_public:event.target.checked}))}/>{t(' 公开显示',' Show publicly')}</label>
      <div className="xcss-actions"><Button type="submit" disabled={saving}>{saving?t('保存中…','Saving…'):t('保存信笺','Save letter')}</Button><Button disabled={!dirty||saving} onClick={()=>{setForm(original);setFailure('');}}>{t('撤销修改','Reset changes')}</Button><a href="/letter" target="_blank" rel="noopener noreferrer">{t('查看公开页面','View public page')}</a></div>
    </fieldset></form>:!failure&&<LoadingState/>}
  </section>;
}

const emptyFamily={bg_cover:'',man_cover:'',woman_cover:'',man_name:'',woman_name:'',timing:'',countdown_title:'',countdown_time:'',family_info:'',status:true};
function familyForm(item:Family){return {bg_cover:item.bg_cover||'',man_cover:item.man_cover||'',woman_cover:item.woman_cover||'',man_name:item.man_name||'',woman_name:item.woman_name||'',timing:item.timing||'',countdown_title:item.countdown_title||'',countdown_time:item.countdown_time||'',family_info:item.family_info||'',status:item.status===1};}
export function FamilyPage(){
  const {client,notify}=useAdminApplication();
  const [form,setForm]=useState(emptyFamily),[editing,setEditing]=useState<number|null>(null);
  const [loaded,setLoaded]=useState(false),[saving,setSaving]=useState(false),[failure,setFailure]=useState(''),[version,setVersion]=useState(0);
  const savePending=useRef(false);
  useEffect(()=>{
    const controller=new AbortController();setLoaded(false);setFailure('');
    void client.request('/api/v1/content/family',(value):value is Family[]=>Array.isArray(value)&&value.length<=1&&value.every(isFamily),{signal:controller.signal})
      .then(value=>{if(!controller.signal.aborted){const home=value[0];setEditing(home?.id??null);setForm(home?familyForm(home):emptyFamily);setLoaded(true);}})
      .catch(reason=>{if(!controller.signal.aborted)setFailure(publicErrorMessage(reason,t("恋爱笔记加载失败", "Unable to load love journal")));});
    return()=>controller.abort();
  },[client,version]);
  function change<K extends keyof typeof emptyFamily>(key:K,value:(typeof emptyFamily)[K]){setForm(current=>({...current,[key]:value}));}
  async function save(event:FormEvent){
    event.preventDefault();if(!loaded||savePending.current)return;savePending.current=true;setSaving(true);setFailure('');
    try{
      const payload={...form,bg_cover:form.bg_cover||null,man_cover:form.man_cover||null,woman_cover:form.woman_cover||null,countdown_title:form.countdown_title||null,countdown_time:form.countdown_time||null,family_info:form.family_info||null};
      const home=await client.request(editing?`/api/v1/content/family/${editing}`:'/api/v1/content/family',isFamily,{method:editing?'PUT':'POST',body:JSON.stringify(payload)});
      setEditing(home.id);setForm(familyForm(home));notify(t("家的资料已保存", "Home details saved"));
    }catch(reason){setFailure(publicErrorMessage(reason,t("保存失败", "Unable to save")));}
    finally{savePending.current=false;setSaving(false);}
  }
  async function remove(){
    if(!editing||savePending.current||!window.confirm(t("确定删除家的资料？", "Delete your home details?")))return;
    savePending.current=true;setSaving(true);setFailure('');
    try{await client.request(`/api/v1/content/family/${editing}`,isDeleted,{method:'DELETE'});setVersion(value=>value+1);notify(t("家的资料已删除", "Home details deleted"));}
    catch(reason){setFailure(publicErrorMessage(reason,t("删除失败", "Unable to delete")));}
    finally{savePending.current=false;setSaving(false);}
  }
  return <section className="xcss-content-stack">
    <PageHeader><div><h1>{t("恋爱笔记", "Love journal")}</h1><p>{t("这里仅由管理员创建并维护一份家的资料。", "The administrator creates and maintains one home here.")}</p></div></PageHeader>
    {failure&&<ErrorState onRetry={loaded?undefined:()=>setVersion(value=>value+1)}>{failure}</ErrorState>}
    {!loaded&&!failure&&<LoadingState>{t("正在加载家的资料…", "Loading home details…")}</LoadingState>}
    {loaded&&<fieldset className="xocs-fieldset" disabled={saving}><form className="editor-form" onSubmit={event=>void save(event)}>
      <h2>{editing?t("编辑家的资料", "Edit home details"):t("创建我的家", "Create my home")}</h2>
      <div className="form-grid"><FormField label={t("男生昵称", "His name")}><TextField required maxLength={32} value={form.man_name} onChange={event=>change('man_name',event.target.value)}/></FormField><FormField label={t("女生昵称", "Her name")}><TextField required maxLength={32} value={form.woman_name} onChange={event=>change('woman_name',event.target.value)}/></FormField></div>
      <div className="form-grid"><FormField label={t("起始时间", "Start date")}><TextField maxLength={32} value={form.timing} onChange={event=>change('timing',event.target.value)}/></FormField><FormField label={t("倒计时标题", "Countdown title")}><TextField maxLength={32} value={form.countdown_title} onChange={event=>change('countdown_title',event.target.value)}/></FormField></div>
      <FormField label={t("倒计时时间", "Countdown date")}><TextField maxLength={32} value={form.countdown_time} onChange={event=>change('countdown_time',event.target.value)}/></FormField>
      <FormField label={t("背景图片", "Background image")}><TextField value={form.bg_cover} onChange={event=>change('bg_cover',event.target.value)}/></FormField>
      <div className="form-grid"><FormField label={t("男生头像", "His avatar")}><TextField value={form.man_cover} onChange={event=>change('man_cover',event.target.value)}/></FormField><FormField label={t("女生头像", "Her avatar")}><TextField value={form.woman_cover} onChange={event=>change('woman_cover',event.target.value)}/></FormField></div>
      <FormField label={t("简介", "Introduction")}><textarea className="xcss-input" maxLength={1024} value={form.family_info} onChange={event=>change('family_info',event.target.value)}/></FormField>
      <label><input type="checkbox" checked={form.status} onChange={event=>change('status',event.target.checked)}/>{t(" 公开显示", " Show publicly")}</label>
      <div className="xcss-actions"><Button type="submit">{saving?t("保存中…", "Saving…"):t("保存", "Save")}</Button>{editing&&<Button className="xcss-danger" onClick={()=>void remove()}>{t("删除家的资料", "Delete home details")}</Button>}</div>
    </form></fieldset>}
  </section>;
}
