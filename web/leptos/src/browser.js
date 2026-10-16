import { validationMessage, t, getLocale } from "@xcss/web/admin-ui/i18n";
import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';
const renderer = new MarkdownIt({html:false,linkify:true,breaks:true});
export function markdownHtml(content) { return DOMPurify.sanitize(renderer.render(content)); }
export function readingMarkup(content, copyLabel, imageLabel, sectionLabel) {
  const holder=document.createElement('div');holder.innerHTML=markdownHtml(content);
  const headings=[...holder.querySelectorAll('h2,h3,h4')].map((node,index)=>{const id=`section-${index+1}`;node.id=id;return {id,title:node.textContent?.trim()||`${sectionLabel} ${index+1}`,level:Number(node.tagName[1])};});
  holder.querySelectorAll('pre').forEach(pre=>{const button=document.createElement('button');button.type='button';button.className='article-copy-code';button.textContent=copyLabel;pre.prepend(button);});
  holder.querySelectorAll('img').forEach(img=>{img.setAttribute('role','button');img.tabIndex=0;img.setAttribute('aria-label',`${imageLabel}${img.alt||sectionLabel}`);});
  return JSON.stringify({html:holder.innerHTML,headings});
}
export function observeSliding(viewport) {
  const label=viewport.querySelector('.sliding-title-text');if(!label)return()=>{};
  let active=true;const measure=()=>{if(!active)return;const distance=Math.max(0,label.scrollWidth-viewport.clientWidth);viewport.style.setProperty('--title-travel',`${distance}px`);viewport.style.setProperty('--title-duration',`${Math.max(6,distance/28+3)}s`);viewport.classList.toggle('is-overflowing',distance>0);};
  const observer=new ResizeObserver(measure);observer.observe(viewport);observer.observe(label);measure();void document.fonts.ready.then(measure);return()=>{active=false;observer.disconnect();};
}

export function installDocumentPolicies() {
  const isControl = node => node instanceof HTMLInputElement || node instanceof HTMLSelectElement || node instanceof HTMLTextAreaElement;
  const invalid = event => { const node=event.target;if(!isControl(node)||node.validity.customError)return;node.setCustomValidity(node instanceof HTMLTextAreaElement ? node.validity.valueMissing?t('请填写此项。','Please fill out this field.'):node.validity.tooLong?t('输入内容过长。','The value is too long.'):node.validity.tooShort?t('输入内容过短。','The value is too short.'):t('请检查输入内容。','Please check this value.') : validationMessage(node)); };
  const clear = event => {if(isControl(event.target))event.target.setCustomValidity('');};
  const language = event => {if(!(event.target instanceof Element))return;const link=event.target.closest('a[href]');if(!link||link.hasAttribute('download')||link.getAttribute('href')?.startsWith('#'))return;const current=new URL(location.href);if(!current.searchParams.has('lang'))return;try{const url=new URL(link.href,location.href);if(url.origin===location.origin&&['http:','https:'].includes(url.protocol)){url.searchParams.set('lang',getLocale());link.href=url.pathname+url.search+url.hash;}}catch{}};
  document.addEventListener('invalid',invalid,true);document.addEventListener('input',clear,true);document.addEventListener('change',clear,true);document.addEventListener('click',language,true);
  return()=>{document.removeEventListener('invalid',invalid,true);document.removeEventListener('input',clear,true);document.removeEventListener('change',clear,true);document.removeEventListener('click',language,true);};
}
