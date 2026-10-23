
    use crate::syntax_arena::Allocator;
    use crate::source_location::{SourceType, Span};
    use crate::syntax_types::module_record::*;

    use crate::parser::Parser;

    fn build<'a>(allocator: &'a Allocator, source_text: &'a str) -> ModuleRecord<'a> {
        let source_type = SourceType::ts();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        assert!(ret.diagnostics.is_empty());
        ret.module_record
    }

    // Table 55 gives examples of ImportEntry records fields used to represent the syntactic import forms:
    // `https://tc39.es/ecma262/#table-import-forms-mapping-to-importentry-records`

    #[test]
    fn import_default() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import v from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 19),
            module_request: NameSpan::new("mod".into(), Span::new(14, 19)),
            import_name: ImportImportName::Default(Span::new(7, 8)),
            local_name: NameSpan::new("v".into(), Span::new(7, 8)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_namespace() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import * as ns from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 25),
            module_request: NameSpan::new("mod".into(), Span::new(20, 25)),
            import_name: ImportImportName::NamespaceObject,
            local_name: NameSpan::new("ns".into(), Span::new(12, 14)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_specifier() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import { x } from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 23),
            module_request: NameSpan::new("mod".into(), Span::new(18, 23)),
            import_name: ImportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            local_name: NameSpan::new("x".into(), Span::new(9, 10)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_specifier_alias() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import { x as v } from 'mod'");
        let import_entry = ImportEntry {
            statement_span: Span::new(0, 28),
            module_request: NameSpan::new("mod".into(), Span::new(23, 28)),
            import_name: ImportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            local_name: NameSpan::new("v".into(), Span::new(14, 15)),
            is_type: false,
        };
        assert_eq!(module_record.import_entries.len(), 1);
        assert_eq!(module_record.import_entries[0], import_entry);
    }

    #[test]
    fn import_without_binding() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import 'mod'");
        assert!(module_record.import_entries.is_empty());
    }

    // Table 57 gives examples of the ExportEntry record fields used to represent the syntactic export forms
    // `https://tc39.es/ecma262/#table-export-forms-mapping-to-exportentry-records`

    #[test]
    fn export_star() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportFromClause : *
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export * from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 19),
            module_request: Some(NameSpan::new("mod".into(), Span::new(14, 19))),
            import_name: ExportImportName::AllButDefault,
            span: Span::new(0, 19),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.star_export_entries.len(), 1);
        assert_eq!(module_record.star_export_entries[0], export_entry);
    }

    #[test]
    fn export_star_as_namespace() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportFromClause : * as ModuleExportName
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export * as ns from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 25),
            module_request: Some(NameSpan::new("mod".into(), Span::new(20, 25))),
            import_name: ExportImportName::All,
            export_name: ExportExportName::Name(NameSpan::new("ns".into(), Span::new(12, 14))),
            span: Span::new(0, 25),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports() {
        // ExportDeclaration : export NamedExports ;
        // ExportSpecifier : ModuleExportName
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x }; var x");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 13),
            span: Span::new(9, 10),
            export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            local_name: ExportLocalName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_alias() {
        // ExportDeclaration : export NamedExports ;
        // ExportSpecifier : ModuleExportName as ModuleExportName
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x as v }; var x");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 18),
            span: Span::new(9, 15),
            export_name: ExportExportName::Name(NameSpan::new("v".into(), Span::new(14, 15))),
            local_name: ExportLocalName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_from() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportSpecifier : ModuleExportName
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x } from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 23),
            module_request: Some(NameSpan::new("mod".into(), Span::new(18, 23))),
            export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            import_name: ExportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            span: Span::new(9, 10),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn named_exports_alias_from() {
        // ExportDeclaration : export ExportFromClause FromClause ;
        // ExportSpecifier : ModuleExportName as ModuleExportName
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { x as v } from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 28),
            module_request: Some(NameSpan::new("mod".into(), Span::new(23, 28))),
            export_name: ExportExportName::Name(NameSpan::new("v".into(), Span::new(14, 15))),
            import_name: ExportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
            span: Span::new(9, 15),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn export_declaration() {
        // ExportDeclaration : export VariableStatement
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export var v");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 12),
            export_name: ExportExportName::Name(NameSpan::new("v".into(), Span::new(11, 12))),
            local_name: ExportLocalName::Name(NameSpan::new("v".into(), Span::new(11, 12))),
            span: Span::new(7, 12),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_declaration() {
        // ExportDeclaration : export default HoistableDeclaration
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default function f() {}");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 30),
            span: Span::new(15, 30),
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Name(NameSpan::new("f".into(), Span::new(24, 25))),
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_function_expression() {
        // ExportDeclaration : export default HoistableDeclaration
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default function() {}");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 28),
            span: Span::new(15, 28),
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Null,
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_default_expression() {
        // ExportDeclaration : export default HoistableDeclaration
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default 42");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 17),
            span: Span::new(15, 17),
            export_name: ExportExportName::Default(Span::new(7, 14)),
            local_name: ExportLocalName::Null,
            ..ExportEntry::default()
        };
        assert_eq!(module_record.local_export_entries.len(), 1);
        assert_eq!(module_record.local_export_entries[0], export_entry);
    }

    #[test]
    fn export_named_default() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export { default } from 'mod'");
        let export_entry = ExportEntry {
            statement_span: Span::new(0, 29),
            module_request: Some(NameSpan::new("mod".into(), Span::new(24, 29))),
            span: Span::new(9, 16),
            export_name: ExportExportName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            import_name: ExportImportName::Name(NameSpan::new("default".into(), Span::new(9, 16))),
            local_name: ExportLocalName::Null,
            ..ExportEntry::default()
        };
        assert_eq!(module_record.indirect_export_entries.len(), 1);
        assert_eq!(module_record.indirect_export_entries[0], export_entry);
    }

    #[test]
    fn indirect_export_entries() {
        let allocator = Allocator::default();
        let module_record =
            build(&allocator, "import { x } from 'mod';export { x };export * as ns from 'mod';");
        assert_eq!(module_record.indirect_export_entries.len(), 2);
        assert_eq!(
            module_record.indirect_export_entries[0],
            ExportEntry {
                statement_span: Span::new(0, 24),
                span: Span::new(33, 34),
                module_request: Some(NameSpan::new("mod".into(), Span::new(18, 23))),
                import_name: ExportImportName::Name(NameSpan::new("x".into(), Span::new(9, 10))),
                export_name: ExportExportName::Name(NameSpan::new("x".into(), Span::new(33, 34))),
                local_name: ExportLocalName::Null,
                is_type: false
            }
        );
        assert_eq!(
            module_record.indirect_export_entries[1],
            ExportEntry {
                statement_span: Span::new(37, 63),
                span: Span::new(37, 63),
                module_request: Some(NameSpan::new("mod".into(), Span::new(57, 62))),
                import_name: ExportImportName::All,
                export_name: ExportExportName::Name(NameSpan::new("ns".into(), Span::new(49, 51))),
                local_name: ExportLocalName::Null,
                is_type: false
            }
        );
    }

    #[test]
    fn import_meta() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import.meta.foo; import.meta.bar");
        assert_eq!(module_record.import_metas.len(), 2);
        assert_eq!(module_record.import_metas[0], Span::new(0, 11));
        assert_eq!(module_record.import_metas[1], Span::new(17, 28));
    }

    #[test]
    fn dynamic_imports() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "import('foo')");
        assert_eq!(module_record.dynamic_imports.len(), 1);
        assert_eq!(module_record.dynamic_imports[0].span, Span::new(0, 13));
        assert_eq!(module_record.dynamic_imports[0].module_request, Span::new(7, 12));
    }

    #[test]
    fn export_default_interface() {
        let allocator = Allocator::default();
        let module_record = build(&allocator, "export default interface Foo {}");
        assert_eq!(module_record.local_export_entries.len(), 1);
        let entry = &module_record.local_export_entries[0];
        assert_eq!(entry.local_name.name().unwrap().as_str(), "Foo");
        assert!(entry.export_name.is_default());
        assert!(entry.is_type);
    }
