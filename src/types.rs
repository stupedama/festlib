use roxmltree::Node;
use crate::xml;

// TODO: remove #[allow(dead_code)] and implement all the missing parts
// of the structs that have this attribute.

/// Holds the last date for when the fest was last
/// updated. (HentetDato).
pub struct LastUpdate {
    update: String,
}

impl LastUpdate {
    pub fn new(date: &str) -> Self {
        LastUpdate {
            update: String::from(date),
        }
    }

    pub fn date(&self) -> &String {
        &self.update
    }

}

/// Holds the id reference for generic packages/drugs
#[allow(dead_code)]
#[derive(Debug)]
pub struct ExchangeGroup {
    id: String,
    valid_from: Option<String>,
    valid_to: Option<String>,
}

impl ExchangeGroup {
    pub fn new(node: &Node) -> Option<Self> {
        xml::exchange_group(&node)
    }

    pub fn from(id: String, valid_from: Option<String>, valid_to: Option<String>) -> Option<Self> {
        Some(ExchangeGroup {
            id,
            valid_from,
            valid_to,
        })
    }

    pub fn id(self) -> String {
        self.id
    }
}

/// Coded Simple Value
/// Gives a codes value with a String with an option
/// to give the 'v' a meaning 'dn'
#[allow(dead_code)]
#[derive(Debug)]
pub struct Cs {
    v: String,
    dn: String,
}

impl Cs {
    pub fn new(node: &Node, tag: &str) -> Self {
        let (v, dn) = xml::cs(node, tag);

        Cs {
            v,
            dn,
        }
    }
}

/// Coded Value with a OID (object identifier)
/// s = oid.
/// the oid have a constant value but the last part
/// is the identifier
#[allow(dead_code)]
#[derive(Debug)]
pub struct Cv {
    v: String,
    s: String,
    dn: String,
}

impl Cv {
    pub fn new(node: &Node, tag: &str) -> Self {
        let (v, s, dn) = xml::cv(node, tag);

        Cv {
            v,
            s,
            dn,
        }
    }

    pub fn v(&self) -> &String {
        &self.v
    }
}

/// Holds the metadata of the xml entry
#[allow(dead_code)]
#[derive(Debug)]
pub struct Metadata {
    id: String,
    time: String,
    status: Cs,
}

impl Metadata {
    pub fn new(node: &Node) -> Self {
        let (id, time) = xml::metadata(node);
        let status = Cs::new(&node, "Status");

        Metadata {
            id,
            time,
            status,
        }
    }
}

/// Holds the information about the drug package (Legemiddelpakning).
#[allow(dead_code)]
#[derive(Debug)]
pub struct Package {
    metadata: Metadata,
    atc: Cv,
    name: String,
    group: Cs,
    id: String,
    itemnum: String,
    ean: String,
    exchange_group: Option<ExchangeGroup>,
}

impl Package {
    pub fn from(
        metadata: Metadata,
        atc: Cv,
        name: String,
        group: Cs,
        id: String,
        itemnum: String,
        ean: String,
        exchange_group: Option<ExchangeGroup>) -> Option<Self> {
        Some(Package {
            metadata, atc, name, group, id, itemnum, ean, exchange_group
        })
    }

    pub fn new(node: &Node) -> Option<Self> {
        xml::package(&node)
    }

    /// Returns the EAN code for the package
    pub fn ean(&self) -> &String {
        &self.ean
    }

    /// Returns the itemnumber (varenr) for the package
    pub fn itemnum(&self) -> &String {
        &self.itemnum
    }

    /// Return the ATC (Anatomical Therapeutic Chemical)
    /// code for the package
    pub fn atc(&self) -> &Cv {
        &self.atc
    }

    /// Returns the unique id of the entry
    pub fn id(&self) -> &String {
        &self.id
    }

    /// Returns the exchange id reference for generic
    /// products. Returns None if there is no id
    /// TODO: or the id is not longer valid
    pub fn exchange_id(&self) -> Option<&String> {
        match &self.exchange_group {
            Some(e) => Some(&e.id),
            None => None,
        }
    }
}


/// Part of Interaction. Is the Substance
/// that interacts with other substances
#[derive(Debug)]
pub struct Substance {
    name: String,
    atc: Cv,
}

impl Substance {
    pub fn new(name: String, atc: Cv) -> Self {
        Substance {
            name,
            atc,
        }
    }

    /// drug name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Atc code the interaction applies to
    pub fn atc(&self) -> &String {
        &self.atc.v
    }
}

/// Holds the information about an Interaction between two or more 
/// Packages (substances).
#[allow(dead_code)]
#[derive(Debug)]
pub struct Interaction {
    metadata: Metadata,
    id: String,
    relevance: Cs,
    consequence: String,
    mechanism: String,
    basis: Cs,
    handling: String,
    //Visningsregler: <Vec<Cv>,
    //references: Vec<Reference>,
    substances: Vec<Substance>,
}

impl Interaction {
    pub fn new(metadata: Metadata, id: String,
        relevance: Cs, consequence: String,
        mechanism: String, basis: Cs, handling: String,
        substances: Vec<Substance>) -> Self {
        Interaction {
            metadata, id, relevance, consequence,
            mechanism, basis, handling, substances
        }
    }

    /// Substances the interaction applies to
    pub fn substances(&self) -> &Vec<Substance> {
        &self.substances
    }

    /// Unique entry id
    pub fn id(&self) -> &String {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Fest;

    // Helper function to find first OppfLegemiddelpakning node
    fn find_first_package_node<'a>(doc: &'a roxmltree::Document) -> Option<roxmltree::Node<'a, 'a>> {
        doc.root_element()
            .children()
            .find(|n| n.has_tag_name("KatLegemiddelpakning"))?
            .children()
            .find(|n| n.has_tag_name("OppfLegemiddelpakning"))
    }

    #[test]
    fn test_metadata() {
        let fest = Fest::new("test_fest.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        if let Some(node) = find_first_package_node(&content) {
            let metadata = Metadata::new(&node);

            assert_eq!(metadata.id, "ID_F994748F-3A21-4FC3-9964-DBE097924A75");
            assert_eq!(metadata.time, "2024-04-21T00:51:31");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_cs() {
        let fest = Fest::new("test_fest.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        if let Some(node) = find_first_package_node(&content) {
            let metadata = Metadata::new(&node);
            let cs = metadata.status;
            assert_eq!(cs.v, "A");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_cv() {
        let fest = Fest::new("test_fest.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        if let Some(node) = find_first_package_node(&content) {
            // Navigate to the Legemiddelpakning child
            for child in node.children() {
                if child.has_tag_name("Legemiddelpakning") {
                    let cv = Cv::new(&child, "LegemiddelformKort");
                    assert_eq!(cv.v, "32");
                    assert_eq!(cv.s, "2.16.578.1.12.4.1.1.7448");
                    assert_eq!(cv.dn, "Kapsel");
                    return;
                }
            }
            panic!("Could not find Legemiddelpakning node");
        } else {
            panic!("Could not find package node");
        }
    }

    #[test]
    fn test_package() {
        let fest = Fest::new("test_fest.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        if let Some(node) = find_first_package_node(&content) {
            let package = Package::new(&node).unwrap();

            assert_eq!(package.id, "ID_0138BA04-7B67-4FB5-B44D-7491336CAF20");
            assert_eq!(package.itemnum, "061561");
            assert_eq!(package.ean, "7001234567890");
        } else {
            panic!("Could not find package node");
        }
    }
}
