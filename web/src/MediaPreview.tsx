import { t } from '@xcss/web/admin-ui/i18n';
import {useEffect,useRef} from 'react';

export function safeImageUrl(value:string|null|undefined){
  if(!value)return null;
  try{const url=new URL(value,window.location.origin);return ['http:','https:','blob:'].includes(url.protocol)?url.href:null;}
  catch{return null;}
}

export function ImageLightbox({src,onClose}:{src:string|null;onClose:()=>void}){
  const dialog=useRef<HTMLDialogElement>(null);
  const close=useRef(onClose);close.current=onClose;
  const open=!!src;
  useEffect(()=>{
    if(!open)return;
    const node=dialog.current;if(!node)return;
    const opener=document.activeElement;
    node.showModal();
    return()=>{node.close();if(opener instanceof HTMLElement && opener.isConnected)opener.focus();};
  },[open]);
  if(!src)return null;
  return <dialog ref={dialog} className="image-lightbox" aria-label={t("查看图片", "View image")} onCancel={event=>{event.preventDefault();close.current();}} onMouseDown={event=>{if(event.target===event.currentTarget)close.current();}}><button type="button" className="image-lightbox-close" onClick={()=>close.current()} aria-label={t("关闭图片", "Close image")}>×</button><img src={src} alt={t("放大的图片", "Enlarged image")} /></dialog>;
}
