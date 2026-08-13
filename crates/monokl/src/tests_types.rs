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

