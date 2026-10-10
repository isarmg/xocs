import { uploadFilename } from './upload-filename.ts';
import { publicErrorMessage } from './api';
import { t } from '@xcss/web/admin-ui/i18n';
import {useEffect,useRef,useState,type FormEvent} from 'react';
import {useAdminApplication} from '@xcss/web/admin-shell';
import {Button,EmptyState,ErrorState,FormField,LoadingState,PageHeader,Select,Table,TextField} from '@xcss/web/admin-ui';
import type {Category,Page,SiteInfo} from './api';
import {ImageLightbox,safeImageUrl} from './MediaPreview';
import {adminGroups} from './AdminLayout';

type User={id:number;username:string|null;email:string|null;phone_number:string|null;user_status:number;user_type:number;create_time:string|null};
type Comment={id:number;source:number;comment_type:string|null;user_id:number|null;username:string|null;comment_content:string;create_time:string|null};
type Resource={id:number;original_name:string|null;path:string|null;resource_type:string|null;size:number|null;status:number|null;create_time:string|null};
type Label={id:number;sort_id:number;label_name:string;label_description:string|null};
type Statistics={articles:number;comments:number;members:number;categories:number};
type Link={id:number;title:string|null;classify:string|null;cover:string|null;url:string|null;introduction:string|null;link_type:string|null;status:number|null};
type TreeHole={id:number;user_id:number|null;username:string|null;avatar:string|null;message:string;image_path:string|null;create_time:string|null};
type Note={id:number;user_id:number|null;username:string|null;content:string;image_path:string|null;is_public:number;create_time:string|null};
type Family={id:number;bg_cover:string|null;man_cover:string|null;woman_cover:string|null;man_name:string|null;woman_name:string|null;timing:string|null;countdown_title:string|null;countdown_time:string|null;family_info:string|null;status:number|null};

const isRecord=(value:unknown):value is Record<string,unknown>=>typeof value==='object'&&value!==null&&!Array.isArray(value);
const isPage=(value:unknown):value is Page<User|Comment|Resource>=>isRecord(value)&&Array.isArray(value.items)&&typeof value.total==='number';
const isList=(value:unknown):value is Label[]=>Array.isArray(value)&&value.every(item=>isRecord(item)&&typeof item.id==='number');
const isSite=(value:unknown):value is SiteInfo=>isRecord(value)&&'web_name'in value;
const isStatistics=(value:unknown):value is Statistics=>isRecord(value)&&['articles','comments','members','categories'].every(key=>typeof value[key]==='number');
const isStatus=(value:unknown):value is {active:boolean}=>isRecord(value)&&typeof value.active==='boolean';
const isDeleted=(value:unknown):value is {deleted:true}=>isRecord(value)&&value.deleted===true;
const isLink=(value:unknown):value is Link=>isRecord(value)&&Number.isSafeInteger(value.id)&&Number(value.id)>0&&['title','classify','cover','url','introduction','link_type'].every(key=>value[key]===null||typeof value[key]==='string')&&(value.status===null||value.status===0||value.status===1);
const isLinksPage=(value:unknown):value is Page<Link>=>isRecord(value)&&Array.isArray(value.items)&&Number.isSafeInteger(value.total)&&Number(value.total)>=0&&Number.isSafeInteger(value.page)&&Number(value.page)>0&&Number.isSafeInteger(value.size)&&Number(value.size)>0&&value.items.length<=Number(value.size)&&value.items.every(isLink);
const isTreePage=(value:unknown):value is Page<TreeHole>=>isRecord(value)&&Array.isArray(value.items)&&typeof value.total==='number'&&value.items.every(item=>isRecord(item)&&typeof item.id==='number'&&typeof item.message==='string');
const isNotesPage=(value:unknown):value is Page<Note>=>isRecord(value)&&Array.isArray(value.items)&&typeof value.total==='number'&&value.items.every(item=>isRecord(item)&&typeof item.id==='number'&&typeof item.content==='string');
function AdminPager({data,page,onPage}:{data:Page<unknown>|null;page:number;onPage:(page:number)=>void}){if(!data||(data.total<=data.size&&page<=1))return null;return <nav className="pager" aria-label={t("分页", "Pagination")}><Button disabled={page<=1} onClick={()=>onPage(page-1)}>{t("上一页", "Previous page")}</Button><span>{t("第 {0} 页 · 共 {1} 条", "Page {0} · {1} items", [page, data.total])}</span><Button disabled={page*data.size>=data.total} onClick={()=>onPage(page+1)}>{t("下一页", "Next page")}</Button></nav>;}

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

export function UsersPage(){
  const {client,notify}=useAdminApplication();const [data,setData]=useState<Page<User>|null>(null);const [failure,setFailure]=useState('');const [version,setVersion]=useState(0);const [currentPage,setCurrentPage]=useState(1);
  useEffect(()=>{const controller=new AbortController();void client.request(`/api/v1/content/users?page=${currentPage}&size=20`,isPage,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setData(value as Page<User>);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("用户列表加载失败", "Unable to load users"));});return()=>controller.abort();},[client,version,currentPage]);
  async function change(user:User){try{await client.request(`/api/v1/content/users/${user.id}/status`,isStatus,{method:'PUT',body:JSON.stringify({active:!user.user_status})});setVersion(v=>v+1);notify(t("用户状态已更新", "User status updated"));}catch{setFailure(t("用户状态修改失败", "Unable to update user status"));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("用户管理", "Users")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}{data?.items.length?<Table aria-label={t("用户列表", "User list")}><thead><tr><th>ID</th><th>{t("用户名", "Username")}</th><th>{t("邮箱", "Email")}</th><th>{t("状态", "Status")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(user=><tr key={user.id}><td>{user.id}</td><td>{user.username||'—'}</td><td>{user.email||'—'}</td><td>{user.user_status?t("正常", "Active"):t("已停用", "Disabled")}</td><td><Button onClick={()=>void change(user)}>{user.user_status?t("停用", "Disable"):t("启用", "Enable")}</Button></td></tr>)}</tbody></Table>:data?<EmptyState>{t("暂无用户", "No users yet")}</EmptyState>:!failure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage}/></section>;
}

export function CommentsPage(){
  const {client,notify}=useAdminApplication();const [data,setData]=useState<Page<Comment>|null>(null);const [failure,setFailure]=useState('');const [version,setVersion]=useState(0);const [currentPage,setCurrentPage]=useState(1);
  useEffect(()=>{const controller=new AbortController();void client.request(`/api/v1/content/comments?page=${currentPage}&size=20`,isPage,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setData(value as Page<Comment>);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("评论加载失败", "Unable to load comments"));});return()=>controller.abort();},[client,version,currentPage]);
  async function remove(id:number){if(!window.confirm(t("删除这条评论及其回复？", "Delete this comment and its replies?")))return;try{await client.request(`/api/v1/content/comments/${id}`,isDeleted,{method:'DELETE'});notify(t("评论已删除", "Comment deleted"));setVersion(v=>v+1);}catch{setFailure(t("删除评论失败", "Unable to delete comment"));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("评论管理", "Comments")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}{data?.items.length?<Table aria-label={t("评论列表", "Comment list")}><thead><tr><th>{t("来源", "Source")}</th><th>{t("用户", "Users")}</th><th>{t("内容", "Content")}</th><th>{t("时间", "Time")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(comment=><tr key={comment.id}><td>{comment.comment_type==='message'?t("留言板", "Message board"):comment.comment_type==='love'?<a href="/love" target="_blank" rel="noopener noreferrer">{t("祝福板", "Wishes")}</a>:<a href={`/article/${comment.source}`} target="_blank" rel="noopener noreferrer">{t("文章 #", "Article #")}{comment.source}</a>}</td><td>{comment.username||comment.user_id||t("匿名访客", "Anonymous guest")}</td><td className="comment-cell">{comment.comment_content}</td><td>{comment.create_time||'—'}</td><td><Button className="xcss-danger" onClick={()=>void remove(comment.id)}>{t("删除", "Delete")}</Button></td></tr>)}</tbody></Table>:data?<EmptyState>{t("暂无评论", "No comments yet")}</EmptyState>:!failure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage}/></section>;
}

export function SitePage(){
  const {client,notify}=useAdminApplication();
  const [site,setSite]=useState<SiteInfo|null>(null);
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
    setSaving(true);setFailure('');
    try{
      const updated=await client.request('/api/v1/content/site',isSite,{method:'PUT',body:JSON.stringify(site)});
      setSite(updated);notify(t('站点设置已保存','Site settings saved'));
    }catch(reason){setFailure(publicErrorMessage(reason,t('保存失败','Unable to save')));}
    finally{setSaving(false);}
  }
  function change(key:keyof SiteInfo,value:string){setSite(current=>current?{...current,[key]:value}:current);}
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
          </div>
          <FormField label={t('公告','Notices')}>
            <textarea className="xcss-input" aria-describedby="site-notices-help" value={site.notices||''} onChange={event=>change('notices',event.target.value)}/>
          </FormField>
          <p id="site-notices-help" className="xocs-field-help">{t('使用 JSON 字符串数组或每行一条公告。','Use a JSON string array or one notice per line.')}</p>
          <details><summary>{t('推送格式','Promotion format')}</summary><p className="xocs-field-help">{t('推送条目使用这些前缀：','Use these prefixes for promotion entries:')} <code>推送标题：</code> <code>推送封面：</code> <code>推送链接：</code></p></details>
          <FormField label={t('页脚','Footer')}><textarea className="xcss-input" value={site.footer||''} onChange={event=>change('footer',event.target.value)}/></FormField>
        </section>
        <section className="xcss-content-panel">
          <h2>{t('外观','Appearance')}</h2>
          <div className="form-grid">
            <FormField label={t('背景图片','Background image')}><TextField value={site.background_image||''} onChange={event=>change('background_image',event.target.value)}/></FormField>
            <FormField label={t('站长头像','Site owner avatar')}><TextField value={site.avatar||''} onChange={event=>change('avatar',event.target.value)}/></FormField>
          </div>
        </section>
        <section className="xcss-content-panel">
          <h2>{t('随机内容','Random content')}</h2>
          <p className="xocs-field-help">{t('使用 JSON 字符串数组或每行一个值；图片请填写图片地址。','Use a JSON string array or one value per line. For images, enter image URLs.')}</p>
          <FormField label={t('随机头像','Random avatars')}><textarea className="xcss-input" value={site.random_avatar||''} onChange={event=>change('random_avatar',event.target.value)}/></FormField>
          <FormField label={t('随机昵称','Random names')}><textarea className="xcss-input" value={site.random_name||''} onChange={event=>change('random_name',event.target.value)}/></FormField>
          <FormField label={t('随机封面','Random covers')}><textarea className="xcss-input" value={site.random_cover||''} onChange={event=>change('random_cover',event.target.value)}/></FormField>
        </section>
        <section className="xcss-content-panel">
          <h2>{t('高级设置','Advanced settings')}</h2>
          <FormField label={t('看板娘配置 JSON','Mascot configuration JSON')}><textarea className="xcss-input" value={site.waifu_json||''} onChange={event=>change('waifu_json',event.target.value)}/></FormField>
        </section>
      </fieldset>
      <div className="xcss-actions"><Button type="submit" disabled={saving}>{saving?t('保存中…','Saving…'):t('保存设置','Save settings')}</Button></div>
    </form>:!failure&&<LoadingState />}
  </section>;
}

export function LabelsPage(){
  const {client,notify}=useAdminApplication();const [labels,setLabels]=useState<Label[]>([]);const [categories,setCategories]=useState<Category[]>([]);const [name,setName]=useState('');const [description,setDescription]=useState('');const [sortId,setSortId]=useState(0);const [editing,setEditing]=useState<number|null>(null);const [failure,setFailure]=useState('');const [version,setVersion]=useState(0);
  const [saving,setSaving]=useState(false);const savePending=useRef(false);
  useEffect(()=>{const controller=new AbortController();void client.request('/api/v1/content/labels',isList,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setLabels(value);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("标签加载失败", "Unable to load tags"));});void client.request('/api/v1/categories',(v):v is Category[]=>Array.isArray(v),{signal:controller.signal}).then(value=>{if(!controller.signal.aborted)setCategories(value);}).catch(()=>{});return()=>controller.abort();},[client,version]);
  function edit(item:Label){setEditing(item.id);setSortId(item.sort_id);setName(item.label_name);setDescription(item.label_description||'');}
  function reset(){setEditing(null);setSortId(0);setName('');setDescription('');}
  async function save(event:FormEvent){
    event.preventDefault();if(savePending.current)return;
    savePending.current=true;setSaving(true);
    try{
      await client.request(editing?`/api/v1/content/labels/${editing}`:'/api/v1/content/labels',(v):v is Label=>isRecord(v)&&typeof v.id==='number',{method:editing?'PUT':'POST',body:JSON.stringify({sort_id:sortId,name,description:description||null})});
      reset();setVersion(v=>v+1);notify(t("标签已保存", "Tag saved"));setFailure('');
    }catch(reason){setFailure(editing&&typeof reason==='object'&&reason!==null&&'status'in reason&&reason.status===409?t("标签仍被文章使用，不能更改分类。请先修改文章的分类和标签。", "This tag is still used by articles. Update their categories and tags before moving this tag."):publicErrorMessage(reason, t("标签保存失败", "Unable to save tag")));}
    finally{savePending.current=false;setSaving(false);}
  }
  async function remove(id:number){if(!window.confirm(t("确定删除这个标签？", "Delete this tag?")))return;try{await client.request(`/api/v1/content/labels/${id}`,isDeleted,{method:'DELETE'});setVersion(v=>v+1);notify(t("标签已删除", "Tag deleted"));}catch(reason){setFailure(publicErrorMessage(reason, t("标签删除失败", "Unable to delete tag")));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("标签管理", "Tags")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}<fieldset className="xocs-fieldset" disabled={saving}><form className="inline-form" onSubmit={event=>void save(event)}><FormField label={t("分类", "Category")}><Select required value={sortId} onChange={event=>setSortId(Number(event.target.value))}><option value={0}>{t("选择分类", "Select a category")}</option>{categories.map(item=><option key={item.id} value={item.id}>{item.sort_name}</option>)}</Select></FormField><FormField label={t("标签名称", "Tag name")}><TextField value={name} required maxLength={32} onChange={event=>setName(event.target.value)}/></FormField><FormField label={t("标签描述", "Tag description")}><TextField value={description} maxLength={200} onChange={event=>setDescription(event.target.value)}/></FormField><Button type="submit" disabled={saving}>{saving?t("保存中…", "Saving…"):editing?t("保存修改", "Save changes"):t("新增标签", "Add tag")}</Button>{editing&&<Button type="button" onClick={reset}>{t("取消编辑", "Cancel editing")}</Button>}</form>{labels.length?<Table aria-label={t("标签列表", "Tag list")}><thead><tr><th>ID</th><th>{t("标签", "Tags")}</th><th>{t("描述", "Description")}</th><th>{t("分类", "Category")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{labels.map(item=><tr key={item.id}><td>{item.id}</td><td>{item.label_name}</td><td>{item.label_description||'—'}</td><td>{categories.find(category=>category.id===item.sort_id)?.sort_name||item.sort_id}</td><td><div className="xcss-actions"><Button onClick={()=>edit(item)}>{t("编辑", "Edit")}</Button><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></div></td></tr>)}</tbody></Table>:<EmptyState>{t("暂无标签", "No tags yet")}</EmptyState>}</fieldset></section>;
}

export function ResourcesPage(){
  const {client,notify}=useAdminApplication();const [data,setData]=useState<Page<Resource>|null>(null);const [failure,setFailure]=useState('');const [file,setFile]=useState<File|null>(null);const [uploaded,setUploaded]=useState('');const [version,setVersion]=useState(0);const [currentPage,setCurrentPage]=useState(1);
  useEffect(()=>{const controller=new AbortController();void client.request(`/api/v1/content/resources?page=${currentPage}&size=20`,isPage,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setData(value as Page<Resource>);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("文件列表加载失败", "Unable to load files"));});return()=>controller.abort();},[client,version,currentPage]);
  async function upload(event:FormEvent){event.preventDefault();if(!file)return;if(file.size>10*1024*1024){setFailure(t("图片不能超过 10 MB", "Images must not exceed 10 MB"));return;}try{const result=await client.request('/api/v1/content/upload',(value):value is {id:number;path:string}=>isRecord(value)&&typeof value.id==='number'&&typeof value.path==='string',{method:'POST',headers:{'Content-Type':file.type||'application/octet-stream','X-File-Name':uploadFilename(file.name)},body:file});setUploaded(result.path);setFile(null);setVersion(v=>v+1);notify(t("图片已上传", "Image uploaded"));setFailure('');}catch(reason){setFailure(publicErrorMessage(reason, t("上传失败", "Upload failed")));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("文件管理", "Files")}</h1><p>{t("上传图片后复制地址，用于文章封面或内容资源。", "Upload an image and copy its URL for article covers or content resources.")}</p></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}<form className="inline-form" onSubmit={event=>void upload(event)}><FormField label={t("上传图片（PNG、JPEG、GIF、WebP）", "Upload image (PNG, JPEG, GIF, WebP)")}><input type="file" accept="image/png,image/jpeg,image/gif,image/webp" onChange={event=>setFile(event.target.files?.[0]||null)}/></FormField><Button type="submit" disabled={!file}>{t("上传", "Upload")}</Button></form>{uploaded&&<p>{t("新图片地址：", "New image URL: ")}<code>{uploaded}</code></p>}{data?.items.length?<Table aria-label={t("文件列表", "File list")}><thead><tr><th>{t("名称", "Name")}</th><th>{t("类型", "Type")}</th><th>{t("大小", "Size")}</th><th>{t("位置", "Location")}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}><td>{item.original_name||t("文件 {0}", "File {0}", [item.id])}</td><td>{item.resource_type||'—'}</td><td>{item.size??'—'}</td><td>{item.path&&/^https?:\/\/|^\/media\//.test(item.path)?<a href={item.path} target="_blank" rel="noopener noreferrer">{t("打开", "Open")}</a>:'—'}</td></tr>)}</tbody></Table>:data?<EmptyState>{t("暂无文件", "No files yet")}</EmptyState>:!failure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage}/></section>;
}

const emptyLink={title:'',classify:'',cover:'',url:'',introduction:'',link_type:'favorites',status:true};
export function LinksPage(){
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
  async function save(event:FormEvent){event.preventDefault();try{await client.request(editing?`/api/v1/content/links/${editing}`:'/api/v1/content/links',isLink,{method:editing?'PUT':'POST',body:JSON.stringify({...form,classify:form.classify||null,cover:form.cover||null,url:form.url||null,introduction:form.introduction||null})});setForm(emptyLink);setEditing(null);reload();notify(t("资源已保存", "Resource saved"));setFailure('');}catch(reason){setFailure(publicErrorMessage(reason, t("保存资源失败", "Unable to save resource")));}}
  async function remove(id:number){if(!window.confirm(t("确定删除这条资源？", "Delete this resource?")))return;try{await client.request(`/api/v1/content/links/${id}`,isDeleted,{method:'DELETE'});reload();notify(t("资源已删除", "Resource deleted"));}catch{setFailure(t("删除资源失败", "Unable to delete resource"));}}
  async function approve(id:number){try{await client.request(`/api/v1/content/links/${id}/status`,isLink,{method:'PUT',body:JSON.stringify({active:true})});reload();notify(t("友链已审核通过", "Friend link approved"));}catch(reason){setFailure(publicErrorMessage(reason, t("审核失败", "Review failed")));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("内容资源", "Content resources")}</h1><p>{t("维护友链、收藏、照片和音乐条目。", "Manage friend links, favorites, photos, and music.")}</p></div></PageHeader>{(failure||loadFailure)&&<ErrorState onRetry={reload}>{failure||loadFailure}</ErrorState>}<div className="inline-form"><FormField label={t("类型筛选", "Filter by type")}><Select value={kind} onChange={event=>{const kind=event.target.value;setSelection(current=>({...current,kind,page:1}));}}><option value="">{t("全部", "All")}</option><option value="friendUrl">{t("友链", "Friend links")}</option><option value="favorites">{t("收藏", "Favorites")}</option><option value="lovePhoto">{t("照片", "Photos")}</option><option value="funny">{t("音乐与趣味", "Music and fun")}</option></Select></FormField><FormField label={t("状态筛选", "Filter by status")}><Select value={status} onChange={event=>{const status=Number(event.target.value);setSelection(current=>({...current,status,page:1}));}}><option value={-1}>{t("全部", "All")}</option><option value={0}>{t("待审核/隐藏", "Pending review / hidden")}</option><option value={1}>{t("已公开", "Published")}</option></Select></FormField></div><form className="editor-form" onSubmit={event=>void save(event)}><h2>{editing?t("编辑资源", "Edit resource"):t("新增资源", "Add resource")}</h2><div className="form-grid"><FormField label={t("类型", "Type")}><Select value={form.link_type} onChange={event=>change('link_type',event.target.value)}><option value="favorites">{t("收藏", "Favorites")}</option><option value="friendUrl">{t("友链", "Friend links")}</option><option value="lovePhoto">{t("照片", "Photos")}</option><option value="funny">{t("音乐与趣味", "Music and fun")}</option></Select></FormField><FormField label={t("分类", "Category")}><TextField maxLength={32} value={form.classify} onChange={event=>change('classify',event.target.value)}/></FormField></div><FormField label={t("标题", "Title")}><TextField required maxLength={64} value={form.title} onChange={event=>change('title',event.target.value)}/></FormField><FormField label={t("图片地址", "Image URL")}><TextField value={form.cover} onChange={event=>change('cover',event.target.value)}/></FormField><FormField label={t("目标地址", "Target URL")}><TextField value={form.url} onChange={event=>change('url',event.target.value)}/></FormField><FormField label={t("简介", "Introduction")}><TextField value={form.introduction} onChange={event=>change('introduction',event.target.value)}/></FormField><label><input type="checkbox" checked={form.status} onChange={event=>change('status',event.target.checked)}/>{t(" 公开显示", " Show publicly")}</label><div className="xcss-actions"><Button type="submit">{editing?t("保存修改", "Save changes"):t("新增资源", "Add resource")}</Button>{editing&&<Button onClick={()=>{setEditing(null);setForm(emptyLink);}}>{t("取消编辑", "Cancel editing")}</Button>}</div></form>{data?.items.length?<Table aria-label={t("内容资源列表", "Content resource list")}><thead><tr><th>{t("标题", "Title")}</th><th>{t("类型", "Type")}</th><th>{t("状态", "Status")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}><td>{item.title||'—'}</td><td>{item.link_type||'—'}</td><td>{item.status?t("公开", "Public"):t("待审核/隐藏", "Pending review / hidden")}</td><td><div className="xcss-actions"><Button onClick={()=>edit(item)}>{t("编辑", "Edit")}</Button>{item.link_type==='friendUrl'&&!item.status&&<Button onClick={()=>void approve(item.id)}>{t("审核通过", "Approve")}</Button>}<Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></div></td></tr>)}</tbody></Table>:data?<EmptyState>{data.total===0?t("暂无内容资源", "No content resources yet"):t("本页暂无内容资源", "No resources on this page")}</EmptyState>:!loadFailure&&<LoadingState />}<AdminPager data={data} page={page} onPage={setPage}/></section>;
}

export function TreeHolePage(){
  const {client,notify}=useAdminApplication();const [data,setData]=useState<Page<TreeHole>|null>(null);const [failure,setFailure]=useState('');const [version,setVersion]=useState(0);const [lightbox,setLightbox]=useState<string|null>(null);const [currentPage,setCurrentPage]=useState(1);
  useEffect(()=>{const controller=new AbortController();void client.request(`/api/v1/content/tree-hole?page=${currentPage}&size=20`,isTreePage,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setData(value);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("留言加载失败", "Unable to load messages"));});return()=>controller.abort();},[client,version,currentPage]);
  async function remove(id:number){if(!window.confirm(t("确定删除这条留言？", "Delete this message?")))return;try{await client.request(`/api/v1/content/tree-hole/${id}`,isDeleted,{method:'DELETE'});setVersion(v=>v+1);notify(t("留言已删除", "Message deleted"));}catch{setFailure(t("删除留言失败", "Unable to delete message"));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("留言板管理", "Message board")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}{data?.items.length?<Table aria-label={t("留言列表", "Message list")}><thead><tr><th>ID</th><th>{t("作者", "Author")}</th><th>{t("内容", "Content")}</th><th>{t("图片", "Image")}</th><th>{t("时间", "Time")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}><td>{item.id}</td><td>{item.username||(item.user_id?t("用户 {0}", "User {0}", [item.user_id]):t("匿名访客", "Anonymous guest"))}</td><td>{item.message||t("图片留言", "Image message")}</td><td>{safeImageUrl(item.image_path)&&<button type="button" className="moderation-photo" onClick={()=>setLightbox(safeImageUrl(item.image_path))}><img src={safeImageUrl(item.image_path)||''} alt={t("查看留言图片", "View message image")}/></button>}</td><td>{item.create_time||'—'}</td><td><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></td></tr>)}</tbody></Table>:data?<EmptyState>{t("暂无留言", "No messages yet")}</EmptyState>:!failure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage}/><ImageLightbox src={lightbox} onClose={()=>setLightbox(null)}/></section>;
}

export function NotesPage(){
  const {client,notify}=useAdminApplication();const [data,setData]=useState<Page<Note>|null>(null);const [failure,setFailure]=useState('');const [version,setVersion]=useState(0);const [lightbox,setLightbox]=useState<string|null>(null);const [currentPage,setCurrentPage]=useState(1);const [draft,setDraft]=useState('');const [isPublic,setIsPublic]=useState(true);
  useEffect(()=>{const controller=new AbortController();void client.request(`/api/v1/content/notes?page=${currentPage}&size=20`,isNotesPage,{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setData(value);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("动态加载失败", "Unable to load posts"));});return()=>controller.abort();},[client,version,currentPage]);
  async function publish(event:FormEvent){event.preventDefault();try{await client.request('/api/v1/content/notes',(value):value is Note=>isRecord(value)&&typeof value.id==='number',{method:'POST',body:JSON.stringify({content:draft,is_public:isPublic})});setDraft('');setCurrentPage(1);setVersion(v=>v+1);setFailure('');notify(t("微言已发布", "Post published"));}catch(reason){setFailure(publicErrorMessage(reason, t("发布失败", "Unable to publish")));}}
  async function remove(id:number){if(!window.confirm(t("确定删除这条动态？", "Delete this post?")))return;try{await client.request(`/api/v1/content/notes/${id}`,isDeleted,{method:'DELETE'});setVersion(v=>v+1);notify(t("动态已删除", "Post deleted"));}catch{setFailure(t("删除动态失败", "Unable to delete post"));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("动态管理", "Posts")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}<form className="editor-form" onSubmit={event=>void publish(event)}><FormField label={t("发布微言", "Publish post")}><textarea className="xcss-input" maxLength={1024} required value={draft} onChange={event=>setDraft(event.target.value)}/></FormField><label><input type="checkbox" checked={isPublic} onChange={event=>setIsPublic(event.target.checked)}/>{t(" 公开可见", " Publicly visible")}</label><Button type="submit">{t("发布", "Publish")}</Button></form>{data?.items.length?<Table aria-label={t("动态列表", "Post list")}><thead><tr><th>{t("作者", "Author")}</th><th>{t("内容", "Content")}</th><th>{t("图片", "Image")}</th><th>{t("状态", "Status")}</th><th>{t("时间", "Time")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{data.items.map(item=><tr key={item.id}><td>{item.username||t("用户 {0}", "User {0}", [item.user_id??t("未知", "Unknown")])}</td><td>{item.content||t("图片动态", "Image post")}</td><td>{safeImageUrl(item.image_path)&&<button type="button" className="moderation-photo" onClick={()=>setLightbox(safeImageUrl(item.image_path))}><img src={safeImageUrl(item.image_path)||''} alt={t("查看动态图片", "View post image")}/></button>}</td><td>{item.is_public?t("公开", "Public"):t("私密", "Private")}</td><td>{item.create_time||'—'}</td><td><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></td></tr>)}</tbody></Table>:data?<EmptyState>{t("暂无动态", "No posts yet")}</EmptyState>:!failure&&<LoadingState />}<AdminPager data={data} page={currentPage} onPage={setCurrentPage}/><ImageLightbox src={lightbox} onClose={()=>setLightbox(null)}/></section>;
}

const emptyFamily={bg_cover:'',man_cover:'',woman_cover:'',man_name:'',woman_name:'',timing:'',countdown_title:'',countdown_time:'',family_info:'',status:true};
export function FamilyPage(){
  const {client,notify}=useAdminApplication();const [families,setFamilies]=useState<Family[]>([]);const [form,setForm]=useState(emptyFamily);const [editing,setEditing]=useState<number|null>(null);const [failure,setFailure]=useState('');const [version,setVersion]=useState(0);
  useEffect(()=>{const controller=new AbortController();void client.request('/api/v1/content/family',(value):value is Family[]=>Array.isArray(value)&&value.every(item=>isRecord(item)&&typeof item.id==='number'),{signal:controller.signal}).then(value=>{if(!controller.signal.aborted){setFamilies(value);setFailure('');}}).catch(()=>{if(!controller.signal.aborted)setFailure(t("恋爱笔记加载失败", "Unable to load love journal"));});return()=>controller.abort();},[client,version]);
  function change<K extends keyof typeof emptyFamily>(key:K,value:(typeof emptyFamily)[K]){setForm(current=>({...current,[key]:value}));}
  function edit(item:Family){setEditing(item.id);setForm({bg_cover:item.bg_cover||'',man_cover:item.man_cover||'',woman_cover:item.woman_cover||'',man_name:item.man_name||'',woman_name:item.woman_name||'',timing:item.timing||'',countdown_title:item.countdown_title||'',countdown_time:item.countdown_time||'',family_info:item.family_info||'',status:item.status===1});}
  async function save(event:FormEvent){event.preventDefault();try{const payload={...form,bg_cover:form.bg_cover||null,man_cover:form.man_cover||null,woman_cover:form.woman_cover||null,countdown_title:form.countdown_title||null,countdown_time:form.countdown_time||null,family_info:form.family_info||null};await client.request(editing?`/api/v1/content/family/${editing}`:'/api/v1/content/family',(value):value is Family=>isRecord(value)&&typeof value.id==='number',{method:editing?'PUT':'POST',body:JSON.stringify(payload)});setForm(emptyFamily);setEditing(null);setVersion(v=>v+1);setFailure('');notify(t("恋爱笔记已保存", "Love journal saved"));}catch(reason){setFailure(publicErrorMessage(reason, t("保存失败", "Unable to save")));}}
  async function remove(id:number){if(!window.confirm(t("确定删除这条恋爱笔记？", "Delete this love journal entry?")))return;try{await client.request(`/api/v1/content/family/${id}`,isDeleted,{method:'DELETE'});setVersion(v=>v+1);notify(t("恋爱笔记已删除", "Love journal entry deleted"));}catch{setFailure(t("删除失败", "Unable to delete"));}}
  return <section className="xcss-content-stack"><PageHeader><div><h1>{t("恋爱笔记", "Love journal")}</h1></div></PageHeader>{failure&&<ErrorState onRetry={()=>setVersion(value=>value+1)}>{failure}</ErrorState>}<form className="editor-form" onSubmit={event=>void save(event)}><h2>{editing?t("编辑笔记", "Edit entry"):t("新增笔记", "Add entry")}</h2><div className="form-grid"><FormField label={t("男生昵称", "His name")}><TextField required maxLength={32} value={form.man_name} onChange={event=>change('man_name',event.target.value)}/></FormField><FormField label={t("女生昵称", "Her name")}><TextField required maxLength={32} value={form.woman_name} onChange={event=>change('woman_name',event.target.value)}/></FormField></div><div className="form-grid"><FormField label={t("起始时间", "Start date")}><TextField maxLength={32} value={form.timing} onChange={event=>change('timing',event.target.value)}/></FormField><FormField label={t("倒计时标题", "Countdown title")}><TextField maxLength={32} value={form.countdown_title} onChange={event=>change('countdown_title',event.target.value)}/></FormField></div><FormField label={t("倒计时时间", "Countdown date")}><TextField maxLength={32} value={form.countdown_time} onChange={event=>change('countdown_time',event.target.value)}/></FormField><FormField label={t("背景图片", "Background image")}><TextField value={form.bg_cover} onChange={event=>change('bg_cover',event.target.value)}/></FormField><div className="form-grid"><FormField label={t("男生头像", "His avatar")}><TextField value={form.man_cover} onChange={event=>change('man_cover',event.target.value)}/></FormField><FormField label={t("女生头像", "Her avatar")}><TextField value={form.woman_cover} onChange={event=>change('woman_cover',event.target.value)}/></FormField></div><FormField label={t("简介", "Introduction")}><textarea className="xcss-input" maxLength={1024} value={form.family_info} onChange={event=>change('family_info',event.target.value)}/></FormField><label><input type="checkbox" checked={form.status} onChange={event=>change('status',event.target.checked)}/>{t(" 公开显示", " Show publicly")}</label><div className="xcss-actions"><Button type="submit">{t("保存", "Save")}</Button>{editing&&<Button onClick={()=>{setEditing(null);setForm(emptyFamily);}}>{t("取消编辑", "Cancel editing")}</Button>}</div></form>{families.length?<Table aria-label={t("恋爱笔记列表", "Love journal entries")}><thead><tr><th>{t("标题", "Title")}</th><th>{t("时间", "Time")}</th><th>{t("状态", "Status")}</th><th>{t("操作", "Actions")}</th></tr></thead><tbody>{families.map(item=><tr key={item.id}><td>{item.man_name}{t(" 与 ", " and ")}{item.woman_name}</td><td>{item.timing||'—'}</td><td>{item.status?t("公开", "Public"):t("待审核 / 隐藏", "Pending review / hidden")}</td><td><div className="xcss-actions"><Button onClick={()=>edit(item)}>{t("编辑", "Edit")}</Button><Button className="xcss-danger" onClick={()=>void remove(item.id)}>{t("删除", "Delete")}</Button></div></td></tr>)}</tbody></Table>:<EmptyState>{t("暂无恋爱笔记", "No love journal entries yet")}</EmptyState>}</section>;
}
