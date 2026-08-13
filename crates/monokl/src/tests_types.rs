use crate::types::{SymbolKind, Visibility};

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
