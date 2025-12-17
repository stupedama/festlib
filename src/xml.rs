use crate::types::{Cs, Cv, ExchangeGroup, Metadata, Package, Interaction, Substance};
use roxmltree::{Document, Node};

/// Parses the content string into a roxmltree::Document
pub fn document(content: &str) -> Document<'_> {
    // panic if invalid xml
    roxmltree::Document::parse(&content[0..]).unwrap()   
}

/// Extract a Coded Simple Value from xml
pub fn cs(node: &Node, tag: &str) -> (String, String) {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .map(|n| {
            let v = n.attribute("V").unwrap_or("").to_string();
            let dn = n.attribute("DN").unwrap_or("").to_string();
            (v, dn)
        })
        .unwrap_or_default()
}

/// Extract a Coded Value from xml
pub fn cv(node: &Node, tag: &str) -> (String, String, String) {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .map(|n| {
            let v = n.attribute("V").unwrap_or("").to_string();
            let s = n.attribute("S").unwrap_or("").to_string();
            let dn = n.attribute("DN").unwrap_or("").to_string();
            (v, s, dn)
        })
        .unwrap_or_default()
}

/// Extract a single value from a node
pub fn string_value(node: &Node, tag: &str) -> String {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string()
}

/// Extracts the <HentetDato></HentetDato> from the xml file
///
/// # Example
///
/// ```
/// use festlib::Fest;
///
/// let fest = Fest::new("test_fest.xml").unwrap();
/// let date = fest.delivery_date();
///
/// assert_eq!("2024-09-09T14:21:28", date.date());
/// ```
pub fn delivery_date(content: &String) -> String {
    string_value(&document(content).root_element(), "HentetDato")
}

/// Retreives the Metadata from xml string
/// Its the <Enkeltoppforing> that contains unique id,
/// time of creation and status
pub fn metadata(node: &Node) -> (String, String) {
    let id = string_value(&node, "Id");
    let time = string_value(&node, "Tidspunkt");

    (id, time)
}

/// Retrieves the xml from <OppfInteraksjon>
pub fn interaction(node: &Node) -> Option<Interaction> {
    let metadata = Metadata::new(node);
    let node = move_node_forward(&node, "Interaksjon")?;

    let id = string_value(&node, "Id");
    let relevance = Cs::new(&node, "Relevans");
    let consequence = string_value(&node, "KliniskKonsekvens");
    let mechanism = string_value(&node, "Interaksjonsmekanisme");
    let basis = Cs::new(&node, "Kildegrunnlag");
    let handling = string_value(&node, "Handtering");

    let substances: Vec<Substance> = node
        .children()
        .filter(|x| x.has_tag_name("Substansgruppe"))
        .flat_map(|x| x.children())
        .filter(|s| s.has_tag_name("Substans"))
        .map(|s| {
            let name = string_value(&s, "Substans");
            let atc = Cv::new(&s, "Atc");
            Substance::new(name, atc)
        })
        .collect();

    Some(Interaction::new(
        metadata,
        id,
        relevance,
        consequence,
        mechanism,
        basis,
        handling,
        substances
    ))
}

fn move_node_forward<'a>(node: &'a Node, destination: &str) -> Option<Node<'a, 'a>> {
    node.children().find(|n| n.has_tag_name(destination))
}

/// Retrives the xml data from <OppfLegemiddelpakning>
pub fn package(node: &Node) -> Option<Package> {
    let metadata = Metadata::new(node);
    let node = move_node_forward(&node, "Legemiddelpakning")?;

    Package::from(
        metadata,
        Cv::new(&node, "Atc"),
        string_value(&node, "NavnFormStyrke"),
        Cs::new(&node, "Reseptgruppe"),
        string_value(&node, "Id"),
        string_value(&node, "Varenr"),
        string_value(&node, "Ean"),
        exchange_group(&node),
    )
}

/// Retrieves all the packages (OppfLegemiddelpakning) from the xml file
pub fn packages(document: &Document) -> Vec<Package> {
    document
        .root_element()
        .children()
        .find(|n| n.has_tag_name("KatLegemiddelpakning"))
        .into_iter()
        .flat_map(|n| n.children())
        .filter(|x| x.has_tag_name("OppfLegemiddelpakning"))
        .filter_map(|x| package(&x))
        .collect()
}

/// Retreives all the interactions (OppfInteraksjon) from the xml file
pub fn interactions(document: &Document) -> Vec<Interaction> {
    document
        .root_element()
        .children()
        .find(|n| n.has_tag_name("KatInteraksjon"))
        .into_iter()
        .flat_map(|n| n.children())
        .filter(|x| x.has_tag_name("OppfInteraksjon"))
        .filter_map(|x| interaction(&x))
        .collect()
}

/// Retrieves the Exchange group. <PakningByttegruppe>
pub fn exchange_group(node: &Node) -> Option<ExchangeGroup> {
    node.children()
        .find(|n| n.has_tag_name("PakningByttegruppe"))
        .map(|n| string_value(&n, "RefByttegruppe"))
        .filter(|id| !id.is_empty())
        .and_then(|id| ExchangeGroup::from(id, None, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // get the file content
    fn file_content() -> String {
        fs::read_to_string("test_fest.xml").unwrap()
    }

    // Helper function to find first OppfLegemiddelpakning node
    fn find_first_package_node<'a>(doc: &'a Document) -> Option<Node<'a, 'a>> {
        doc.root_element()
            .children()
            .find(|n| n.has_tag_name("KatLegemiddelpakning"))
            .and_then(|n| n.children().find(|c| c.has_tag_name("OppfLegemiddelpakning")))
    }

    #[test]
    fn test_document() {
        let content = file_content();
        let document = document(&content);
        assert_eq!(document.root_element().has_tag_name("FEST"), true);
    }

    #[test]
    fn test_delivery_date() {
        let content = file_content();
        let date = delivery_date(&content);
        assert_eq!(date, "2024-09-09T14:21:28");
    }

    #[test]
    fn test_metadata() {
        let content = file_content();
        let document = document(&content);

        if let Some(node) = find_first_package_node(&document) {
            let (res1, res2) = metadata(&node);
            assert_eq!(res1, "ID_F994748F-3A21-4FC3-9964-DBE097924A75");
            assert_eq!(res2, "2024-04-21T00:51:31");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_cs() {
        let content = file_content();
        let document = document(&content);

        if let Some(node) = find_first_package_node(&document) {
            let (res1, _) = cs(&node, "Status");
            assert_eq!(res1, "A");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_cv() {
        let content = file_content();
        let document = document(&content);

        if let Some(node) = find_first_package_node(&document) {
            // Navigate to the Legemiddelpakning child
            for child in node.children() {
                if child.has_tag_name("Legemiddelpakning") {
                    let (res1, res2, res3) = cv(&child, "LegemiddelformKort");
                    assert_eq!(res1, "32");
                    assert_eq!(res2, "2.16.578.1.12.4.1.1.7448");
                    assert_eq!(res3, "Kapsel");
                    return;
                }
            }
            panic!("Could not find Legemiddelpakning node");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_string_value() {
        let content = file_content();
        let document = document(&content);

        if let Some(node) = find_first_package_node(&document) {
            let id = string_value(&node, "Id");
            assert_eq!(id, "ID_F994748F-3A21-4FC3-9964-DBE097924A75");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_package() {
        let content = file_content();
        let document = document(&content);

        if let Some(node) = find_first_package_node(&document) {
            let package = package(&node);
            assert!(package.is_some());
            assert_eq!(package.as_ref().unwrap().id(), "ID_0138BA04-7B67-4FB5-B44D-7491336CAF20");
            assert_eq!(package.unwrap().itemnum(), "061561");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_packages() {
        let content = file_content();
        let document = document(&content);

        let packages = packages(&document);
        assert_eq!(packages.len(), 5);
    }

//    #[test]
//    fn test_interactions() {
//        let content = file_content();
//        let document = document(&content);
//
//        let interactions = interactions(&document);
//        assert_eq!(interactions.len(), 9793);
//    }
//
//    #[test]
//    fn test_interaction() {
//        let content = file_content();
//        let document = document(&content);
//
//        let node = document.get_node(NodeId::new(3658322)).unwrap();
//        let interaction = interaction(&node);
//
//        println!("{:?}", interaction);
//
//
//        assert_eq!(interaction.unwrap().id, "ID_028A3D4C-C908-43D8-AA07-9F8F00E6E7A3");
//    }


   // #[test]
   // fn test_exchange_group() {
   //     let content = file_content();
   //     let document = document(&content);

   //     let node = document.get_node(NodeId::new(329123)).unwrap();

   //     let group = exchange_group(&node);
   //     println!("{:?}", group);

   //     assert_eq!(group.is_some(), true);
   // }
}
