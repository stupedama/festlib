use festlib::Fest;

#[test]
fn interaction_test() {
    let fest = Fest::new("fest251.xml").expect("Could not open xml file");

    // First find the package we want to find generic products for
    let package1 = fest.find_package("061561");
    let package2 = fest.find_package("017701");

    assert!(package1.is_some());
    assert!(package2.is_some());
    let mut packages = Vec::new();

    if let Some(p) = package1 {
        packages.push(p);
    }

    if let Some(p) = package2 {
        packages.push(p);
    }

    let interaction = fest.find_interaction(&packages);
    assert_eq!(interaction.unwrap().len(), 1);
}
