from pathlib import Path
import re,sys
root=Path(sys.argv[1]);compiler=root/'src/foundation/assets/compiler';style=root/'src/foundation/assets/style-compiler'
css={'style_color','style_optimizer','style_parser','style_selectors','cached_hash','float_format','short_float_format','relative_path'}
css_deps={'ahash','bitflags','const_str','data_encoding','indexmap','itertools','lazy_static','lightningcss_derive','pastey','smallvec','cssparser_macros','itoa','phf','log','rustc_hash','serde','serde_json'}
js_drop=css_deps-{'bitflags','smallvec','itoa','log','rustc_hash','serde','serde_json'}|{'getrandom'}
(style/'parts').mkdir(parents=True);(style/'src').mkdir()
for name in css:(compiler/'parts'/name).rename(style/'parts'/name)
lib=(compiler/'src/lib.rs').read_text();header,modules=lib.split('#[path',1);modules='#[path'+modules
parts=re.findall(r'#\[path = "../parts/([^/]+)/[^\"]+"\]\n(?:#\[macro_use\]\n)?pub mod [a-z_]+;\n?',modules)
assert len(parts)==35,len(parts)
blocks=re.findall(r'#\[path = "../parts/([^/]+)/[^\"]+"\]\n(?:#\[macro_use\]\n)?pub mod [a-z_]+;\n?',modules)
module_re=re.compile(r'#\[path = "../parts/([^/]+)/[^\"]+"\]\n(?:#\[macro_use\]\n)?pub mod [a-z_]+;\n?')
css_blocks=[];js_blocks=[]
for m in module_re.finditer(modules):(css_blocks if m[1] in css else js_blocks).append(m[0])
(compiler/'src/lib.rs').write_text(header.replace('JavaScript/CSS AST, semantic analysis, linking and minification.','JavaScript AST, semantic analysis and minification.')+''.join(js_blocks))
(style/'src/lib.rs').write_text(header.replace('Host asset compiler: JavaScript/CSS AST, semantic analysis, linking and minification.','Host stylesheet compiler: CSS parsing, optimization and serialization.').replace('blog_assets_compiler','blog_assets_style_compiler')+''.join(css_blocks))
manifest=(compiler/'Cargo.toml').read_text();header=manifest.split('[dependencies]',1)[0]
blocks={m[1]:m[0] for m in re.finditer(r'\[dependencies\.([^\]]+)\]\n.*?(?=\n\[|\Z)',manifest,re.S)}
assert len(blocks)>30
(compiler/'Cargo.toml').write_text(header.replace('宿主 JavaScript 与 CSS 解析、语义分析和压缩编译器','宿主 JavaScript 解析、语义分析和压缩编译器')+'[dependencies]\n'+ '\n'.join(v for k,v in blocks.items() if k not in js_drop))
(style/'Cargo.toml').write_text(header.replace('blog-assets-compiler','blog-assets-style-compiler').replace('blog_assets_compiler','blog_assets_style_compiler').replace('宿主 JavaScript 与 CSS 解析、语义分析和压缩编译器','宿主 CSS 解析、层叠样式优化和序列化')+'[dependencies]\n'+ '\n'.join(v for k,v in blocks.items() if k in css_deps))
for p in (root/'src/foundation/assets/style-value-derive/src').rglob('*.rs'):
 s=p.read_text()
 if 'blog_assets_compiler' in s:p.write_text(s.replace('blog_assets_compiler','blog_assets_style_compiler'))
p=root/'tools/assets/src/assets.rs';p.write_text(p.read_text().replace('compiler::style_','style_compiler::style_'))
p=root/'tools/assets/Cargo.toml';p.write_text(p.read_text()+'\n[dependencies.style-compiler]\npackage = "blog-assets-style-compiler"\npath = "../../src/foundation/assets/style-compiler"\n')
# Benchmark snapshot only: final root workspace and provenance are owned by root.
if "--workspace" in sys.argv:
 p=root/'Cargo.toml';p.write_text(p.read_text().replace('exclude = [','exclude = ["src/foundation/assets/style-compiler", ',1))
