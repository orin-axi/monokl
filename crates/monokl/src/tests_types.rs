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

