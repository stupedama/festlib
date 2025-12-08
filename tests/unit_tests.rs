use std::{env, path::PathBuf};

#[cfg(test)]
mod tests {
    use super::*;
    use festlib::Fest;

    // Helper function to get the test file path relative to cargo project root
    fn get_test_file_path() -> String {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let mut path = PathBuf::from(manifest_dir);
        path.push("test_fest.xml");
        path.to_string_lossy().to_string()
    }

    #[test]
    fn test_fest_creation() {
        // Test creating Fest from the stable test file
        let test_file = get_test_file_path();
        let result = Fest::new(&test_file);
        assert!(result.is_ok(), "Should be able to create Fest from valid XML file");
    }

    #[test]
    fn test_fest_delivery_date() {
        let test_file = get_test_file_path();
        if let Ok(fest) = Fest::new(&test_file) {
            let date = fest.delivery_date();
            assert_eq!(date.date(), "2024-09-09T14:21:28");
        } else {
            panic!("Failed to create Fest instance");
        }
    }

    #[test]
    fn test_fest_packages() {
        let test_file = get_test_file_path();
        if let Ok(fest) = Fest::new(&test_file) {
            let packages = fest.packages();
            assert_eq!(packages.len(), 5);
            
            // Test first package properties
            let package = &packages[0];
            assert_eq!(package.id(), "ID_0138BA04-7B67-4FB5-B44D-7491336CAF20");
            assert_eq!(package.itemnum(), "061561");
            assert_eq!(package.ean(), "7001234567890");
        } else {
            panic!("Failed to create Fest instance");
        }
    }

    #[test]
    fn test_find_package_by_itemnum() {
        let test_file = get_test_file_path();
        if let Ok(fest) = Fest::new(&test_file) {
            let package = fest.find_package("061561");
            assert!(package.is_some(), "Should find package with itemnum 061561");
            
            if let Some(p) = package {
                assert_eq!(p.id(), "ID_0138BA04-7B67-4FB5-B44D-7491336CAF20");
                assert_eq!(p.ean(), "7001234567890");
            }
        } else {
            panic!("Failed to create Fest instance");
        }
    }

    #[test]
    fn test_find_nonexistent_package() {
        let test_file = get_test_file_path();
        if let Ok(fest) = Fest::new(&test_file) {
            let package = fest.find_package("999999");
            assert!(package.is_none(), "Should not find package with non-existent itemnum");
        } else {
            panic!("Failed to create Fest instance");
        }
    }

    #[test] 
    fn test_invalid_file() {
        // Test with non-existent file
        let result = Fest::new("non_existent_file.xml");
        assert!(result.is_err(), "Should fail when file doesn't exist");
    }

    #[test]
    fn test_find_generic() {
        let test_file = get_test_file_path();
        if let Ok(fest) = Fest::new(&test_file) {
            if let Some(package) = fest.find_package("061561") {
                let generics = fest.find_generic(&package);
                assert!(generics.is_some());
                assert_eq!(generics.unwrap().len(), 4); // Should find 4 generics with same exchange group
            } else {
                panic!("Could not find package for generic test");
            }
        } else {
            panic!("Failed to create Fest instance");
        }
    }

    #[test]
    fn test_interactions() {
        let test_file = get_test_file_path();
        if let Ok(fest) = Fest::new(&test_file) {
            let package1 = fest.find_package("061561");
            let package2 = fest.find_package("017701");
            
            assert!(package1.is_some());
            assert!(package2.is_some());
            
            let packages = vec![package1.unwrap(), package2.unwrap()];
            let interactions = fest.find_interaction(&packages);
            
            // Our test XML has 1 interaction between these ATCs
            assert!(interactions.is_some());
            assert_eq!(interactions.unwrap().len(), 1);
        } else {
            panic!("Failed to create Fest instance");
        }
    }
}