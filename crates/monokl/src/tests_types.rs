use crate::types::{SymbolEntry, SymbolKind, Visibility};

#[test]
fn kind_camel_case_forms() {
    let pairs: [(SymbolKind, &str); 15] = [
        (SymbolKind::Function, "\"function\""),
        (SymbolKind::Method, "\"method\""),
        (SymbolKind::Constructor, "\"constructor\""),
        (SymbolKind::Class, "\"class\""),
        (SymbolKind::Struct, "\"struct\""),
        (SymbolKind::Enum, "\"enum\""),
        (SymbolKind::Interface, "\"interface\""),
        (SymbolKind::TypeAlias, "\"typeAlias\""),
        (SymbolKind::Property, "\"property\""),
        (SymbolKind::Field, "\"field\""),
        (SymbolKind::Variable, "\"variable\""),
        (SymbolKind::Module, "\"module\""),
        (SymbolKind::Impl, "\"impl\""),
        (SymbolKind::Macro, "\"macro\""),
        (SymbolKind::Other, "\"other\""),
    ];
    for (variant, expected) in pairs {
        let s = serde_json::to_string(&variant).unwrap();
        assert_eq!(s, expected);
        let round: SymbolKind = serde_json::from_str(expected).unwrap();
        assert_eq!(round, variant);
    }
}

#[test]
fn visibility_camel_case_forms() {
    let pairs: [(Visibility, &str); 4] = [
        (Visibility::Public, "\"public\""),
        (Visibility::Crate, "\"crate\""),
        (Visibility::Module, "\"module\""),
        (Visibility::Private, "\"private\""),
    ];
    for (variant, expected) in pairs {
        let s = serde_json::to_string(&variant).unwrap();
        assert_eq!(s, expected);
        let round: Visibility = serde_json::from_str(expected).unwrap();
        assert_eq!(round, variant);
    }
}

#[test]
fn kind_unknown_variant_errors() {
    let err = serde_json::from_str::<SymbolKind>("\"bogus\"").unwrap_err();
    assert!(err.to_string().contains("unknown variant"));
}

#[test]
fn visibility_unknown_variant_errors() {
    let err = serde_json::from_str::<Visibility>("\"super\"").unwrap_err();
    assert!(err.to_string().contains("unknown variant"));
}

#[test]
fn signature_null_key_present_others_omitted() {
    let e = SymbolEntry {
        name: "foo".into(),
        kind: SymbolKind::Function,
        line: 1,
        signature: None,
        owner: None,
        trait_impl: None,
        visibility: None,
        kind_detail: None,
    };
    let v = serde_json::to_value(&e).unwrap();
    let obj = v.as_object().unwrap();
    assert!(obj.contains_key("signature"));
    assert_eq!(obj.get("signature"), Some(&serde_json::Value::Null));
    assert!(!obj.contains_key("owner"));
    assert!(!obj.contains_key("traitImpl"));
    assert!(!obj.contains_key("visibility"));
    assert!(!obj.contains_key("kindDetail"));
}

#[test]
fn missing_optional_keys_deserialize_to_none() {
    let json = r#"{"name":"foo","kind":"function","line":1}"#;
    let e: SymbolEntry = serde_json::from_str(json).unwrap();
    assert_eq!(e.signature, None);
    assert_eq!(e.owner, None);
    assert_eq!(e.trait_impl, None);
    assert_eq!(e.visibility, None);
    assert_eq!(e.kind_detail, None);
}

#[test]
fn trait_impl_kind_detail_camel_case_keys() {
    let e = SymbolEntry {
        name: "foo".into(),
        kind: SymbolKind::Function,
        line: 1,
        signature: None,
        owner: None,
        trait_impl: Some("Bar".into()),
        visibility: None,
        kind_detail: Some("x".into()),
    };
    let s = serde_json::to_string(&e).unwrap();
    assert!(s.contains("\"traitImpl\""));
    assert!(s.contains("\"kindDetail\""));
    assert!(!s.contains("\"trait_impl\""));
    assert!(!s.contains("\"kind_detail\""));
}

#[test]
fn extra_unknown_key_silently_discarded() {
    let json = r#"{"name":"foo","kind":"function","line":1,"owner":"RealOwner","impl_owner":"FakeOwner","bogusKey":123}"#;
    let e: SymbolEntry = serde_json::from_str(json).unwrap();
    assert_eq!(e.name, "foo");
    assert_eq!(e.owner.as_deref(), Some("RealOwner"));
}

#[test]
fn missing_required_field_errors() {
    let err_name = serde_json::from_str::<SymbolEntry>(r#"{"kind":"function","line":1}"#).unwrap_err();
    assert!(err_name.to_string().contains("missing field `name`"));
    let err_kind = serde_json::from_str::<SymbolEntry>(r#"{"name":"foo","line":1}"#).unwrap_err();
    assert!(err_kind.to_string().contains("missing field `kind`"));
    let err_line = serde_json::from_str::<SymbolEntry>(r#"{"name":"foo","kind":"function"}"#).unwrap_err();
    assert!(err_line.to_string().contains("missing field `line`"));
}

#[test]
fn explicit_null_optional_fields_to_none() {
    let json = r#"{"name":"foo","kind":"function","line":1,"signature":null,"owner":null,"traitImpl":null,"visibility":null,"kindDetail":null}"#;
    let e: SymbolEntry = serde_json::from_str(json).unwrap();
    assert_eq!(e.signature, None);
    assert_eq!(e.owner, None);
    assert_eq!(e.trait_impl, None);
    assert_eq!(e.visibility, None);
    assert_eq!(e.kind_detail, None);
}

#[test]
fn line_negative_errors() {
    let json = r#"{"name":"foo","kind":"function","line":-1}"#;
    let err = serde_json::from_str::<SymbolEntry>(json).unwrap_err();
    assert!(err.to_string().contains("invalid value"));
}

#[test]
fn line_non_integral_errors() {
    let json = r#"{"name":"foo","kind":"function","line":1.5}"#;
    let err = serde_json::from_str::<SymbolEntry>(json).unwrap_err();
    assert!(err.to_string().contains("invalid type"));
}

#[test]
fn line_zero_valid() {
    let json = r#"{"name":"foo","kind":"function","line":0}"#;
    let e: SymbolEntry = serde_json::from_str(json).unwrap();
    assert_eq!(e.line, 0);
}

#[test]
fn symbol_entry_full_round_trip_exact_shape() {
    let json = r#"{"name":"foo","kind":"typeAlias","line":7,"signature":"sig","owner":"Own","traitImpl":"T","visibility":"public","kindDetail":"rust-impl"}"#;
    let e: SymbolEntry = serde_json::from_str(json).unwrap();
    assert_eq!(e.name, "foo");
    assert_eq!(e.kind, SymbolKind::TypeAlias);
    assert_eq!(e.line, 7);
    assert_eq!(e.signature.as_deref(), Some("sig"));
    assert_eq!(e.owner.as_deref(), Some("Own"));
    assert_eq!(e.trait_impl.as_deref(), Some("T"));
    assert_eq!(e.visibility, Some(Visibility::Public));
    assert_eq!(e.kind_detail.as_deref(), Some("rust-impl"));

    let reserialized = serde_json::to_string(&e).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn kind_variant_count_and_order_pinned() {
    let assert_shape = |k: SymbolKind| match k {
        SymbolKind::Function
        | SymbolKind::Method
        | SymbolKind::Constructor
        | SymbolKind::Class
        | SymbolKind::Struct
        | SymbolKind::Enum
        | SymbolKind::Interface
        | SymbolKind::TypeAlias
        | SymbolKind::Property
        | SymbolKind::Field
        | SymbolKind::Variable
        | SymbolKind::Module
        | SymbolKind::Impl
        | SymbolKind::Macro
        | SymbolKind::Other => {}
    };
    assert_shape(SymbolKind::Function);

    assert_eq!(SymbolKind::Function as usize, 0);
    assert_eq!(SymbolKind::Method as usize, 1);
    assert_eq!(SymbolKind::Constructor as usize, 2);
    assert_eq!(SymbolKind::Class as usize, 3);
    assert_eq!(SymbolKind::Struct as usize, 4);
    assert_eq!(SymbolKind::Enum as usize, 5);
    assert_eq!(SymbolKind::Interface as usize, 6);
    assert_eq!(SymbolKind::TypeAlias as usize, 7);
    assert_eq!(SymbolKind::Property as usize, 8);
    assert_eq!(SymbolKind::Field as usize, 9);
    assert_eq!(SymbolKind::Variable as usize, 10);
    assert_eq!(SymbolKind::Module as usize, 11);
    assert_eq!(SymbolKind::Impl as usize, 12);
    assert_eq!(SymbolKind::Macro as usize, 13);
    assert_eq!(SymbolKind::Other as usize, 14);
}

#[test]
fn visibility_variant_count_and_order_pinned() {
    let assert_shape = |v: Visibility| match v {
        Visibility::Public | Visibility::Crate | Visibility::Module | Visibility::Private => {}
    };
    assert_shape(Visibility::Public);

    assert_eq!(Visibility::Public as usize, 0);
    assert_eq!(Visibility::Crate as usize, 1);
    assert_eq!(Visibility::Module as usize, 2);
    assert_eq!(Visibility::Private as usize, 3);
}

#[test]
fn field_types_and_derives_pinned() {
    fn assert_enum_derives<T: std::fmt::Debug + Clone + Copy + PartialEq + Eq>() {}
    fn assert_entry_derives<T: std::fmt::Debug + Clone>() {}
    assert_enum_derives::<SymbolKind>();
    assert_enum_derives::<Visibility>();
    assert_entry_derives::<SymbolEntry>();

    let json = r#"{"name":"foo","kind":"function","line":7}"#;
    let e: SymbolEntry = serde_json::from_str(json).unwrap();
    let line: usize = e.line;
    assert_eq!(line, 7);
}

use crate::types::BindingKind;

#[test]
fn binding_kind_camel_case_forms() {
    let pairs: [(BindingKind, &str); 5] = [
        (BindingKind::Named, "\"named\""),
        (BindingKind::Default, "\"default\""),
        (BindingKind::Namespace, "\"namespace\""),
        (BindingKind::Glob, "\"glob\""),
        (BindingKind::NamespaceWide, "\"namespaceWide\""),
    ];
    for (variant, expected) in pairs {
        let s = serde_json::to_string(&variant).unwrap();
        assert_eq!(s, expected);
        let round: BindingKind = serde_json::from_str(expected).unwrap();
        assert_eq!(round, variant);
    }
}

use crate::types::RustPathAnchor;

#[test]
fn rust_path_anchor_camel_case_forms() {
    let crate_json = serde_json::to_string(&RustPathAnchor::Crate).unwrap();
    assert_eq!(crate_json, "\"crate\"");
    let round: RustPathAnchor = serde_json::from_str(&crate_json).unwrap();
    assert!(matches!(round, RustPathAnchor::Crate));

    let super_json = serde_json::to_string(&RustPathAnchor::Super).unwrap();
    assert_eq!(super_json, "\"super\"");
    let round: RustPathAnchor = serde_json::from_str(&super_json).unwrap();
    assert!(matches!(round, RustPathAnchor::Super));

    let self_json = serde_json::to_string(&RustPathAnchor::Selff).unwrap();
    assert_eq!(self_json, "\"self\"");
    let round: RustPathAnchor = serde_json::from_str(&self_json).unwrap();
    assert!(matches!(round, RustPathAnchor::Selff));

    let extern_json = serde_json::to_string(&RustPathAnchor::Extern("foo".into())).unwrap();
    assert_eq!(extern_json, "{\"extern\":\"foo\"}");
    let round: RustPathAnchor = serde_json::from_str(&extern_json).unwrap();
    assert!(matches!(round, RustPathAnchor::Extern(ref s) if s == "foo"));
}

use crate::types::DependencyTarget;

#[test]
fn dependency_target_internal_tag_snake_case_fields() {
    let file = DependencyTarget::File {
        specifier: "./foo".into(),
        resolved: None,
        is_relative: true,
    };
    let v = serde_json::to_value(&file).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("kind"), Some(&serde_json::Value::String("file".into())));
    assert!(obj.contains_key("is_relative"));
    assert!(!obj.contains_key("isRelative"));
    assert_eq!(obj.get("specifier"), Some(&serde_json::Value::String("./foo".into())));

    let rust_path = DependencyTarget::RustPath {
        segments: vec!["crate".into(), "foo".into()],
        anchor: RustPathAnchor::Crate,
        resolved: None,
    };
    let v2 = serde_json::to_value(&rust_path).unwrap();
    assert_eq!(v2.get("kind"), Some(&serde_json::Value::String("rustPath".into())));

    let ns = DependencyTarget::Namespace {
        segments: vec!["Foo".into(), "Bar".into()],
        is_static: true,
        alias: None,
    };
    let v3 = serde_json::to_value(&ns).unwrap();
    let obj3 = v3.as_object().unwrap();
    assert_eq!(obj3.get("kind"), Some(&serde_json::Value::String("namespace".into())));
    assert!(obj3.contains_key("is_static"));
    assert!(!obj3.contains_key("isStatic"));

    let json = r#"{"kind":"file","specifier":"./foo","resolved":null,"is_relative":true}"#;
    let round: DependencyTarget = serde_json::from_str(json).unwrap();
    assert!(matches!(round, DependencyTarget::File { is_relative: true, .. }));
}

#[test]
fn dependency_target_unknown_kind_errors() {
    let json = r#"{"kind":"bogus","specifier":"x","resolved":null,"is_relative":false}"#;
    let err = serde_json::from_str::<DependencyTarget>(json).unwrap_err();
    assert!(err.to_string().contains("unknown variant"));
}

#[test]
fn dependency_target_camel_case_field_key_fails_missing_field() {
    let json = r#"{"kind":"file","specifier":"x","resolved":null,"isRelative":false}"#;
    let err = serde_json::from_str::<DependencyTarget>(json).unwrap_err();
    assert!(err.to_string().contains("missing field `is_relative`"));
}

#[test]
fn dependency_target_option_fields_lenient_on_missing_key() {
    let json = r#"{"kind":"file","specifier":"x","is_relative":false}"#;
    let v: DependencyTarget = serde_json::from_str(json).unwrap();
    assert!(matches!(v, DependencyTarget::File { resolved: None, .. }));

    let json2 = r#"{"kind":"namespace","segments":[],"is_static":false}"#;
    let v2: DependencyTarget = serde_json::from_str(json2).unwrap();
    assert!(matches!(v2, DependencyTarget::Namespace { alias: None, .. }));

    let json3 = r#"{"kind":"rustPath","segments":["crate","foo"],"anchor":"crate"}"#;
    let v3: DependencyTarget = serde_json::from_str(json3).unwrap();
    assert!(matches!(v3, DependencyTarget::RustPath { resolved: None, .. }));

    let err_specifier =
        serde_json::from_str::<DependencyTarget>(r#"{"kind":"file","is_relative":false}"#).unwrap_err();
    assert!(err_specifier.to_string().contains("missing field `specifier`"));

    let err_rust_path_segments =
        serde_json::from_str::<DependencyTarget>(r#"{"kind":"rustPath","anchor":"crate"}"#).unwrap_err();
    assert!(err_rust_path_segments.to_string().contains("missing field `segments`"));

    let err_namespace_segments =
        serde_json::from_str::<DependencyTarget>(r#"{"kind":"namespace","is_static":false}"#).unwrap_err();
    assert!(err_namespace_segments.to_string().contains("missing field `segments`"));

    let err_namespace_is_static =
        serde_json::from_str::<DependencyTarget>(r#"{"kind":"namespace","segments":[]}"#).unwrap_err();
    assert!(err_namespace_is_static.to_string().contains("missing field `is_static`"));
}

#[test]
fn dependency_target_namespace_round_trips_normally() {
    let ns = DependencyTarget::Namespace {
        segments: vec!["System".into(), "Collections".into()],
        is_static: false,
        alias: Some("Coll".into()),
    };
    let s = serde_json::to_string(&ns).unwrap();
    let round: DependencyTarget = serde_json::from_str(&s).unwrap();
    assert!(matches!(
        round,
        DependencyTarget::Namespace { alias: Some(ref a), is_static: false, .. } if a == "Coll"
    ));
}

use crate::types::DependencyBinding;

#[test]
fn dependency_binding_missing_required_field_errors() {
    let err_imported = serde_json::from_str::<DependencyBinding>(r#"{"local":"x","kind":"named"}"#).unwrap_err();
    assert!(err_imported.to_string().contains("missing field `imported`"));
    let err_local = serde_json::from_str::<DependencyBinding>(r#"{"imported":"x","kind":"named"}"#).unwrap_err();
    assert!(err_local.to_string().contains("missing field `local`"));
    let err_kind = serde_json::from_str::<DependencyBinding>(r#"{"imported":"x","local":"y"}"#).unwrap_err();
    assert!(err_kind.to_string().contains("missing field `kind`"));
}

use crate::types::DependencyRecord;

#[test]
fn dependency_record_bindings_default_and_required_fields() {
    let json = r#"{"line":1,"target":{"kind":"file","specifier":"x","resolved":null,"is_relative":false}}"#;
    let rec: DependencyRecord = serde_json::from_str(json).unwrap();
    assert_eq!(rec.bindings.len(), 0);

    let v = serde_json::to_value(&rec).unwrap();
    let obj = v.as_object().unwrap();
    assert!(obj.contains_key("bindings"));
    assert_eq!(obj.get("bindings"), Some(&serde_json::Value::Array(vec![])));

    let err_line = serde_json::from_str::<DependencyRecord>(
        r#"{"target":{"kind":"file","specifier":"x","resolved":null,"is_relative":false}}"#,
    )
    .unwrap_err();
    assert!(err_line.to_string().contains("missing field `line`"));

    let err_target = serde_json::from_str::<DependencyRecord>(r#"{"line":1}"#).unwrap_err();
    assert!(err_target.to_string().contains("missing field `target`"));
}

use crate::types::ExportRecord;

#[test]
fn export_record_required_fields_and_camel_case() {
    let rec = ExportRecord { name: "foo".into(), line: 3, re_export: true };
    let s = serde_json::to_string(&rec).unwrap();
    assert!(s.contains("\"reExport\":true"));
    assert!(!s.contains("re_export"));

    let err_name = serde_json::from_str::<ExportRecord>(r#"{"line":1,"reExport":false}"#).unwrap_err();
    assert!(err_name.to_string().contains("missing field `name`"));
    let err_line = serde_json::from_str::<ExportRecord>(r#"{"name":"foo","reExport":false}"#).unwrap_err();
    assert!(err_line.to_string().contains("missing field `line`"));
    let err_re = serde_json::from_str::<ExportRecord>(r#"{"name":"foo","line":1}"#).unwrap_err();
    assert!(err_re.to_string().contains("missing field `reExport`"));
}

use crate::types::JsxAttribute;

#[test]
fn jsx_attribute_required_fields_and_string_value_leniency() {
    let attr = JsxAttribute { name: "foo".into(), string_value: None, is_expression: false, is_spread: false };
    let v = serde_json::to_value(&attr).unwrap();
    let obj = v.as_object().unwrap();
    assert!(obj.contains_key("stringValue"));
    assert_eq!(obj.get("stringValue"), Some(&serde_json::Value::Null));

    let json = r#"{"name":"foo","isExpression":false,"isSpread":false}"#;
    let parsed: JsxAttribute = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.string_value, None);

    let err_name = serde_json::from_str::<JsxAttribute>(r#"{"isExpression":false,"isSpread":false}"#).unwrap_err();
    assert!(err_name.to_string().contains("missing field `name`"));
    let err_expr = serde_json::from_str::<JsxAttribute>(r#"{"name":"foo","isSpread":false}"#).unwrap_err();
    assert!(err_expr.to_string().contains("missing field `isExpression`"));
    let err_spread = serde_json::from_str::<JsxAttribute>(r#"{"name":"foo","isExpression":false}"#).unwrap_err();
    assert!(err_spread.to_string().contains("missing field `isSpread`"));
}

use crate::types::JsxElementEntry;

#[test]
fn jsx_element_entry_attributes_required_no_default_leniency() {
    let err = serde_json::from_str::<JsxElementEntry>(r#"{"name":"Foo","isHtml":true,"line":1}"#).unwrap_err();
    assert!(err.to_string().contains("missing field `attributes`"));

    let json = r#"{"name":"Foo","isHtml":true,"line":1,"attributes":[]}"#;
    let parsed: JsxElementEntry = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.attributes.len(), 0);

    let err_name =
        serde_json::from_str::<JsxElementEntry>(r#"{"isHtml":true,"line":1,"attributes":[]}"#).unwrap_err();
    assert!(err_name.to_string().contains("missing field `name`"));

    let err_is_html =
        serde_json::from_str::<JsxElementEntry>(r#"{"name":"Foo","line":1,"attributes":[]}"#).unwrap_err();
    assert!(err_is_html.to_string().contains("missing field `isHtml`"));

    let err_line =
        serde_json::from_str::<JsxElementEntry>(r#"{"name":"Foo","isHtml":true,"attributes":[]}"#).unwrap_err();
    assert!(err_line.to_string().contains("missing field `line`"));
}

use crate::types::{GoData, JavaData, PythonData, RustData};

#[test]
fn empty_lang_data_structs_serialize_as_empty_object() {
    assert_eq!(serde_json::to_string(&RustData {}).unwrap(), "{}");
    assert_eq!(serde_json::to_string(&PythonData {}).unwrap(), "{}");
    assert_eq!(serde_json::to_string(&GoData {}).unwrap(), "{}");
    assert_eq!(serde_json::to_string(&JavaData {}).unwrap(), "{}");

    let _: RustData = serde_json::from_str("{}").unwrap();
    let _: PythonData = serde_json::from_str("{}").unwrap();
    let _: GoData = serde_json::from_str("{}").unwrap();
    let _: JavaData = serde_json::from_str("{}").unwrap();

    fn assert_default<T: Default>() {}
    assert_default::<RustData>();
    assert_default::<PythonData>();
    assert_default::<GoData>();
    assert_default::<JavaData>();
}

use crate::types::TsData;

#[test]
fn ts_data_default_leniency_and_selective_skip_serializing() {
    let d = TsData::default();
    assert_eq!(d.jsx_elements.len(), 0);
    assert_eq!(d.type_only_imports.len(), 0);
    assert_eq!(d.unresolved_aliases.len(), 0);

    let empty: TsData = serde_json::from_str("{}").unwrap();
    assert_eq!(empty.jsx_elements.len(), 0);
    assert_eq!(empty.type_only_imports.len(), 0);
    assert_eq!(empty.unresolved_aliases.len(), 0);

    let v = serde_json::to_value(&empty).unwrap();
    let obj = v.as_object().unwrap();
    assert!(obj.contains_key("jsxElements"));
    assert!(obj.contains_key("typeOnlyImports"));
    assert!(!obj.contains_key("unresolvedAliases"));
}

use crate::types::LangData;

#[test]
fn lang_data_adjacent_tag_wire_shape() {
    let ts = LangData::Ts(TsData::default());
    let v = serde_json::to_value(&ts).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("language"), Some(&serde_json::Value::String("typescript".into())));
    assert!(obj.contains_key("data"));
    assert_eq!(obj.len(), 2);

    let json = r#"{"language":"typescript","data":{"jsxElements":[],"typeOnlyImports":[]}}"#;
    let round: LangData = serde_json::from_str(json).unwrap();
    assert!(matches!(round, LangData::Ts(_)));
}

#[test]
fn lang_data_tag_values_necessary_vs_redundant_renames() {
    let pairs: [(LangData, &str); 5] = [
        (LangData::Ts(TsData::default()), "typescript"),
        (LangData::Rust(RustData {}), "rust"),
        (LangData::Python(PythonData {}), "python"),
        (LangData::Go(GoData {}), "go"),
        (LangData::Java(JavaData {}), "java"),
    ];
    for (value, expected_tag) in pairs {
        let v = serde_json::to_value(&value).unwrap();
        assert_eq!(v.get("language"), Some(&serde_json::Value::String(expected_tag.into())));
    }
}

#[test]
fn lang_data_unknown_language_tag_errors() {
    let json = r#"{"language":"csharp","data":{}}"#;
    let err = serde_json::from_str::<LangData>(json).unwrap_err();
    assert!(err.to_string().contains("unknown variant"));
    assert!(err.to_string().contains("expected one of `typescript`, `rust`, `python`, `go`, `java`"));
}

#[test]
fn lang_data_malformed_data_shape_errors_per_inner_struct() {
    let json = r#"{"language":"typescript","data":{"jsxElements":"not-an-array"}}"#;
    let err = serde_json::from_str::<LangData>(json).unwrap_err();
    assert!(err.to_string().contains("invalid type"));
}

#[test]
fn lang_data_extra_key_in_data_silently_discarded() {
    let json = r#"{"language":"typescript","data":{"jsxElements":[],"typeOnlyImports":[],"bogusKey":123}}"#;
    let parsed: LangData = serde_json::from_str(json).unwrap();
    assert!(matches!(parsed, LangData::Ts(_)));
}

#[test]
fn lang_data_ts_and_jsx_elements_methods() {
    let entry = JsxElementEntry { name: "div".into(), is_html: true, line: 1, attributes: vec![] };
    let ts_data = TsData { jsx_elements: vec![entry.clone()], type_only_imports: vec![], unresolved_aliases: vec![] };
    let ts = LangData::Ts(ts_data);
    assert!(ts.ts().is_some());
    assert_eq!(ts.jsx_elements().len(), 1);
    assert_eq!(ts.jsx_elements()[0].name, "div");

    let others: [LangData; 4] = [
        LangData::Rust(RustData {}),
        LangData::Python(PythonData {}),
        LangData::Go(GoData {}),
        LangData::Java(JavaData {}),
    ];
    for variant in others {
        assert!(variant.ts().is_none());
        assert_eq!(variant.jsx_elements().len(), 0);
    }
}


#[test]
fn extra_unknown_key_silently_discarded_across_cluster() {
    let json = r#"{"name":"foo","line":1,"reExport":false,"bogusKey":123}"#;
    let rec: ExportRecord = serde_json::from_str(json).unwrap();
    assert_eq!(rec.name, "foo");

    let json2 = r#"{"bogusKey":123}"#;
    let _: RustData = serde_json::from_str(json2).unwrap();

    let json3 = r#"{"kind":"file","specifier":"x","resolved":null,"is_relative":false,"bogusKey":123}"#;
    let dt: DependencyTarget = serde_json::from_str(json3).unwrap();
    assert!(matches!(dt, DependencyTarget::File { .. }));

    let json4 = r#"{"language":"rust","data":{},"bogusKey":123}"#;
    let ld: LangData = serde_json::from_str(json4).unwrap();
    assert!(matches!(ld, LangData::Rust(_)));
}


#[test]
fn binding_kind_external_tagging_shapes() {
    let v: BindingKind = serde_json::from_str(r#""named""#).unwrap();
    assert!(matches!(v, BindingKind::Named));

    let err = serde_json::from_str::<BindingKind>(r#""bogus""#).unwrap_err();
    assert!(err.to_string().contains("unknown variant"));
    assert!(err.to_string().contains("expected one of `named`, `default`, `namespace`, `glob`, `namespaceWide`"));

    let v2: BindingKind = serde_json::from_str(r#"{"named":null}"#).unwrap();
    assert!(matches!(v2, BindingKind::Named));
    let v3: BindingKind = serde_json::from_str(r#"{"namespaceWide":null}"#).unwrap();
    assert!(matches!(v3, BindingKind::NamespaceWide));

    for (bad, want) in [
        (r#"{"named":true}"#, "invalid type: boolean `true`, expected unit"),
        (r#"{"named":"x"}"#, "invalid type: string \"x\", expected unit"),
        (r#"{"named":1}"#, "invalid type: integer `1`, expected unit"),
        (r#"{"named":{}}"#, "invalid type: map, expected unit"),
        (r#"{"named":[]}"#, "invalid type: sequence, expected unit"),
    ] {
        let e = serde_json::from_str::<BindingKind>(bad).unwrap_err();
        assert!(e.to_string().contains(want));
    }

    let err3 = serde_json::from_str::<BindingKind>(r#"{"named":null,"bogusKey":1}"#).unwrap_err();
    assert!(err3.to_string().contains("expected value"));

    for bad in ["123", "true", "null", r#"["named"]"#] {
        let e = serde_json::from_str::<BindingKind>(bad).unwrap_err();
        assert!(e.to_string().contains("expected value"));
    }
}

#[test]
fn binding_kind_variant_count_and_order_pinned() {
    let assert_shape = |k: BindingKind| match k {
        BindingKind::Named
        | BindingKind::Default
        | BindingKind::Namespace
        | BindingKind::Glob
        | BindingKind::NamespaceWide => {}
    };
    assert_shape(BindingKind::Named);

    assert_eq!(BindingKind::Named as usize, 0);
    assert_eq!(BindingKind::Default as usize, 1);
    assert_eq!(BindingKind::Namespace as usize, 2);
    assert_eq!(BindingKind::Glob as usize, 3);
    assert_eq!(BindingKind::NamespaceWide as usize, 4);
}

#[test]
fn rust_path_anchor_unknown_variant_and_second_key_rejected() {
    let err = serde_json::from_str::<RustPathAnchor>(r#""bogus""#).unwrap_err();
    assert!(err.to_string().contains("unknown variant"));
    assert!(err.to_string().contains("expected one of `crate`, `super`, `self`, `extern`"));

    let err2 = serde_json::from_str::<RustPathAnchor>(r#"{"extern":"foo","bogusKey":123}"#).unwrap_err();
    assert!(err2.to_string().contains("expected value"));
}

#[test]
fn extra_unknown_key_silently_discarded_remaining_cluster_types() {
    let json = r#"{"line":1,"target":{"kind":"file","specifier":"x","resolved":null,"is_relative":false},"bogusKey":123}"#;
    let rec: DependencyRecord = serde_json::from_str(json).unwrap();
    assert_eq!(rec.line, 1);

    let json2 = r#"{"imported":"a","local":"b","kind":"named","bogusKey":123}"#;
    let binding: DependencyBinding = serde_json::from_str(json2).unwrap();
    assert_eq!(binding.imported, "a");

    let json3 = r#"{"name":"foo","isExpression":false,"isSpread":false,"bogusKey":123}"#;
    let attr: JsxAttribute = serde_json::from_str(json3).unwrap();
    assert_eq!(attr.name, "foo");

    let json4 = r#"{"name":"Foo","isHtml":true,"line":1,"attributes":[],"bogusKey":123}"#;
    let elem: JsxElementEntry = serde_json::from_str(json4).unwrap();
    assert_eq!(elem.name, "Foo");

    let json5 = r#"{"bogusKey":123}"#;
    let _: PythonData = serde_json::from_str(json5).unwrap();
    let _: GoData = serde_json::from_str(json5).unwrap();
    let _: JavaData = serde_json::from_str(json5).unwrap();
}

#[test]
fn dependency_record_full_round_trip_exact_shape() {
    let json = r#"{"line":10,"bindings":[{"imported":"a","local":"b","kind":"named"}],"target":{"kind":"file","specifier":"./foo.ts","resolved":"src/foo.ts","is_relative":true}}"#;
    let rec: DependencyRecord = serde_json::from_str(json).unwrap();
    assert_eq!(rec.line, 10);
    assert_eq!(rec.bindings.len(), 1);
    assert_eq!(rec.bindings[0].imported, "a");
    assert_eq!(rec.bindings[0].local, "b");
    assert!(matches!(rec.bindings[0].kind, BindingKind::Named));
    assert!(matches!(
        rec.target,
        DependencyTarget::File { ref specifier, resolved: Some(ref resolved), is_relative: true }
            if specifier == "./foo.ts" && resolved.as_str() == "src/foo.ts"
    ));

    let reserialized = serde_json::to_string(&rec).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn dependency_binding_full_round_trip_exact_shape() {
    let json = r#"{"imported":"foo","local":"bar","kind":"glob"}"#;
    let binding: DependencyBinding = serde_json::from_str(json).unwrap();
    assert_eq!(binding.imported, "foo");
    assert_eq!(binding.local, "bar");
    assert!(matches!(binding.kind, BindingKind::Glob));

    let reserialized = serde_json::to_string(&binding).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn export_record_full_round_trip_exact_shape() {
    let json = r#"{"name":"foo","line":5,"reExport":true}"#;
    let rec: ExportRecord = serde_json::from_str(json).unwrap();
    assert_eq!(rec.name, "foo");
    assert_eq!(rec.line, 5);
    assert!(rec.re_export);

    let reserialized = serde_json::to_string(&rec).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn jsx_attribute_full_round_trip_exact_shape() {
    let json = r#"{"name":"onClick","stringValue":"handleClick","isExpression":true,"isSpread":false}"#;
    let attr: JsxAttribute = serde_json::from_str(json).unwrap();
    assert_eq!(attr.name, "onClick");
    assert_eq!(attr.string_value.as_deref(), Some("handleClick"));
    assert!(attr.is_expression);
    assert!(!attr.is_spread);

    let reserialized = serde_json::to_string(&attr).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn jsx_element_entry_full_round_trip_exact_shape() {
    let json = r#"{"name":"div","isHtml":true,"line":12,"attributes":[{"name":"id","stringValue":"main","isExpression":false,"isSpread":false}]}"#;
    let elem: JsxElementEntry = serde_json::from_str(json).unwrap();
    assert_eq!(elem.name, "div");
    assert!(elem.is_html);
    assert_eq!(elem.line, 12);
    assert_eq!(elem.attributes.len(), 1);
    assert_eq!(elem.attributes[0].name, "id");
    assert_eq!(elem.attributes[0].string_value.as_deref(), Some("main"));
    assert!(!elem.attributes[0].is_expression);
    assert!(!elem.attributes[0].is_spread);

    let reserialized = serde_json::to_string(&elem).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn ts_data_full_round_trip_exact_shape() {
    let json = r#"{"jsxElements":[{"name":"Foo","isHtml":false,"line":3,"attributes":[]}],"typeOnlyImports":["TypeA"]}"#;
    let data: TsData = serde_json::from_str(json).unwrap();
    assert_eq!(data.jsx_elements.len(), 1);
    assert_eq!(data.jsx_elements[0].name, "Foo");
    assert_eq!(data.type_only_imports, vec!["TypeA".to_string()]);
    assert_eq!(data.unresolved_aliases.len(), 0);

    let reserialized = serde_json::to_string(&data).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn dependency_target_rust_path_full_round_trip_exact_shape() {
    let json = r#"{"kind":"rustPath","segments":["crate","foo","Bar"],"anchor":"super","resolved":"target/foo.rs"}"#;
    let target: DependencyTarget = serde_json::from_str(json).unwrap();
    assert!(matches!(
        target,
        DependencyTarget::RustPath { ref segments, anchor: RustPathAnchor::Super, resolved: Some(ref resolved) }
            if segments == &["crate", "foo", "Bar"] && resolved.as_str() == "target/foo.rs"
    ));

    let reserialized = serde_json::to_string(&target).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn dependency_target_namespace_full_round_trip_exact_shape() {
    let json = r#"{"kind":"namespace","segments":["System","Collections"],"is_static":true,"alias":"Coll"}"#;
    let target: DependencyTarget = serde_json::from_str(json).unwrap();
    assert!(matches!(
        target,
        DependencyTarget::Namespace { ref segments, is_static: true, alias: Some(ref alias) }
            if segments == &["System", "Collections"] && alias == "Coll"
    ));

    let reserialized = serde_json::to_string(&target).unwrap();
    assert_eq!(reserialized, json);
}

#[test]
fn dependency_record_line_negative_errors() {
    let json = r#"{"line":-5,"target":{"kind":"file","specifier":"x","resolved":null,"is_relative":false}}"#;
    let err = serde_json::from_str::<DependencyRecord>(json).unwrap_err();
    assert!(err.to_string().contains("invalid value"));
    assert!(err.to_string().contains("usize"));
}

#[test]
fn export_record_line_negative_errors() {
    let json = r#"{"name":"foo","line":-5,"reExport":false}"#;
    let err = serde_json::from_str::<ExportRecord>(json).unwrap_err();
    assert!(err.to_string().contains("invalid value"));
    assert!(err.to_string().contains("usize"));
}

#[test]
fn jsx_element_entry_line_negative_errors() {
    let json = r#"{"name":"Foo","isHtml":true,"line":-5,"attributes":[]}"#;
    let err = serde_json::from_str::<JsxElementEntry>(json).unwrap_err();
    assert!(err.to_string().contains("invalid value"));
    assert!(err.to_string().contains("usize"));
}

#[test]
fn spec_005_derive_lists_pinned() {
    fn assert_copy_eq_derives<T: std::fmt::Debug + Clone + Copy + PartialEq + Eq>() {}
    fn assert_clone_derives<T: std::fmt::Debug + Clone>() {}
    fn assert_clone_default_derives<T: std::fmt::Debug + Clone + Default>() {}

    // AC-003: BindingKind is a fieldless enum -- Copy/PartialEq/Eq required,
    // matching SymbolKind's and Visibility's pattern.
    assert_copy_eq_derives::<BindingKind>();

    // AC-008, AC-005, AC-002, AC-001, AC-009, AC-010, AC-011, AC-012: none of
    // these carry Copy/PartialEq/Eq. A generic bound check can only prove a
    // trait IS implemented, not that it is absent, so this pins Debug+Clone
    // only -- it does not (and cannot, on stable Rust) prove the absence of
    // Copy/PartialEq/Eq on any of these types. In particular, AC-002 states
    // DependencyBinding must NOT derive PartialEq/Eq/Copy; that specific
    // absence is not covered by this check and would need a different
    // technique to pin.
    assert_clone_derives::<RustPathAnchor>();
    assert_clone_derives::<DependencyTarget>();
    assert_clone_derives::<DependencyBinding>();
    assert_clone_derives::<DependencyRecord>();
    assert_clone_derives::<ExportRecord>();
    assert_clone_derives::<JsxAttribute>();
    assert_clone_derives::<JsxElementEntry>();
    assert_clone_derives::<LangData>();

    // AC-017, AC-018: TsData and the four empty per-language data structs
    // additionally derive Default.
    assert_clone_default_derives::<TsData>();
    assert_clone_default_derives::<RustData>();
    assert_clone_default_derives::<PythonData>();
    assert_clone_default_derives::<GoData>();
    assert_clone_default_derives::<JavaData>();

    // AC-001, AC-009, AC-011: line:usize pinned exactly (not i64/u32) on
    // DependencyRecord, ExportRecord, and JsxElementEntry, mirroring
    // field_types_and_derives_pinned's compile-time binding for
    // SymbolEntry.line (SPEC-003).
    let dep_json = r#"{"line":10,"target":{"kind":"file","specifier":"x","resolved":null,"is_relative":false}}"#;
    let dep: DependencyRecord = serde_json::from_str(dep_json).unwrap();
    let dep_line: usize = dep.line;
    assert_eq!(dep_line, 10);

    let export_json = r#"{"name":"foo","line":10,"reExport":false}"#;
    let export: ExportRecord = serde_json::from_str(export_json).unwrap();
    let export_line: usize = export.line;
    assert_eq!(export_line, 10);

    let jsx_json = r#"{"name":"Foo","isHtml":true,"line":10,"attributes":[]}"#;
    let jsx: JsxElementEntry = serde_json::from_str(jsx_json).unwrap();
    let jsx_line: usize = jsx.line;
    assert_eq!(jsx_line, 10);
}

fn enum_declaration_block<'a>(src: &'a str, decl: &str) -> &'a str {
    let start = src.find(decl).unwrap_or_else(|| panic!("declaration `{decl}` not found in types.rs"));
    let open = src[start..].find('{').unwrap_or_else(|| panic!("no opening brace found for `{decl}`")) + start;
    let mut depth = 0usize;
    for (i, ch) in src[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[open..=open + i];
                }
            }
            _ => {}
        }
    }
    panic!("no matching closing brace found for `{decl}`");
}

/// `as usize` discriminant casts (used elsewhere in this file to pin
/// declaration order for the fieldless SymbolKind/Visibility/BindingKind
/// enums) only apply to C-like enums with no data-carrying variants --
/// RustPathAnchor, DependencyTarget, and LangData all have at least one
/// variant that carries data, so that technique doesn't apply to them.
/// Declaration order is instead pinned by checking that each variant name
/// appears at a strictly increasing text position within the enum's own
/// source declaration block, read directly from types.rs via `include_str!`.
fn assert_declaration_order(block: &str, variants_in_order: &[&str]) {
    let mut last_end = 0usize;
    for name in variants_in_order {
        let found = block[last_end..].find(name).unwrap_or_else(|| {
            panic!("variant `{name}` not found after position {last_end} -- out of declaration order or missing")
        });
        last_end += found + name.len();
    }
}

#[test]
fn rust_path_anchor_variant_set_and_declaration_order_pinned() {
    // Exhaustive match with no wildcard arm: adding, removing, or renaming a
    // variant is a compile error, pinning the variant SET.
    fn assert_shape(v: RustPathAnchor) {
        match v {
            RustPathAnchor::Crate => {}
            RustPathAnchor::Super => {}
            RustPathAnchor::Selff => {}
            RustPathAnchor::Extern(_) => {}
        }
    }
    assert_shape(RustPathAnchor::Crate);

    let src = include_str!("types.rs");
    let block = enum_declaration_block(src, "pub enum RustPathAnchor");
    assert_declaration_order(block, &["Crate", "Super", "Selff", "Extern"]);
}

#[test]
fn dependency_target_variant_set_and_declaration_order_pinned() {
    fn assert_shape(v: DependencyTarget) {
        match v {
            DependencyTarget::File { .. } => {}
            DependencyTarget::RustPath { .. } => {}
            DependencyTarget::Namespace { .. } => {}
        }
    }
    assert_shape(DependencyTarget::Namespace { segments: vec![], is_static: false, alias: None });

    let src = include_str!("types.rs");
    let block = enum_declaration_block(src, "pub enum DependencyTarget");
    assert_declaration_order(block, &["File", "RustPath", "Namespace"]);
}

#[test]
fn lang_data_variant_set_and_declaration_order_pinned() {
    fn assert_shape(v: LangData) {
        match v {
            LangData::Ts(_) => {}
            LangData::Rust(_) => {}
            LangData::Python(_) => {}
            LangData::Go(_) => {}
            LangData::Java(_) => {}
        }
    }
    assert_shape(LangData::Rust(RustData {}));

    let src = include_str!("types.rs");
    let block = enum_declaration_block(src, "pub enum LangData");
    assert_declaration_order(block, &["Ts", "Rust", "Python", "Go", "Java"]);
}

#[test]
fn dependency_target_option_fields_key_present_and_typed_when_none() {
    let file = DependencyTarget::File { specifier: "x".into(), resolved: None, is_relative: false };
    let v = serde_json::to_value(&file).unwrap();
    let obj = v.as_object().unwrap();
    assert!(obj.contains_key("resolved"));
    assert_eq!(obj.get("resolved"), Some(&serde_json::Value::Null));

    let rust_path = DependencyTarget::RustPath { segments: vec![], anchor: RustPathAnchor::Crate, resolved: None };
    let v2 = serde_json::to_value(&rust_path).unwrap();
    let obj2 = v2.as_object().unwrap();
    assert!(obj2.contains_key("resolved"));
    assert_eq!(obj2.get("resolved"), Some(&serde_json::Value::Null));

    let ns = DependencyTarget::Namespace { segments: vec![], is_static: false, alias: None };
    let v3 = serde_json::to_value(&ns).unwrap();
    let obj3 = v3.as_object().unwrap();
    assert!(obj3.contains_key("alias"));
    assert_eq!(obj3.get("alias"), Some(&serde_json::Value::Null));

    // Compile-time type pins (AC-005): File.resolved and RustPath.resolved are
    // exactly Option<Utf8PathBuf>, not Option<String> -- both serialize as a
    // JSON string or null identically either way, so only a compile-time
    // binding distinguishes them (same technique as field_types_and_derives_pinned
    // and spec_005_derive_lists_pinned's `usize` pins).
    if let DependencyTarget::File { resolved, .. } = file {
        let pinned: Option<camino::Utf8PathBuf> = resolved;
        assert_eq!(pinned, None);
    }
    if let DependencyTarget::RustPath { resolved, .. } = rust_path {
        let pinned: Option<camino::Utf8PathBuf> = resolved;
        assert_eq!(pinned, None);
    }
}

#[test]
fn dependency_target_rust_path_anchor_required_field_errors() {
    let err = serde_json::from_str::<DependencyTarget>(
        r#"{"kind":"rustPath","segments":["crate","foo"],"resolved":null}"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("missing field `anchor`"));
}

#[test]
fn ts_data_unresolved_aliases_populated_full_round_trip() {
    let data = TsData {
        jsx_elements: vec![],
        type_only_imports: vec![],
        unresolved_aliases: vec!["Foo".to_string(), "Bar".to_string()],
    };
    let json = r#"{"jsxElements":[],"typeOnlyImports":[],"unresolvedAliases":["Foo","Bar"]}"#;
    let s = serde_json::to_string(&data).unwrap();
    assert_eq!(s, json);

    let round: TsData = serde_json::from_str(json).unwrap();
    assert_eq!(round.unresolved_aliases, vec!["Foo".to_string(), "Bar".to_string()]);

    // Compile-time type pin (AC-017): unresolved_aliases is exactly
    // Vec<String>, not e.g. Vec<Utf8PathBuf> -- both serialize identically as
    // a JSON array of strings, so only a compile-time binding distinguishes
    // them.
    let pinned: Vec<String> = round.unresolved_aliases;
    assert_eq!(pinned, vec!["Foo".to_string(), "Bar".to_string()]);
}

#[test]
fn lang_data_jsx_elements_returns_full_slice_not_truncated() {
    let entries = vec![
        JsxElementEntry { name: "div".into(), is_html: true, line: 1, attributes: vec![] },
        JsxElementEntry { name: "Foo".into(), is_html: false, line: 2, attributes: vec![] },
        JsxElementEntry { name: "span".into(), is_html: true, line: 3, attributes: vec![] },
    ];
    let ts_data = TsData { jsx_elements: entries, type_only_imports: vec![], unresolved_aliases: vec![] };
    let ts = LangData::Ts(ts_data);
    assert_eq!(ts.jsx_elements().len(), 3);
    assert_eq!(ts.jsx_elements()[0].name, "div");
    assert_eq!(ts.jsx_elements()[1].name, "Foo");
    assert_eq!(ts.jsx_elements()[2].name, "span");
}

/// Exhaustive compile-time field-type pinning for every field of every
/// SPEC-005 type. serde treats several distinct Rust types identically on
/// the wire (`String` vs `camino::Utf8PathBuf`, `Vec<String>` vs
/// `Vec<Utf8PathBuf>`, a struct field vs its boxed equivalent, etc.), so a
/// runtime assertion on a deserialized value can never distinguish the
/// declared field type from a serde-equivalent sibling -- only a
/// compile-time `let x: ExactType = value.field` binding can. Five prior
/// fix rounds each pinned only the specific fields a gate pass happened to
/// name; this test instead inventories and pins EVERY field/payload of all
/// 14 SPEC-005 types in one place, so completeness is self-evident without
/// cross-referencing types.rs by hand.
///
/// Full inventory (35 fields/payloads total; 6 already had a typed pin
/// before this test -- DependencyRecord.line, ExportRecord.line,
/// JsxElementEntry.line, DependencyTarget::File.resolved,
/// DependencyTarget::RustPath.resolved, TsData.unresolved_aliases, see
/// `spec_005_derive_lists_pinned` and
/// `dependency_target_option_fields_key_present_and_typed_when_none` above
/// -- this test does not duplicate those 6, only the remaining 29):
///
/// - BindingKind: fieldless (5 variants) -- nothing to pin.
/// - RustPathAnchor: Crate/Super/Selff fieldless; Extern(String) payload --
///   pinned below.
/// - DependencyTarget::File: specifier: String, resolved: Option<Utf8PathBuf>
///   (already pinned), is_relative: bool -- specifier and is_relative
///   pinned below.
/// - DependencyTarget::RustPath: segments: Vec<String>, anchor:
///   RustPathAnchor, resolved: Option<Utf8PathBuf> (already pinned) --
///   segments and anchor pinned below.
/// - DependencyTarget::Namespace: segments: Vec<String>, is_static: bool,
///   alias: Option<String> -- all 3 pinned below.
/// - DependencyBinding: imported: String, local: String, kind: BindingKind
///   -- all 3 pinned below.
/// - DependencyRecord: line: usize (already pinned), bindings:
///   Vec<DependencyBinding>, target: DependencyTarget -- bindings and
///   target pinned below.
/// - ExportRecord: name: String, line: usize (already pinned), re_export:
///   bool -- name and re_export pinned below.
/// - JsxAttribute: name: String, string_value: Option<String>,
///   is_expression: bool, is_spread: bool -- all 4 pinned below.
/// - JsxElementEntry: name: String, is_html: bool, line: usize (already
///   pinned), attributes: Vec<JsxAttribute> -- name, is_html, attributes
///   pinned below.
/// - RustData, PythonData, GoData, JavaData: fieldless -- nothing to pin.
/// - TsData: jsx_elements: Vec<JsxElementEntry>, type_only_imports:
///   Vec<String>, unresolved_aliases: Vec<String> (already pinned) --
///   jsx_elements and type_only_imports pinned below.
/// - LangData: Ts(TsData), Rust(RustData), Python(PythonData), Go(GoData),
///   Java(JavaData) -- all 5 wrapped payload types pinned below (each
///   variant's own inner fields are covered by that type's own entry
///   above, not repeated here).
///
/// Every `match`/`if let` below has no wildcard arm on the field-bearing
/// side, so a variant rename or a field being added/removed is a compile
/// error here too, not just a silent miss.
#[test]
fn spec_005_all_field_types_pinned() {
    // -- RustPathAnchor::Extern(String) --
    match RustPathAnchor::Extern("foo".into()) {
        RustPathAnchor::Extern(payload) => {
            let pinned: String = payload;
            assert_eq!(pinned, "foo");
        }
        RustPathAnchor::Crate | RustPathAnchor::Super | RustPathAnchor::Selff => {
            panic!("expected Extern variant")
        }
    }

    // -- DependencyTarget::File { specifier, is_relative, .. } --
    // (resolved already pinned in dependency_target_option_fields_key_present_and_typed_when_none)
    let file = DependencyTarget::File { specifier: "spec".into(), resolved: None, is_relative: true };
    if let DependencyTarget::File { specifier, is_relative, .. } = file {
        let pinned_specifier: String = specifier;
        let pinned_is_relative: bool = is_relative;
        assert_eq!(pinned_specifier, "spec");
        assert!(pinned_is_relative);
    } else {
        panic!("expected File variant");
    }

    // -- DependencyTarget::RustPath { segments, anchor, .. } --
    // (resolved already pinned in dependency_target_option_fields_key_present_and_typed_when_none)
    let rust_path =
        DependencyTarget::RustPath { segments: vec!["crate".into(), "foo".into()], anchor: RustPathAnchor::Crate, resolved: None };
    if let DependencyTarget::RustPath { segments, anchor, .. } = rust_path {
        let pinned_segments: Vec<String> = segments;
        let pinned_anchor: RustPathAnchor = anchor;
        assert_eq!(pinned_segments, vec!["crate".to_string(), "foo".to_string()]);
        assert!(matches!(pinned_anchor, RustPathAnchor::Crate));
    } else {
        panic!("expected RustPath variant");
    }

    // -- DependencyTarget::Namespace { segments, is_static, alias } --
    let ns = DependencyTarget::Namespace { segments: vec!["System".into()], is_static: true, alias: Some("S".into()) };
    if let DependencyTarget::Namespace { segments, is_static, alias } = ns {
        let pinned_segments: Vec<String> = segments;
        let pinned_is_static: bool = is_static;
        let pinned_alias: Option<String> = alias;
        assert_eq!(pinned_segments, vec!["System".to_string()]);
        assert!(pinned_is_static);
        assert_eq!(pinned_alias, Some("S".to_string()));
    } else {
        panic!("expected Namespace variant");
    }

    // -- DependencyBinding { imported, local, kind } --
    let binding = DependencyBinding { imported: "im".into(), local: "lo".into(), kind: BindingKind::Named };
    let DependencyBinding { imported, local, kind } = binding;
    let pinned_imported: String = imported;
    let pinned_local: String = local;
    let pinned_kind: BindingKind = kind;
    assert_eq!(pinned_imported, "im");
    assert_eq!(pinned_local, "lo");
    assert!(matches!(pinned_kind, BindingKind::Named));

    // -- DependencyRecord { bindings, target, .. } --
    // (line already pinned in spec_005_derive_lists_pinned)
    let rec = DependencyRecord {
        line: 1,
        bindings: vec![DependencyBinding { imported: "a".into(), local: "b".into(), kind: BindingKind::Glob }],
        target: DependencyTarget::File { specifier: "x".into(), resolved: None, is_relative: false },
    };
    let DependencyRecord { bindings, target, .. } = rec;
    let pinned_bindings: Vec<DependencyBinding> = bindings;
    let pinned_target: DependencyTarget = target;
    assert_eq!(pinned_bindings.len(), 1);
    assert!(matches!(pinned_target, DependencyTarget::File { .. }));

    // -- ExportRecord { name, re_export, .. } --
    // (line already pinned in spec_005_derive_lists_pinned)
    let export_rec = ExportRecord { name: "n".into(), line: 1, re_export: true };
    let ExportRecord { name, re_export, .. } = export_rec;
    let pinned_name: String = name;
    let pinned_re_export: bool = re_export;
    assert_eq!(pinned_name, "n");
    assert!(pinned_re_export);

    // -- JsxAttribute { name, string_value, is_expression, is_spread } --
    let attr = JsxAttribute { name: "n".into(), string_value: Some("v".into()), is_expression: true, is_spread: false };
    let JsxAttribute { name, string_value, is_expression, is_spread } = attr;
    let pinned_attr_name: String = name;
    let pinned_string_value: Option<String> = string_value;
    let pinned_is_expression: bool = is_expression;
    let pinned_is_spread: bool = is_spread;
    assert_eq!(pinned_attr_name, "n");
    assert_eq!(pinned_string_value, Some("v".to_string()));
    assert!(pinned_is_expression);
    assert!(!pinned_is_spread);

    // -- JsxElementEntry { name, is_html, attributes, .. } --
    // (line already pinned in spec_005_derive_lists_pinned)
    let elem = JsxElementEntry {
        name: "n".into(),
        is_html: true,
        line: 1,
        attributes: vec![JsxAttribute { name: "a".into(), string_value: None, is_expression: false, is_spread: false }],
    };
    let JsxElementEntry { name, is_html, attributes, .. } = elem;
    let pinned_elem_name: String = name;
    let pinned_is_html: bool = is_html;
    let pinned_attributes: Vec<JsxAttribute> = attributes;
    assert_eq!(pinned_elem_name, "n");
    assert!(pinned_is_html);
    assert_eq!(pinned_attributes.len(), 1);

    // -- TsData { jsx_elements, type_only_imports, .. } --
    // (unresolved_aliases already pinned in ts_data_unresolved_aliases_populated_full_round_trip)
    let ts_data = TsData {
        jsx_elements: vec![JsxElementEntry { name: "d".into(), is_html: true, line: 1, attributes: vec![] }],
        type_only_imports: vec!["T".into()],
        unresolved_aliases: vec![],
    };
    let TsData { jsx_elements, type_only_imports, .. } = ts_data;
    let pinned_jsx_elements: Vec<JsxElementEntry> = jsx_elements;
    let pinned_type_only_imports: Vec<String> = type_only_imports;
    assert_eq!(pinned_jsx_elements.len(), 1);
    assert_eq!(pinned_type_only_imports, vec!["T".to_string()]);

    // -- LangData's 5 wrapped payload types --
    match LangData::Ts(TsData::default()) {
        LangData::Ts(inner) => {
            let pinned: TsData = inner;
            assert_eq!(pinned.jsx_elements.len(), 0);
        }
        LangData::Rust(_) | LangData::Python(_) | LangData::Go(_) | LangData::Java(_) => {
            panic!("expected Ts variant")
        }
    }
    match LangData::Rust(RustData {}) {
        LangData::Rust(inner) => {
            let pinned: RustData = inner;
            let _ = pinned;
        }
        LangData::Ts(_) | LangData::Python(_) | LangData::Go(_) | LangData::Java(_) => {
            panic!("expected Rust variant")
        }
    }
    match LangData::Python(PythonData {}) {
        LangData::Python(inner) => {
            let pinned: PythonData = inner;
            let _ = pinned;
        }
        LangData::Ts(_) | LangData::Rust(_) | LangData::Go(_) | LangData::Java(_) => {
            panic!("expected Python variant")
        }
    }
    match LangData::Go(GoData {}) {
        LangData::Go(inner) => {
            let pinned: GoData = inner;
            let _ = pinned;
        }
        LangData::Ts(_) | LangData::Rust(_) | LangData::Python(_) | LangData::Java(_) => {
            panic!("expected Go variant")
        }
    }
    match LangData::Java(JavaData {}) {
        LangData::Java(inner) => {
            let pinned: JavaData = inner;
            let _ = pinned;
        }
        LangData::Ts(_) | LangData::Rust(_) | LangData::Python(_) | LangData::Go(_) => {
            panic!("expected Java variant")
        }
    }
}

use crate::types::CodeBlock;

#[test]
fn code_block_symbol_signature_null_present_and_matched_vecs_skip_when_empty() {
    let block = CodeBlock {
        file: "src/foo.rs".into(),
        line_start: 1,
        line_end: 5,
        node_kind: SymbolKind::Function,
        code: "fn foo() {}".into(),
        symbol_signature: None,
        matched_lines: vec![],
        matched_keywords: vec![],
    };
    let v = serde_json::to_value(&block).unwrap();
    let obj = v.as_object().unwrap();
    assert!(obj.contains_key("symbolSignature"));
    assert_eq!(obj.get("symbolSignature"), Some(&serde_json::Value::Null));
    assert!(!obj.contains_key("matchedLines"));
    assert!(!obj.contains_key("matchedKeywords"));

    let json = r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":5,"nodeKind":"function","code":"fn foo() {}"}"#;
    let parsed: CodeBlock = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.symbol_signature, None);
    assert_eq!(parsed.matched_lines.len(), 0);
    assert_eq!(parsed.matched_keywords.len(), 0);

    let populated = CodeBlock {
        file: "src/bar.rs".into(),
        line_start: 2,
        line_end: 9,
        node_kind: SymbolKind::Struct,
        code: "struct Bar;".into(),
        symbol_signature: Some("struct Bar".into()),
        matched_lines: vec![2, 3],
        matched_keywords: vec!["Bar".into()],
    };
    let v2 = serde_json::to_value(&populated).unwrap();
    let obj2 = v2.as_object().unwrap();
    assert!(obj2.contains_key("matchedLines"));
    assert!(obj2.contains_key("matchedKeywords"));

    let expected = r#"{"file":"src/bar.rs","lineStart":2,"lineEnd":9,"nodeKind":"struct","code":"struct Bar;","symbolSignature":"struct Bar","matchedLines":[2,3],"matchedKeywords":["Bar"]}"#;
    let reserialized = serde_json::to_string(&populated).unwrap();
    assert_eq!(reserialized, expected);
}

#[test]
fn code_block_missing_required_field_errors() {
    let err_file = serde_json::from_str::<CodeBlock>(
        r#"{"lineStart":1,"lineEnd":5,"nodeKind":"function","code":"fn foo() {}"}"#,
    )
    .unwrap_err();
    assert!(err_file.to_string().contains("missing field `file`"));

    let err_line_start = serde_json::from_str::<CodeBlock>(
        r#"{"file":"src/foo.rs","lineEnd":5,"nodeKind":"function","code":"fn foo() {}"}"#,
    )
    .unwrap_err();
    assert!(err_line_start.to_string().contains("missing field `lineStart`"));

    let err_line_end = serde_json::from_str::<CodeBlock>(
        r#"{"file":"src/foo.rs","lineStart":1,"nodeKind":"function","code":"fn foo() {}"}"#,
    )
    .unwrap_err();
    assert!(err_line_end.to_string().contains("missing field `lineEnd`"));

    let err_node_kind = serde_json::from_str::<CodeBlock>(
        r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":5,"code":"fn foo() {}"}"#,
    )
    .unwrap_err();
    assert!(err_node_kind.to_string().contains("missing field `nodeKind`"));

    let err_code = serde_json::from_str::<CodeBlock>(
        r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":5,"nodeKind":"function"}"#,
    )
    .unwrap_err();
    assert!(err_code.to_string().contains("missing field `code`"));
}

#[test]
fn code_block_unknown_node_kind_errors() {
    let json = r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":5,"nodeKind":"bogus","code":"fn foo() {}"}"#;
    let err = serde_json::from_str::<CodeBlock>(json).unwrap_err();
    assert!(err.to_string().contains("unknown variant"));
}

#[test]
fn code_block_extra_unknown_key_silently_discarded() {
    let json = r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":5,"nodeKind":"function","code":"fn foo() {}","extraField":"x"}"#;
    let block: CodeBlock = serde_json::from_str(json).unwrap();
    assert_eq!(block.file.as_str(), "src/foo.rs");
}

#[test]
fn code_block_negative_integer_fields_error() {
    let err_line_start = serde_json::from_str::<CodeBlock>(
        r#"{"file":"src/foo.rs","lineStart":-1,"lineEnd":5,"nodeKind":"function","code":"fn foo() {}"}"#,
    )
    .unwrap_err();
    assert!(err_line_start.to_string().contains("invalid value"));

    let err_line_end = serde_json::from_str::<CodeBlock>(
        r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":-5,"nodeKind":"function","code":"fn foo() {}"}"#,
    )
    .unwrap_err();
    assert!(err_line_end.to_string().contains("invalid value"));

    let err_matched_lines = serde_json::from_str::<CodeBlock>(
        r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":5,"nodeKind":"function","code":"fn foo() {}","matchedLines":[1,-2]}"#,
    )
    .unwrap_err();
    assert!(err_matched_lines.to_string().contains("invalid value"));
}

#[test]
fn code_block_unenforced_invariants_deserialize_successfully() {
    let json = r#"{"file":"src/foo.rs","lineStart":100,"lineEnd":5,"nodeKind":"function","code":"fn foo() {}"}"#;
    let block: CodeBlock = serde_json::from_str(json).unwrap();
    assert_eq!(block.line_start, 100);
    assert_eq!(block.line_end, 5);
    assert!(block.line_start > block.line_end);

    let json2 = r#"{"file":"src/foo.rs","lineStart":1,"lineEnd":5,"nodeKind":"function","code":"fn foo() {}","matchedLines":[1,999999]}"#;
    let block2: CodeBlock = serde_json::from_str(json2).unwrap();
    assert_eq!(block2.matched_lines, vec![1, 999999]);
    assert!(block2.matched_lines.iter().any(|&l| l > block2.line_end));
}

use crate::types::ParentContext;

#[test]
fn parent_context_serialize_shape() {
    let ctx = ParentContext { kind: SymbolKind::Class, name: "Foo".into(), line: 3 };
    let v = serde_json::to_value(&ctx).unwrap();
    let obj = v.as_object().unwrap();
    assert_eq!(obj.get("kind"), Some(&serde_json::Value::String("class".into())));
    assert_eq!(obj.get("name"), Some(&serde_json::Value::String("Foo".into())));
    assert_eq!(obj.get("line"), Some(&serde_json::Value::Number(3.into())));
    assert_eq!(obj.len(), 3);

    let reserialized = serde_json::to_string(&ctx).unwrap();
    assert_eq!(reserialized, r#"{"kind":"class","name":"Foo","line":3}"#);
}

use crate::types::RankedBlock;

#[test]
fn ranked_block_flatten_key_shape_no_collision_and_parent_context_null() {
    let block = CodeBlock {
        file: "src/foo.rs".into(),
        line_start: 1,
        line_end: 5,
        node_kind: SymbolKind::Function,
        code: "fn foo() {}".into(),
        symbol_signature: Some("fn foo()".into()),
        matched_lines: vec![2],
        matched_keywords: vec!["foo".into()],
    };
    let ranked = RankedBlock {
        block,
        bm25_score: 1.5,
        coverage_boost: 0.5,
        node_type_boost: 0.25,
        final_score: 2.25,
        rank: 1,
        parent_context: None,
    };
    let v = serde_json::to_value(&ranked).unwrap();
    let obj = v.as_object().unwrap();

    assert!(!obj.contains_key("block"));

    let code_block_keys: std::collections::BTreeSet<&str> = [
        "file", "lineStart", "lineEnd", "nodeKind", "code", "symbolSignature", "matchedLines", "matchedKeywords",
    ]
    .into_iter()
    .collect();
    let ranked_block_keys: std::collections::BTreeSet<&str> =
        ["bm25Score", "coverageBoost", "nodeTypeBoost", "finalScore", "rank", "parentContext"].into_iter().collect();
    assert!(code_block_keys.is_disjoint(&ranked_block_keys));

    let actual_keys: std::collections::BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    let expected_keys: std::collections::BTreeSet<&str> = code_block_keys.union(&ranked_block_keys).copied().collect();
    assert_eq!(actual_keys, expected_keys);
    assert_eq!(obj.len(), 14);

    assert!(obj.contains_key("parentContext"));
    assert_eq!(obj.get("parentContext"), Some(&serde_json::Value::Null));
}

/// Pins RankedBlock's 7-field declaration order (AC-005) the same way
/// code_block_symbol_signature_null_present_and_matched_vecs_skip_when_empty,
/// parent_context_serialize_shape, and
/// diagnostic_full_round_trip_exact_shape pin CodeBlock/ParentContext/
/// Diagnostic: an exact `to_string` assertion, not merely a key-set
/// comparison (which the flatten collision test above already does but
/// cannot distinguish reordered fields, since BTreeSet is order-insensitive).
/// matched_lines/matched_keywords are populated (non-empty) so all 8
/// CodeBlock keys plus RankedBlock's own 6 keys -- 14 total -- appear in
/// the output; an empty matched_lines/matched_keywords would omit 2 of
/// them via skip_serializing_if (AC-007) and leave those two fields'
/// relative position unpinned.
#[test]
fn ranked_block_declaration_order_pinned_via_exact_serialize() {
    let block = CodeBlock {
        file: "src/bar.rs".into(),
        line_start: 2,
        line_end: 9,
        node_kind: SymbolKind::Struct,
        code: "struct Bar;".into(),
        symbol_signature: Some("struct Bar".into()),
        matched_lines: vec![2, 3],
        matched_keywords: vec!["Bar".into()],
    };
    let ranked = RankedBlock {
        block,
        bm25_score: 1.5,
        coverage_boost: 0.5,
        node_type_boost: 0.25,
        final_score: 2.25,
        rank: 1,
        parent_context: None,
    };
    let expected = r#"{"file":"src/bar.rs","lineStart":2,"lineEnd":9,"nodeKind":"struct","code":"struct Bar;","symbolSignature":"struct Bar","matchedLines":[2,3],"matchedKeywords":["Bar"],"bm25Score":1.5,"coverageBoost":0.5,"nodeTypeBoost":0.25,"finalScore":2.25,"rank":1,"parentContext":null}"#;
    let reserialized = serde_json::to_string(&ranked).unwrap();
    assert_eq!(reserialized, expected);
}

#[test]
fn ranked_block_flatten_preserves_skip_serializing_if() {
    let block = CodeBlock {
        file: "src/bar.rs".into(),
        line_start: 1,
        line_end: 2,
        node_kind: SymbolKind::Struct,
        code: "struct Bar;".into(),
        symbol_signature: None,
        matched_lines: vec![],
        matched_keywords: vec![],
    };
    let ranked = RankedBlock {
        block,
        bm25_score: 0.0,
        coverage_boost: 0.0,
        node_type_boost: 0.0,
        final_score: 0.0,
        rank: 2,
        parent_context: None,
    };
    let v = serde_json::to_value(&ranked).unwrap();
    let obj = v.as_object().unwrap();
    assert!(!obj.contains_key("matchedLines"));
    assert!(!obj.contains_key("matchedKeywords"));
    assert!(obj.contains_key("symbolSignature"));
    assert_eq!(obj.get("symbolSignature"), Some(&serde_json::Value::Null));
}


#[test]
fn ranked_block_non_finite_scores_serialize_as_null() {
    fn sample_block() -> CodeBlock {
        CodeBlock {
            file: "src/foo.rs".into(),
            line_start: 1,
            line_end: 2,
            node_kind: SymbolKind::Function,
            code: "fn foo() {}".into(),
            symbol_signature: None,
            matched_lines: vec![],
            matched_keywords: vec![],
        }
    }
    fn finite_ranked() -> RankedBlock {
        RankedBlock {
            block: sample_block(),
            bm25_score: 0.0,
            coverage_boost: 0.0,
            node_type_boost: 0.0,
            final_score: 0.0,
            rank: 0,
            parent_context: None,
        }
    }

    for non_finite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut r = finite_ranked();
        r.bm25_score = non_finite;
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v.get("bm25Score"), Some(&serde_json::Value::Null));

        let mut r = finite_ranked();
        r.coverage_boost = non_finite;
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v.get("coverageBoost"), Some(&serde_json::Value::Null));

        let mut r = finite_ranked();
        r.node_type_boost = non_finite;
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v.get("nodeTypeBoost"), Some(&serde_json::Value::Null));

        let mut r = finite_ranked();
        r.final_score = non_finite;
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v.get("finalScore"), Some(&serde_json::Value::Null));
    }
}

use crate::types::DiagnosticKind;

#[test]
fn diagnostic_kind_lowercase_forms() {
    let pairs: [(DiagnosticKind, &str); 3] = [
        (DiagnosticKind::Degraded, "\"degraded\""),
        (DiagnosticKind::Skipped, "\"skipped\""),
        (DiagnosticKind::Warning, "\"warning\""),
    ];
    for (variant, expected) in pairs {
        let s = serde_json::to_string(&variant).unwrap();
        assert_eq!(s, expected);
        let round: DiagnosticKind = serde_json::from_str(expected).unwrap();
        assert_eq!(round, variant);
    }
}

#[test]
fn diagnostic_kind_variant_count_and_order_pinned() {
    let assert_shape = |k: DiagnosticKind| match k {
        DiagnosticKind::Degraded | DiagnosticKind::Skipped | DiagnosticKind::Warning => {}
    };
    assert_shape(DiagnosticKind::Degraded);

    assert_eq!(DiagnosticKind::Degraded as usize, 0);
    assert_eq!(DiagnosticKind::Skipped as usize, 1);
    assert_eq!(DiagnosticKind::Warning as usize, 2);
}

use crate::types::Diagnostic;

#[test]
fn diagnostic_required_fields_and_path_leniency() {
    let err_kind = serde_json::from_str::<Diagnostic>(r#"{"message":"partial results"}"#).unwrap_err();
    assert!(err_kind.to_string().contains("missing field `kind`"));

    let err_message = serde_json::from_str::<Diagnostic>(r#"{"kind":"warning"}"#).unwrap_err();
    assert!(err_message.to_string().contains("missing field `message`"));

    let json_missing_path = r#"{"kind":"warning","message":"partial results"}"#;
    let parsed: Diagnostic = serde_json::from_str(json_missing_path).unwrap();
    assert_eq!(parsed.path, None);

    let json_null_path = r#"{"kind":"warning","path":null,"message":"partial results"}"#;
    let parsed2: Diagnostic = serde_json::from_str(json_null_path).unwrap();
    assert_eq!(parsed2.path, None);

    let v = serde_json::to_value(&parsed).unwrap();
    let obj = v.as_object().unwrap();
    assert!(obj.contains_key("path"));
    assert_eq!(obj.get("path"), Some(&serde_json::Value::Null));
}

#[test]
fn diagnostic_extra_unknown_key_silently_discarded() {
    let json = r#"{"kind":"warning","path":null,"message":"partial results","extraField":"x"}"#;
    let diag: Diagnostic = serde_json::from_str(json).unwrap();
    assert_eq!(diag.kind, DiagnosticKind::Warning);
    assert_eq!(diag.message, "partial results");
    assert_eq!(diag.path, None);
}

#[test]
fn diagnostic_full_round_trip_exact_shape_and_field_count() {
    let json = r#"{"kind":"degraded","path":"src/foo.rs","message":"partial results"}"#;
    let diag: Diagnostic = serde_json::from_str(json).unwrap();
    assert_eq!(diag.kind, DiagnosticKind::Degraded);
    assert_eq!(diag.path.as_ref().map(|p| p.as_str()), Some("src/foo.rs"));
    assert_eq!(diag.message, "partial results");

    let reserialized = serde_json::to_string(&diag).unwrap();
    assert_eq!(reserialized, json);

    let Diagnostic { kind, path, message } = diag;
    let _: DiagnosticKind = kind;
    let _: Option<camino::Utf8PathBuf> = path;
    let _: String = message;
}

#[test]
fn diagnostic_kind_unknown_value_errors() {
    let json = r#"{"kind":"bogus","message":"x"}"#;
    let err = serde_json::from_str::<Diagnostic>(json).unwrap_err();
    assert!(err.to_string().contains("unknown variant"));

    let err2 = serde_json::from_str::<DiagnosticKind>(r#""Degraded""#).unwrap_err();
    assert!(err2.to_string().contains("unknown variant"));
}

use crate::types::LineHit;

#[test]
fn line_hit_field_types_and_derives_pinned() {
    fn assert_debug_clone_derives<T: std::fmt::Debug + Clone>() {}
    assert_debug_clone_derives::<LineHit>();

    let hit = LineHit { line_number: 42, text: "fn foo() {}".into() };
    let LineHit { line_number, text } = hit;
    let pinned_line_number: usize = line_number;
    let pinned_text: String = text;
    assert_eq!(pinned_line_number, 42);
    assert_eq!(pinned_text, "fn foo() {}");

    // LineHit has no serde derive (AC-015), so field order is unobservable
    // on the wire and the destructure above binds by name, not position --
    // neither can pin declaration order. Reusing the include_str!-based
    // technique enum_declaration_block/assert_declaration_order already
    // established for RustPathAnchor/DependencyTarget/LangData (data-carrying
    // enums where `as usize` discriminant casts don't apply): read LineHit's
    // own source block directly from types.rs and check each field name
    // appears at a strictly increasing text position. enum_declaration_block
    // is purely a brace-matched text search keyed off the leading
    // declaration string, so it works unchanged for a `pub struct` block too.
    let src = include_str!("types.rs");
    let block = enum_declaration_block(src, "pub struct LineHit");
    assert_declaration_order(block, &["line_number", "text"]);
}

/// Exhaustive compile-time field-type and derive-list pinning for every
/// SPEC-006 type, mirroring `spec_005_all_field_types_pinned`'s rationale:
/// serde treats several distinct Rust types identically on the wire
/// (`String` vs `camino::Utf8PathBuf`, `f64` vs another float width,
/// `usize` vs another integer width), so only a compile-time `let x:
/// ExactType = value.field` binding -- not a runtime assertion -- can
/// distinguish the declared field type from a serde-equivalent sibling.
/// This test pins every field independently of whatever incidental typed
/// bindings earlier tests in this file may already exercise (deliberately
/// not deduplicated against them), so this test alone is a complete,
/// self-contained inventory:
///
/// - CodeBlock: file: Utf8PathBuf, line_start: usize, line_end: usize,
///   node_kind: SymbolKind, code: String, symbol_signature:
///   Option<String>, matched_lines: Vec<usize>, matched_keywords:
///   Vec<String> -- 8 fields, all pinned below.
/// - ParentContext: kind: SymbolKind, name: String, line: usize -- 3
///   fields, all pinned below.
/// - RankedBlock: block: CodeBlock, bm25_score: f64, coverage_boost: f64,
///   node_type_boost: f64, final_score: f64, rank: usize, parent_context:
///   Option<ParentContext> -- 7 fields, all pinned below.
/// - Diagnostic: kind: DiagnosticKind, path: Option<Utf8PathBuf>, message:
///   String -- 3 fields, all pinned below.
/// - DiagnosticKind: fieldless (3 variants) -- nothing to pin.
/// - LineHit: line_number: usize, text: String -- 2 fields, already
///   pinned in `line_hit_field_types_and_derives_pinned`; not repeated
///   here since that test is this file's dedicated LineHit coverage.
#[test]
fn spec_006_all_field_types_and_derives_pinned() {
    fn assert_full_derives<T: std::fmt::Debug + Clone + serde::Serialize + serde::de::DeserializeOwned>() {}
    fn assert_serialize_only_derives<T: std::fmt::Debug + Clone + serde::Serialize>() {}
    fn assert_enum_derives<T: std::fmt::Debug + Clone + Copy + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned>() {}

    assert_full_derives::<CodeBlock>();
    assert_serialize_only_derives::<ParentContext>();
    assert_serialize_only_derives::<RankedBlock>();
    assert_full_derives::<Diagnostic>();
    assert_enum_derives::<DiagnosticKind>();

    // -- CodeBlock { file, line_start, line_end, node_kind, code,
    //    symbol_signature, matched_lines, matched_keywords } --
    let block = CodeBlock {
        file: "src/foo.rs".into(),
        line_start: 1,
        line_end: 5,
        node_kind: SymbolKind::Function,
        code: "fn foo() {}".into(),
        symbol_signature: Some("fn foo()".into()),
        matched_lines: vec![2],
        matched_keywords: vec!["foo".into()],
    };
    let CodeBlock { file, line_start, line_end, node_kind, code, symbol_signature, matched_lines, matched_keywords } =
        block.clone();
    let pinned_file: camino::Utf8PathBuf = file;
    let pinned_line_start: usize = line_start;
    let pinned_line_end: usize = line_end;
    let pinned_node_kind: SymbolKind = node_kind;
    let pinned_code: String = code;
    let pinned_symbol_signature: Option<String> = symbol_signature;
    let pinned_matched_lines: Vec<usize> = matched_lines;
    let pinned_matched_keywords: Vec<String> = matched_keywords;
    assert_eq!(pinned_file.as_str(), "src/foo.rs");
    assert_eq!(pinned_line_start, 1);
    assert_eq!(pinned_line_end, 5);
    assert!(matches!(pinned_node_kind, SymbolKind::Function));
    assert_eq!(pinned_code, "fn foo() {}");
    assert_eq!(pinned_symbol_signature, Some("fn foo()".to_string()));
    assert_eq!(pinned_matched_lines, vec![2]);
    assert_eq!(pinned_matched_keywords, vec!["foo".to_string()]);

    // -- ParentContext { kind, name, line } --
    let ctx = ParentContext { kind: SymbolKind::Class, name: "Foo".into(), line: 3 };
    let ParentContext { kind, name, line } = ctx;
    let pinned_kind: SymbolKind = kind;
    let pinned_name: String = name;
    let pinned_line: usize = line;
    assert!(matches!(pinned_kind, SymbolKind::Class));
    assert_eq!(pinned_name, "Foo");
    assert_eq!(pinned_line, 3);

    // -- RankedBlock { block, bm25_score, coverage_boost, node_type_boost,
    //    final_score, rank, parent_context } --
    let ranked = RankedBlock {
        block: block.clone(),
        bm25_score: 1.0,
        coverage_boost: 2.0,
        node_type_boost: 3.0,
        final_score: 4.0,
        rank: 5,
        parent_context: Some(ParentContext { kind: SymbolKind::Module, name: "m".into(), line: 1 }),
    };
    let RankedBlock { block: inner_block, bm25_score, coverage_boost, node_type_boost, final_score, rank, parent_context } =
        ranked;
    let pinned_inner_block: CodeBlock = inner_block;
    let pinned_bm25_score: f64 = bm25_score;
    let pinned_coverage_boost: f64 = coverage_boost;
    let pinned_node_type_boost: f64 = node_type_boost;
    let pinned_final_score: f64 = final_score;
    let pinned_rank: usize = rank;
    let pinned_parent_context: Option<ParentContext> = parent_context;
    assert_eq!(pinned_inner_block.code, "fn foo() {}");
    assert_eq!(pinned_bm25_score, 1.0);
    assert_eq!(pinned_coverage_boost, 2.0);
    assert_eq!(pinned_node_type_boost, 3.0);
    assert_eq!(pinned_final_score, 4.0);
    assert_eq!(pinned_rank, 5);
    assert!(pinned_parent_context.is_some());

    // -- Diagnostic { kind, path, message } --
    let diag = Diagnostic { kind: DiagnosticKind::Skipped, path: Some("src/foo.rs".into()), message: "m".into() };
    let Diagnostic { kind, path, message } = diag;
    let pinned_diag_kind: DiagnosticKind = kind;
    let pinned_diag_path: Option<camino::Utf8PathBuf> = path;
    let pinned_diag_message: String = message;
    assert!(matches!(pinned_diag_kind, DiagnosticKind::Skipped));
    assert_eq!(pinned_diag_path.as_ref().map(|p| p.as_str()), Some("src/foo.rs"));
    assert_eq!(pinned_diag_message, "m");
}

struct SerProbe<T>(std::marker::PhantomData<T>);
trait NotSer { const IS: bool = false; }
impl<T> NotSer for SerProbe<T> {}
impl<T: serde::Serialize> SerProbe<T> { const IS: bool = true; }

struct DeProbe<T>(std::marker::PhantomData<T>);
trait NotDe { const IS: bool = false; }
impl<T> NotDe for DeProbe<T> {}
impl<'de, T: serde::de::DeserializeOwned> DeProbe<T> { const IS: bool = true; }

#[test]
fn serde_asymmetric_types_absence_pinned() {
    // Negative claims (the actual point of this test):
    assert!(!SerProbe::<LineHit>::IS);
    assert!(!DeProbe::<LineHit>::IS);
    assert!(!DeProbe::<RankedBlock>::IS);
    assert!(!DeProbe::<ParentContext>::IS);
    // Positive controls (prove the probe itself isn't just always false):
    assert!(SerProbe::<RankedBlock>::IS);
    assert!(DeProbe::<CodeBlock>::IS);
    assert!(DeProbe::<Diagnostic>::IS);
}

