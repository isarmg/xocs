from pathlib import Path
import json,re,shutil,os,collections
import tomlkit
R=Path('/mnt/sarmg.org/blog');M=json.load(open('/tmp/blog-purpose-native-metadata.json'));pn={p['name']:p for p in M['packages']};nodes={p['id']:p for p in M['resolve']['nodes']};selected=json.load(open('/tmp/blog-compiler-selected.json'));packs={n:pn[n] for n in selected}
C=R/'src/foundation/assets/compiler';assert not C.exists();C.mkdir()
mods={n:pn[n]['name'].split('-')[-1].replace('-','_') for n in selected}
# Full functional slugs avoid ambiguous `parser` and `types` roots.
for n in mods:mods[n]=Path(pn[n]['manifest_path']).parent.name.replace('-','_')
assert len(set(mods.values()))==len(mods)
manifest=tomlkit.parse('''[package]\nname = "blog-assets-compiler"\nedition = "2024"\nrust-version = "1.99"\npublish = false\ndescription = "宿主 JavaScript 与 CSS 解析、语义分析和压缩编译器"\nlicense = "MIT AND MPL-2.0 AND Apache-2.0"\nautobins = false\nautoexamples = false\nautotests = false\nautobenches = false\n[lib]\nname = "blog_assets_compiler"\npath = "src/lib.rs"\n[dependencies]\n''')
extdeps={};extalias={};per_alias={};plans=[]
# Consume only dependencies and features actually selected in Blog's host asset compiler.
for n,p in packs.items():
 doc=tomlkit.parse(Path(p['manifest_path']).read_text());node=nodes[p['id']];actual={x['name']:pn[next(q['name'] for q in M['packages'] if q['id']==x['pkg'])] for x in node['deps'] if any(k['kind'] is None for k in x['dep_kinds'])}
 aliases={};dep_entries=[]
 for s in [doc,*doc.get('target',{}).values()]:
  for a,d in s.get('dependencies',{}).items():
   a_key=a.replace('-','_');q=actual.get(a_key)
   if q is None:continue
   if q['name'] in packs:aliases[a_key]='crate::'+mods[q['name']];continue
   target=q['name']
   if target not in extalias:
    canonical=a_key
    if canonical in extalias.values():canonical=q['name'].replace('-','_')
    extalias[target]=canonical;extdeps[target]={'package':target,'path':os.path.relpath(Path(q['manifest_path']).parent,C),'features':[],'default-features':False}
   alias=extalias[target];aliases[a_key]=alias
   # Frozen tool features are a union of actual enabled edges; no optional package can escape this host boundary.
   f=extdeps[target]['features'];f.extend(str(x) for x in d.get('features',[]))
   if d.get('default-features',True):extdeps[target]['default-features']=True
 per_alias[n]=aliases
for n,dep in extdeps.items():
 dep['features']=sorted(set(dep['features']));manifest['dependencies'][extalias[n]]=dep
# Keep derive macro providers conventional; procedural expansions still use their normal extern names.
for n,p in packs.items():
 source=Path(p['manifest_path']).parent;dest=C/'parts'/mods[n];shutil.copytree(source,dest)
 for name in ['Cargo.toml','Cargo.toml.orig','ORIGIN.json']:
  (dest/name).unlink(missing_ok=True)
 flags=set(nodes[p['id']]['features']);aliases=per_alias[n];exported=set()
 for file in dest.rglob('*.rs'):
  t=file.read_text();old=source/file.relative_to(dest)
  # crate-level deployment flags have no meaning inside a host module.
  t=re.sub(r'^\s*#!\[(?:no_std|crate_name\s*=\s*"[^"]*"|recursion_limit\s*=\s*"[^"]*")\]\s*$', '',t,flags=re.M)
  t=re.sub(r'feature\s*=\s*"([^"]+)"',lambda x:'all()' if x[1] in flags else 'any()',t)
  t=re.sub(r'(?<![$\w])crate::','crate::'+mods[n]+'::',t)
  t=t.replace('$crate::','$crate::'+mods[n]+'::')
  # Absolute extern paths also route to the same real module; there are no former crate facades.
  for a,b in sorted(aliases.items(),key=lambda x:-len(x[0])):
   if b==a:continue
   t=re.sub(r'(?<![\w])(?:::)?'+re.escape(a)+r'\s*::',b+'::',t)
   t=re.sub(r'\b(pub\s+)?use '+re.escape(a)+r'\s*;',lambda x:(x[1] or '')+'use '+b+' as '+a+';',t)
   t=re.sub(r'extern crate '+re.escape(a)+r'\s*;','use '+b+' as '+a+';',t)
  # Existing include inputs are relocated by exact file identity, not by string suffix guessing.
  def include(x):
   value=x[2];target=(old.parent/value).resolve()
   if not target.exists() or not target.is_file():return x[0]
   out=None
   for other,pk in packs.items():
    prefix=Path(pk['manifest_path']).parent
    if target.is_relative_to(prefix):out=C/'parts'/mods[other]/target.relative_to(prefix);break
   if out is None:
    out=R/'docs/reference/compiler'/target.relative_to(R);out.parent.mkdir(parents=True,exist_ok=True);out.write_bytes(target.read_bytes())
   return x[1]+'"'+os.path.relpath(out,file.parent)+'"'
  t=re.sub(r'((?:include_str|include_bytes|include)!\(\s*)"([^"\n]+)"',include,t)
  t=t.replace('env!("CARGO_MANIFEST_DIR")','concat!(env!("CARGO_MANIFEST_DIR"), "/parts/'+mods[n]+'")')
  # Only exported rules need an in-module path as well as Rust's crate-root macro namespace.
  exported.update(re.findall(r'#\[macro_export\](?:(?!macro_rules!).)*macro_rules!\s+(\w+)',t,re.S))
  file.write_text(t)
 lib=tomlkit.parse(Path(p['manifest_path']).read_text()).get('lib',{}).get('path','src/lib.rs');lf=dest/lib
 if exported:lf.write_text(lf.read_text()+'\n#[doc(hidden)]\npub use crate::{'+','.join(sorted(exported))+'};\n')
 plans.append({'directory':str(source.relative_to(R)),'name':n,'module':mods[n],'selected_host_features':sorted(flags),'library':str(lf.relative_to(C))})
# Generated AST derives are compiler-private and must name their new provider modules explicitly.
provider=R/'src/foundation/javascript/syntax-derive'
providers={a:b.replace('crate::','::blog_assets_compiler::',1) for n,als in per_alias.items() for a,b in als.items() if b.startswith('crate::')}
for file in provider.rglob('*.rs'):
 t=file.read_text();before=t
 for a,b in sorted(providers.items(),key=lambda x:-len(x[0])):t=re.sub(r'(?<![\w])(?:::)?'+re.escape(a)+r'\s*::',b+'::',t)
 if t!=before:file.write_text(t)
# The selector table is a fixed HTML ASCII set, so remove build-time PHF generation in favour of a sorted constant.
selector=C/'parts'/mods['blog-assets-style-selectors'];build=(selector/'build.rs').read_text();words=re.search(r'ASCII_CASE_INSENSITIVE_HTML_ATTRIBUTES:.*?r#"(.*?)"#',build,re.S)[1].split();words=sorted(set(words))
file=selector/'parser.rs';t=file.read_text();expression='include!(concat!(env!("OUT_DIR"), "/ascii_case_insensitive_html_attributes.rs")).contains(local_name)';assert expression in t
t=t.replace(expression,'['+', '.join(json.dumps(s) for s in words)+'].binary_search(&local_name).is_ok()');file.write_text(t)
(selector/'build.rs').unlink();(C/'parts'/mods['blog-javascript-source-map']/'build.rs').unlink()
(C/'src').mkdir();(C/'src/lib.rs').write_text('''//! Host asset compiler: JavaScript/CSS AST, semantic analysis, linking and minification.\n#![recursion_limit = "512"]\nextern crate alloc;\nextern crate self as blog_assets_compiler;\n'''+''.join('#[path = "../'+p['library']+'"]\npub mod '+p['module']+';\n' for p in plans))
(C/'Cargo.toml').write_text(tomlkit.dumps(manifest))
# All external users now import one compiler, with an explicit language/phase module.
for file in [R/'tools/assets/Cargo.toml']:
 doc=tomlkit.parse(file.read_text());deps=doc['dependencies']
 for alias in ['oxc','lightningcss','parcel_selectors']:deps.pop(alias,None)
 deps['compiler']={'package':'blog-assets-compiler','path':os.path.relpath(C,file.parent)};file.write_text(tomlkit.dumps(doc))
for file in (R/'tools/assets/src').rglob('*.rs'):
 t=file.read_text();old=t
 for a,n in [('oxc','blog-javascript-toolkit'),('lightningcss','blog-assets-style-optimizer'),('parcel_selectors','blog-assets-style-selectors')]:
  t=re.sub(r'\b'+a+r'::','compiler::'+mods[n]+'::',t)
 if t!=old:file.write_text(t)
Path('/tmp/blog-compiler-integration.json').write_text(json.dumps({'package':{'directory':str(C.relative_to(R)),'name':'blog-assets-compiler','version':'0.0.0','layer':'assets','purpose':'宿主 JS/CSS AST、语义分析、资源链接与压缩'},'inputs':plans,'external_aliases':extalias},ensure_ascii=False,indent=2)+'\n')
print('Merged',len(plans),'actual source components into one host compiler; all original source trees await lineage reconciliation.')
