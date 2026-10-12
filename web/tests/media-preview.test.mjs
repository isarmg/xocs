import assert from 'node:assert/strict';
import {test} from 'node:test';
import {HookHost,walk} from './fixtures/hook-host.mjs';
import {originalModule} from './fixtures/original-module.mjs';
const {ImageLightbox}=await originalModule(new URL('../src/MediaPreview.tsx',import.meta.url));

test('native dialog opens once per open lifetime and restores a connected opener',t=>{
 const oldDocument=globalThis.document,oldElement=globalThis.HTMLElement;
 class Element {isConnected=true;focus(){this.focusCount=(this.focusCount||0)+1;}}
 globalThis.HTMLElement=Element;const opener=new Element();globalThis.document={activeElement:opener,addEventListener(){},removeEventListener(){}};
 let shown=0,closed=0,firstClose=0,latestClose=0;
 const dialog={showModal(){shown++;},close(){closed++;}};
 const host=new HookHost(ImageLightbox,{src:'/one.png',onClose:()=>firstClose++});
 t.after(()=>{host.unmount();globalThis.document=oldDocument;globalThis.HTMLElement=oldElement;});
 const render=()=>host.render(tree=>{if(tree?.props.ref)tree.props.ref.current=dialog;});
 render();assert.equal(host.tree.type,'dialog');assert.equal(shown,1);
 host.props={src:'/two.png',onClose:()=>latestClose++};render();assert.equal(shown,1);assert.equal(closed,0);
 let prevented=false;host.tree.props.onCancel({preventDefault(){prevented=true;}});assert.equal(prevented,true);assert.equal(latestClose,1);assert.equal(firstClose,0);
 walk(host.tree,n=>n.type==='button')[0].props.onClick();assert.equal(latestClose,2);
 host.props={src:null,onClose:()=>{}};render();assert.equal(closed,1);assert.equal(opener.focusCount,1);
 opener.isConnected=false;host.props={src:'/three.png',onClose:()=>{}};render();assert.equal(shown,2);
 host.props={src:null,onClose:()=>{}};render();assert.equal(opener.focusCount,1);
});

test('captures the external opener before modal focusing, including React commit autofocus ordering',t=>{
 const oldDocument=globalThis.document,oldElement=globalThis.HTMLElement;
 class Element {isConnected=true;focus(){globalThis.document.activeElement=this;this.focusCount=(this.focusCount||0)+1;}}
 globalThis.HTMLElement=Element;
 const opener=new Element(),closeButton=new Element();globalThis.document={activeElement:opener};
 let activeBeforeModal;
 const dialog={showModal(){activeBeforeModal=document.activeElement;closeButton.focus();},close(){}};
 const host=new HookHost(ImageLightbox,{src:'/one.png',onClose(){}});
 t.after(()=>{host.unmount();globalThis.document=oldDocument;globalThis.HTMLElement=oldElement;});
 host.render(tree=>{
   tree.props.ref.current=dialog;
   // React focuses autoFocus nodes in its commit, before passive effects.
   for(const button of walk(tree,n=>n.type==='button'&&n.props.autoFocus))closeButton.focus();
 });
 assert.equal(activeBeforeModal,opener,'React must not move focus before the opener is captured');
 assert.equal(document.activeElement,closeButton,'showModal performs native initial focus');
 closeButton.isConnected=false;
 host.props={src:null,onClose(){}};host.render();
 assert.equal(document.activeElement,opener,'closing restores the connected external trigger');
});

test('closed dialogs cannot paint or expose focusable children before showModal',async()=>{
 const {readFile}=await import('node:fs/promises');
 const css=await readFile(new URL('../src/style.css',import.meta.url),'utf8');
 assert.match(css,/\.image-lightbox\{[^}]*display:none/);
 assert.match(css,/\.image-lightbox\[open\]\{display:flex\}/);
});
