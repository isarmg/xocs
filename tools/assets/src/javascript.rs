//! All source ranges originate in Oxc's AST, never a JavaScript text regex.
use anyhow::{Result, bail, ensure};
use compiler::toolkit::{
    allocator::Allocator,
    ast::ast::*,
    ast_visit::{Visit, walk},
    codegen::{Codegen, CodegenOptions},
    minifier::{Minifier, MinifierOptions},
    parser::Parser,
    semantic::{Semantic, SemanticBuilder},
    span::{SourceType, Span},
};

#[derive(Debug)]
pub struct Import {
    pub url: String,
    pub source: Span,
    pub statement: Span,
    pub default_asset: bool,
    pub has_bindings: bool,
}

#[derive(Debug, Default)]
pub struct References {
    pub imports: Vec<Import>,
    pub assets: Vec<(String, Span)>,
    pub dynamic: Vec<(String, Span)>,
    errors: Vec<String>,
}

struct ReferenceCollector<'s, 'a> {
    semantic: &'s Semantic<'a>,
    result: References,
}

// The minifier may print a string as a no-substitution template literal.
// Both forms have a statically known cooked value and a complete AST span.
fn static_string(expression: &Expression<'_>) -> Option<(String, Span)> {
    match expression {
        Expression::StringLiteral(literal) => Some((literal.value.to_string(), literal.span)),
        Expression::TemplateLiteral(literal) => literal
            .single_quasi()
            .map(|value| (value.to_string(), literal.span)),
        _ => None,
    }
}

impl<'a> Visit<'a> for ReferenceCollector<'_, 'a> {
    fn visit_import_declaration(&mut self, node: &ImportDeclaration<'a>) {
        let default_asset = node.specifiers.as_ref().is_some_and(|items| {
            items.len() == 1
                && matches!(
                    items[0],
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                )
        });
        if node.with_clause.is_some() || node.phase.is_some() {
            self.result
                .errors
                .push("Import attributes/phases are unsupported".into());
        }
        self.result.imports.push(Import {
            url: node.source.value.to_string(),
            source: node.source.span,
            statement: node.span,
            default_asset,
            has_bindings: node
                .specifiers
                .as_ref()
                .is_some_and(|items| !items.is_empty()),
        });
    }
    fn visit_export_from_declaration(&mut self, node: &ExportFromDeclaration<'a>) {
        let default_asset =
            node.specifiers.len() == 1 && node.specifiers[0].local.name().as_str() == "default";
        if node.with_clause.is_some() {
            self.result
                .errors
                .push("Export attributes are unsupported".into());
        }
        self.result.imports.push(Import {
            url: node.source.value.to_string(),
            source: node.source.span,
            statement: node.span,
            default_asset,
            has_bindings: true,
        });
    }
    fn visit_export_all_declaration(&mut self, node: &ExportAllDeclaration<'a>) {
        if node.with_clause.is_some() {
            self.result
                .errors
                .push("Export attributes are unsupported".into());
        }
        self.result.imports.push(Import {
            url: node.source.value.to_string(),
            source: node.source.span,
            statement: node.span,
            default_asset: false,
            has_bindings: true,
        });
    }
    fn visit_import_expression(&mut self, node: &ImportExpression<'a>) {
        if let Some(reference) = static_string(&node.source) {
            if node.options.is_none() && node.phase.is_none() {
                self.result.dynamic.push(reference);
            } else {
                self.result
                    .errors
                    .push("Dynamic import attributes are unsupported".into());
            }
        } else {
            self.result.errors.push(
                "Computed dynamic imports are unsupported; use an explicit vendored ESM import"
                    .into(),
            );
        }
        walk::walk_import_expression(self, node);
    }
    fn visit_new_expression(&mut self, node: &NewExpression<'a>) {
        if matches!(&node.callee, Expression::Identifier(name) if name.name == "URL"
                && self.semantic.is_reference_to_global_variable(name))
            && node.arguments.len() == 2
            && let Some(reference) = node.arguments[0].as_expression().and_then(static_string)
            && let Argument::StaticMemberExpression(member) = &node.arguments[1]
            && member.property.name == "url"
            && matches!(&member.object, Expression::ImportMeta(_))
        {
            self.result.assets.push(reference);
        }
        walk::walk_new_expression(self, node);
    }
    fn visit_call_expression(&mut self, node: &CallExpression<'a>) {
        if matches!(&node.callee, Expression::Identifier(name) if name.name == "require"
            && self.semantic.is_reference_to_global_variable(name))
        {
            self.result.errors.push(
                "CommonJS require() is unsupported; vendor the upstream ESM distribution".into(),
            );
        }
        walk::walk_call_expression(self, node);
    }
}

pub fn references(text: &str) -> Result<References> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::mjs()).parse();
    ensure!(
        parsed.diagnostics.is_empty(),
        "JavaScript syntax error: {:?}",
        parsed.diagnostics
    );
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parsed.program);
    ensure!(
        semantic.diagnostics.is_empty(),
        "JavaScript semantic error: {:?}",
        semantic.diagnostics
    );
    let mut collector = ReferenceCollector {
        semantic: &semantic.semantic,
        result: References::default(),
    };
    collector.visit_program(&parsed.program);
    let references = collector.result;
    if !references.errors.is_empty() {
        bail!("{}", references.errors.join("; "));
    }
    Ok(references)
}

pub fn minify(text: &str) -> Result<Vec<u8>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::mjs()).parse();
    ensure!(
        parsed.diagnostics.is_empty(),
        "Linked JavaScript syntax error: {:?}",
        parsed.diagnostics
    );
    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions::default()).minify(&allocator, &mut program);
    Ok(Codegen::new()
        .with_options(CodegenOptions::minify())
        .with_scoping(result.scoping)
        .build(&program)
        .code
        .into_bytes())
}

pub fn range(span: Span) -> (usize, usize) {
    (span.start as usize, span.end as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strings_comments_regex_are_not_module_edges() -> Result<()> {
        let text = r#"// import x from 'fake';
const s="import x from 'fake'"; const r=/import\("fake"\)/;
import {x} from "./real.js"; export { x as y } from './other.js';
const asset=new URL('./a.wasm',import.meta.url);"#;
        let result = references(text)?;
        assert_eq!(result.imports.len(), 2);
        assert_eq!(result.assets.len(), 1);
        assert_eq!(result.imports[0].url, "./real.js");
        assert!(references("import(window.name)").is_err());
        Ok(())
    }
    #[test]
    fn syntax_semantics_and_module_loading_contract_remain_strict() -> Result<()> {
        for source in [
            "export const = ;",
            "const x = 1; const x = 2;",
            "import(window.name)",
            "import('./a.js', {with: {type: 'json'}})",
            "const item = require('./a.js');",
        ] {
            assert!(
                references(source).is_err(),
                "Accepted invalid module: {source}"
            );
        }
        let source = "export { x } from './static.js'; const item = import('./dynamic.js');";
        let refs = references(source)?;
        assert_eq!(refs.imports.len(), 1);
        assert_eq!(refs.imports[0].url, "./static.js");
        assert_eq!(refs.dynamic.len(), 1);
        assert_eq!(refs.dynamic[0].0, "./dynamic.js");
        let (start, end) = range(refs.imports[0].source);
        assert_eq!(&source[start..end], "'./static.js'");
        Ok(())
    }
    #[test]
    fn global_asset_references_respect_lexical_bindings() -> Result<()> {
        let source = r#"
            export function local(URL, require) {
                require('./not-a-module');
                return new URL('./not-an-asset', import.meta.url);
            }
            export function hoisted() {
                const value = new URL('./also-not-an-asset', import.meta.url);
                function URL() {}
                return value;
            }
            export const real = new URL('./real.wasm', import.meta.url);
        "#;
        let refs = references(source)?;
        assert_eq!(refs.assets.len(), 1);
        assert_eq!(refs.assets[0].0, "./real.wasm");
        assert!(references("require('./unsupported.js')").is_err());
        Ok(())
    }

    #[test]
    fn static_template_urls_use_cooked_values_and_whole_literal_spans() -> Result<()> {
        let source = r"export const asset = new URL(`./a\u002ewasm`, import.meta.url); import(`./module.js`);";
        let refs = references(source)?;
        assert_eq!(refs.assets[0].0, "./a.wasm");
        let (start, end) = range(refs.assets[0].1);
        assert_eq!(&source[start..end], r"`./a\u002ewasm`");
        assert_eq!(refs.dynamic[0].0, "./module.js");
        assert!(references("import(`./${name}.js`)").is_err());
        Ok(())
    }

    #[test]
    fn export_attributes_cannot_bypass_the_module_contract() {
        for source in [
            "export { default } from './data.json' with { type: 'json' };",
            "export * from './data.json' with { type: 'json' };",
        ] {
            assert!(references(source).is_err(), "Accepted {source}");
        }
    }

    #[test]
    fn compression_preserves_string_and_regular_expression_literals() -> Result<()> {
        let code = minify("export const s = 'a  b'; export const r = /a  b/; // comment\n")?;
        let result = String::from_utf8(code)?;
        assert!(result.contains("a  b"));
        assert!(!result.contains("// comment"));
        references(&result)?;
        Ok(())
    }
}
