use festlib::Fest;

#[test]
fn test_generic() {
    let fest = Fest::new("fest251.xml").expect("Could not open xml file");

    // First find the package we want to find generic products for
    let package = fest.find_package("061561");

    if let Some(p) = package {
        let generic = fest.find_generic(&p);
        assert!(generic.is_some());
        assert_eq!(generic.unwrap().len(), 5);
    }

    assert!(package.is_some());
}
