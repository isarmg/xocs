//! Read module edges and asset locations from the compiler AST, preserving source ranges.
use compiler::toolkit::{
    allocator::Allocator,
    ast::ast::*,
    ast_visit::{Visit, walk},
    codegen::{Codegen, CodegenOptions},
    diagnostics::{Diagnostics, OxcDiagnostic},
    minifier::{Minifier, MinifierOptions},
    parser::Parser,
    semantic::{Semantic, SemanticBuilder},
    span::{SourceType, Span},
};

type Result<T> = std::result::Result<T, JavaScriptError>;

#[derive(Clone, Copy, Debug)]
pub enum JavaScriptStage {
    Source,
    Semantic,
    Linked,
}

impl std::fmt::Display for JavaScriptStage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Source => "source syntax",
            Self::Semantic => "source semantics",
            Self::Linked => "linked syntax",
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnsupportedModuleFeature {
    #[error("import attributes and phases are unsupported")]
    ImportAttributes,
    #[error("export attributes are unsupported")]
    ExportAttributes,
    #[error("dynamic import attributes are unsupported")]
    DynamicImportAttributes,
    #[error("computed dynamic imports require an explicit vendored ESM import")]
    ComputedDynamicImport,
    #[error("commonjs require() needs a vendored ESM distribution")]
    CommonJs,
}

#[derive(Debug, thiserror::Error)]
pub enum JavaScriptError {
    #[error("invalid javascript {stage}")]
    Diagnostics {
        stage: JavaScriptStage,
        #[source]
        primary: OxcDiagnostic,
        additional: Vec<OxcDiagnostic>,
    },
    #[error("unsupported javascript module feature")]
    Unsupported {
        #[source]
        primary: UnsupportedModuleFeature,
        additional: Vec<UnsupportedModuleFeature>,
    },
}

impl JavaScriptError {
    fn diagnostics(stage: JavaScriptStage, diagnostics: Diagnostics) -> Self {
        let mut diagnostics = diagnostics.into_iter();
        Self::Diagnostics {
            stage,
            primary: diagnostics
                .next()
                .expect("invariant: an invalid program has a diagnostic"),
            additional: diagnostics.collect(),
        }
    }
}

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
}

struct ReferenceCollector<'s, 'a> {
    semantic: &'s Semantic<'a>,
    result: References,
    errors: Vec<UnsupportedModuleFeature>,
}

// The minifier may print a string as a no-substitution template literal.
// Both forms have a statically known cooked value and a complete AST span.
fn static_string(expression: &Expression<'_>) -> Option<(String, Span)> {
    match expression {
        Expression::StringLiteral(literal) => Some((literal.value.to_string(), literal.span)),
        Expression::TemplateLiteral(literal) => literal
            .single_quasi()
            .map(|value| (value.to_string(), literal.span)),
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::Identifier(_)
        | Expression::Super(_)
        | Expression::ArrayExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::BinaryExpression(_)
        | Expression::CallExpression(_)
        | Expression::ChainExpression(_)
        | Expression::ClassExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ImportExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::NewExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::ParenthesizedExpression(_)
        | Expression::SequenceExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::ThisExpression(_)
        | Expression::UnaryExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::YieldExpression(_)
        | Expression::PrivateInExpression(_)
        | Expression::ImportMeta(_)
        | Expression::NewTarget(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::TSAsExpression(_)
        | Expression::TSSatisfiesExpression(_)
        | Expression::TSTypeAssertion(_)
        | Expression::TSNonNullExpression(_)
        | Expression::TSInstantiationExpression(_)
        | Expression::V8IntrinsicExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => None,
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
            self.errors.push(UnsupportedModuleFeature::ImportAttributes);
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
            self.errors.push(UnsupportedModuleFeature::ExportAttributes);
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
            self.errors.push(UnsupportedModuleFeature::ExportAttributes);
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
                self.errors
                    .push(UnsupportedModuleFeature::DynamicImportAttributes);
            }
        } else {
            self.errors
                .push(UnsupportedModuleFeature::ComputedDynamicImport);
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
            self.errors.push(UnsupportedModuleFeature::CommonJs);
        }
        walk::walk_call_expression(self, node);
    }
}

pub fn references(text: &str) -> Result<References> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::mjs()).parse();
    if !parsed.diagnostics.is_empty() {
        return Err(JavaScriptError::diagnostics(
            JavaScriptStage::Source,
            parsed.diagnostics,
        ));
    }
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parsed.program);
    if !semantic.diagnostics.is_empty() {
        return Err(JavaScriptError::diagnostics(
            JavaScriptStage::Semantic,
            semantic.diagnostics,
        ));
    }
    let mut collector = ReferenceCollector {
        semantic: &semantic.semantic,
        result: References::default(),
        errors: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    let references = collector.result;
    let mut errors = collector.errors.into_iter();
    if let Some(primary) = errors.next() {
        return Err(JavaScriptError::Unsupported {
            primary,
            additional: errors.collect(),
        });
    }
    Ok(references)
}

pub fn minify(text: &str) -> Result<Vec<u8>> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::mjs()).parse();
    if !parsed.diagnostics.is_empty() {
        return Err(JavaScriptError::diagnostics(
            JavaScriptStage::Linked,
            parsed.diagnostics,
        ));
    }
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
#[path = "../../../../test/tools/assets/unit/javascript.rs"]
mod tests;
