use crate::types::{Cs, Cv, ExchangeGroup, Metadata, Package, Interaction, Substance};
use roxmltree::{Document, Node};

/// Parses the content string into a roxmltree::Document
pub fn document(content: &String) -> Document {
    roxmltree::Document::parse(&content[0..]).unwrap()
}

/// Extract a Coded Simple Value from xml
pub fn cs(node: &Node, tag: &str) -> (String, String) {
    let mut v = String::new();
    let mut dn = String::new();

    for n in node.children() {
        if n.has_tag_name(tag) {
            if let Some(val) = n.attribute("V") {
                v.push_str(val);
            }

            if let Some(val) = n.attribute("DN") {
                dn.push_str(val);
            }
        }
    }

    (v, dn)
}

/// Extract a Coded Value from xml
pub fn cv(node: &Node, tag: &str) -> (String, String, String) {
    let mut v = String::new();
    let mut s = String::new();
    let mut dn = String::new();

    for n in node.children() {
        if n.has_tag_name(tag) {
            if let Some(val) = n.attribute("V") {
                v.push_str(val);
            }

            if let Some(val) = n.attribute("S") {
                s.push_str(val);
            }

            if let Some(val) = n.attribute("DN") {
                dn.push_str(val);
            }
        }
    }
    (v, s, dn)
}

/// Extract a single value from a node
pub fn string_value(node: &Node, tag: &str) -> String {
    let mut result = String::new();

    for n in node.children() {
            if n.has_tag_name(tag) {
                if let Some(val) = n.text() {
                    result.push_str(val);
                }
            }
    }
    result
}

/// Extracts the <HentetDato></HentetDato> from the xml file
///
/// # Example
///
/// ```
/// use festlib::Fest;
///
/// let fest = Fest::new("fest251.xml").unwrap();
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

    let node = move_node_forward(&node, "Interaksjon");

    match node {
        Some(node) => {
    let id = string_value(&node, "Id");
    let relevance = Cs::new(&node, "Relevans");
    let consequence = string_value(&node, "KliniskKonsekvens");
    let mechanism = string_value(&node, "Interaksjonsmekanisme");
    let basis = Cs::new(&node, "Kildegrunnlag");
    let handling = string_value(&node, "Handtering");

    // get all the subtances
    let mut substances = Vec::new();

    for x in node.children() {
        if x.has_tag_name("Substansgruppe") {
            for s in x.children() {
                if s.has_tag_name("Substans") {
                    let name = string_value(&s, "Substans");
                    let atc = Cv::new(&s, "Atc");
                    substances.push(Substance::new(name, atc));
                }
            }
        }
    }

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
        },
        None => None,
    }
}

fn move_node_forward<'a>(node: &'a Node, destination: &'a str) -> Option<Node<'a, 'a>>{
    let mut new_node = None;

    // lets move the node into <Legemiddelpakning>
    for x in node.children() {
        if x.has_tag_name(destination) {
            new_node = Some(x.clone());
            break;
        }
    }
    new_node
}

/// Retrives the xml data from <OppfLegemiddelpakning>
pub fn package(node: &Node) -> Option<Package> {
    let metadata = Metadata::new(node);

    let node = move_node_forward(&node, "Legemiddelpakning");

    match node {
        Some(node) => {
            let atc = Cv::new(&node, "Atc");
            let name = string_value(&node, "NavnFormStyrke");
            let group = Cs::new(&node, "Reseptgruppe");
            let id = string_value(&node, "Id");
            let itemnum = string_value(&node, "Varenr");
            let ean = string_value(&node, "Ean");
            let exchange_group = exchange_group(&node);

            Package::from(
                metadata,
                atc,
                name,
                group,
                id,
                itemnum,
                ean,
                exchange_group,
            )
    },
        None => None,
    }
}

/// Retrieves all the packages (OppfLegemiddelpakning) from the xml file
pub fn packages(document: &Document) -> Vec<Package> {
    let mut result = Vec::new();
    let node = document.root_element();

    for n in node.children() {
        if n.has_tag_name("KatLegemiddelpakning") {

            for x in n.children() {
                if x.has_tag_name("OppfLegemiddelpakning") {
                    if let Some(p) = package(&x) {
                        result.push(p);
                    }
                }
            }
        }
    }
    result
}

/// Retreives all the interactions (OppfInteraksjon) from the xml file
pub fn interactions(document: &Document) -> Vec<Interaction> {
    let mut result = Vec::new();

    let node = document.root_element();

    for n in node.children() {
        if n.has_tag_name("KatInteraksjon") {

            for x in n.children() {
                if x.has_tag_name("OppfInteraksjon") {
                    if let Some(i) = interaction(&x) {
                        result.push(i);
                    }
                }
            }
        }
    }
    result
}

/// Retrieves the Exchange group. <PakningByttegruppe>
pub fn exchange_group(node: &Node) -> Option<ExchangeGroup> {
    let mut id = String::new();

    for n in node.children() {
        if n.has_tag_name("PakningByttegruppe") {
            id = string_value(&n, "RefByttegruppe");
        }
    }

    if id.len() > 0 {
        ExchangeGroup::from(
            id,
            None,
            None,
        )
        } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::NodeId;
    use std::fs;

    // get the file content
    fn file_content() -> String {
        fs::read_to_string("fest251.xml").unwrap()
    }

    #[test]
    fn test_document() {
        let content = file_content();

        let document = document(&content);

        assert_eq!(document.root_element().has_tag_name("FEST"), true);
    }

    #[test]
    fn test_metadata() {
        let content = file_content();
        let document = document(&content);

        let node = document.get_node(NodeId::new(701764)).unwrap();

        let (res1, res2) = metadata(&node);

        assert_eq!(res1, "ID_F994748F-3A21-4FC3-9964-DBE097924A75");
        assert_eq!(res2, "2024-04-21T00:51:31");
    }

    #[test]
    fn test_cs() {
        let content = file_content();
        let document = document(&content);

        let node = document.get_node(NodeId::new(701764)).unwrap();

        let (res1, _) = cs(&node, "Status");

        assert_eq!(res1, "A");
    }

    #[test]
    fn test_cv() {
        let content = file_content();
        let document = document(&content);

        let node = document.get_node(NodeId::new(701764)).unwrap();

        let mut move_forward = None;

        // lets move the node into <Legemiddelpakning>
        for x in node.children() {
            if x.has_tag_name("Legemiddelpakning") {
                move_forward = Some(x.clone());
                break;
            }
        }

        let (res1, res2, res3) = cv(&move_forward.unwrap(), "LegemiddelformKort");

        assert_eq!(res1, "32");
        assert_eq!(res2, "2.16.578.1.12.4.1.1.7448");
        assert_eq!(res3, "Kapsel");
    }

    #[test]
    fn test_string_value() {
        let content = file_content();
        let document = document(&content);

        let node = document.get_node(NodeId::new(701764)).unwrap();

        let id = string_value(&node, "Id");
        assert_eq!(id, "ID_F994748F-3A21-4FC3-9964-DBE097924A75");
    }

    #[test]
    fn test_package() {
        let content = file_content();
        let document = document(&content);

        let node = document.get_node(NodeId::new(701764)).unwrap();

        let package = package(&node);
        assert_eq!(package.as_ref().unwrap().id(), "ID_0138BA04-7B67-4FB5-B44D-7491336CAF20");
        assert_eq!(package.unwrap().itemnum(), "953335");
    }

    #[test]
    fn test_packages() {
        let content = file_content();
        let document = document(&content);

        let packages = packages(&document);
        assert_eq!(packages.len(), 10473);
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
