//! # festlib
//!
//! This project provides a set of functions to parse data from the fest xml file.
//! fest xml file contains data from Norwegian Medical Products Agency (DMP) for
//!
//! This project is licensed under Apache 2.0 or MIT license (see files LICENSE-Apache2.0 and LICENSE-MIT).
//!
//! # Examples
//!
//! ### Date fest file was updated
//! ```
//! use festlib::Fest;
//!
//! let fest = Fest::new("fest251.xml").unwrap();
//! let date = fest.delivery_date();
//!
//! assert_eq!(date.date(), "2024-09-09T14:21:28");
//! ```
//!
//! ### Find packages
//! ```
//! use festlib::Fest;
//!
//! let fest = Fest::new("fest251.xml").unwrap();
//! let packages = fest.packages();
//!
//! assert_eq!(packages.len(), 10473);
//! ```
//!
//! ### Find generic packages
//! ```
//! use festlib::Fest;
//! let fest = Fest::new("fest251.xml").unwrap();
//! let package = fest.find_package("061561").unwrap();
//!
//! let result = fest.find_generic(&package);
//! ```
//! # Contact
//! For questions or feedback use make a issue on our github or john.doe.hemmelig@pm.me.
//!

mod xml;

use std::fs;
use std::error::Error;
use roxmltree::Node;

/// Holds the last date for when the fest was last
/// updated. (HentetDato).
pub struct LastUpdate {
    update: String,
}

impl LastUpdate {
    fn new(date: &str) -> Self {
        LastUpdate {
            update: String::from(date),
        }
    }

    pub fn date(&self) -> &String {
        &self.update
    }

}

/// Holds the id reference for generic packages/drugs
#[derive(Debug, Clone)]
struct ExchangeGroup {
    id: String,
    valid_from: Option<String>,
    valid_to: Option<String>,
}

impl ExchangeGroup {
    fn new(node: &Node) -> Option<ExchangeGroup> {
        xml::exchange_group(&node)
    }

    pub fn id(self) -> String {
        self.id
    }
}

/// Coded Simple Value
/// Gives a codes value with a String with an option
/// to give the 'v' a meaning 'dn'
#[derive(Debug, Clone)]
struct Cs {
    v: String,
    dn: String,
}

impl Cs {
    fn new(node: &Node, tag: &str) -> Self {
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
#[derive(Debug, Clone)]
struct Cv {
    v: String,
    s: String,
    dn: String,
}

impl Cv {
    fn new(node: &Node, tag: &str) -> Self {
        let (v, s, dn) = xml::cv(node, tag);

        Cv {
            v,
            s,
            dn,
        }
    }
}

#[derive(Debug, Clone)]
struct Metadata {
    id: String,
    time: String,
    status: Cs,
}

impl Metadata {
    fn new(node: &Node) -> Self {
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
#[derive(Debug, Clone)]
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
    fn new(node: &Node) -> Option<Self> {
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


#[derive(Debug, Clone)]
struct Substance {
    name: String,
    atc: Cv,
}

impl Substance {
    fn new(name: &str, atc: Cv) -> Self {
        Substance {
            name: String::from(name),
            atc,
        }
    }

    /// drug name
    fn name(&self) -> &String {
        &self.name
    }

    /// Atc code the interaction applies to
    fn atc(&self) -> &String {
        &self.atc.v
    }
}

#[derive(Debug, Clone)]
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
    fn new(metadata: Metadata, id: String,
        relevance: Cs, consequence: String,
        mechanism: String, basis: Cs, handling: String,
        substances: Vec<Substance>) -> Self {
        Interaction {
            metadata, id, relevance, consequence,
            mechanism, basis, handling, substances
        }
    }

    /// Substances the interaction applies to
    fn substances(&self) -> &Vec<Substance> {
        &self.substances
    }
}

/// Container for the fest file
pub struct Fest {
    _filename: String,
    content: String,
    packages: Vec<Package>,
    interactions: Vec<Interaction>,
}

impl Fest {
    /// Constructor for a new instance of the fest file
    pub fn new(filename: &str) -> Result<Self, Box<dyn Error>> {
        let content = Fest::read_file(filename)?;
        let document = xml::document(&content);

        let packages = xml::packages(&document);
        let interactions = xml::interactions(&document);

        Ok(Fest {
            _filename: filename.to_string(),
            content,
            packages,
            interactions,
        })
    }

    /// Retrieve the last update for the fest xml file
    ///
    /// # example
    /// ```
    /// use festlib::Fest;
    ///
    /// let fest = Fest::new("fest251.xml").unwrap();
    /// let date = fest.delivery_date();
    ///
    /// assert_eq!(date.date(), "2024-09-09T14:21:28");
    /// ```
    pub fn delivery_date(&self) -> LastUpdate {

        LastUpdate::new(&xml::delivery_date(&self.content))
    }

    /// Retrieve all drug packages from fest. (OppfLegemiddelpakning)
    /// contains a vector with Package includes values as
    /// drug name, itemnumber, ean, prescription group etc.
    ///
    /// # Example
    ///
    /// ```
    /// use festlib::Fest;
    ///
    /// let fest = Fest::new("fest251.xml").unwrap();
    /// let packages = fest.packages();
    ///
    /// assert_eq!(packages.len(), 10473);
    /// ```
    pub fn packages(&self) -> &Vec<Package> {
        &self.packages
    }

    /// Search for a package with itemnumber
    ///
    /// # Example
    /// ```
    /// use festlib::Fest;
    /// let fest = Fest::new("fest251.xml").unwrap();
    /// let result = fest.find_package("061561");
    ///
    /// assert_eq!(result.unwrap().itemnum(), "061561");
    /// ```
    pub fn find_package(&self, itemnum: &str) -> Option<&Package> {
        self.packages().iter().find(|p| p.itemnum == itemnum)
    }

    /// Search for generic products of a Package
    ///
    /// # Example
    /// ```
    /// use festlib::Fest;
    /// let fest = Fest::new("fest251.xml").unwrap();
    /// let package = fest.find_package("061561").unwrap();
    ///
    /// let result = fest.find_generic(&package);
    /// ```
    pub fn find_generic(&self, package: &Package) -> Option<Vec<Package>> {

        // if the package dont have any id theres no geneirc products for it
        if package.exchange_id().is_none() {
            return None;
        }

        let result = self.packages
            .iter()
            .filter(|p|
                p.exchange_id() ==
                package.exchange_id())
            .cloned()
            .collect();

        Some(result)

    }

    /// Search for interactions for two or more packages
    ///
    /// Will fail if called with vector smaller than 2.
    ///
    /// # Example
    /// ```
    /// use festlib::Fest;
    /// let fest = Fest::new("fest251.xml").unwrap();
    ///
    /// let package1 = fest.find_package("174532").unwrap();
    /// let package2 = fest.find_package("153742").unwrap();
    ///
    /// let check_interaction = vec![package1, package2];
    /// let interaction = fest.find_interaction(&check_interaction);
    ///
    /// ```
    pub fn find_interaction(&self, packages: &Vec<&Package>) -> Option<Vec<Interaction>> {
        // TODO: maybe just return None, since there is no drug that have an interaction with
        // itself.
        assert!(packages.len() > 1);

        let mut result = Vec::new();

        // extract the package atc codes and remove duplicates
        let mut atc_codes: Vec<String> = packages.iter().map(|p| p.atc.v.clone()).collect();
        atc_codes.dedup();

        // first search all interactions for the atc code.
        // if vector > 2 search

        let interactions = self.interactions.clone();
        let mut collection = Vec::new();

        // find all matching interaction for our atc codes and store them in
        // a vector
        for i in &interactions {
            for s in i.substances() {
                for a in &atc_codes {
                    if a == s.atc() {
                        collection.push(i);
                    }
                }
            }
        }

        // find all matching atc codes within our collected interactions
        // and if there is more than 2 matches we have an interaction
        for c in collection.clone() {
            let mut count = 0;
            for a in &atc_codes {
                for s in c.substances() {
                    if s.atc() == a {
                        count += 1;
                    }
                }
                if count > 1 {
                    result.push(c.clone());
                }
            }
            count = 0;
        }

        println!("size interaction: {}", interactions.len());
        println!("size collection: {}", collection.len());


        // TODO: maybe we should store the result in a map?
        // clear our result with dublicate interactions
        result.dedup_by_key(|r| r.id.clone());

        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    fn read_file(file: &str) -> Result<String, Box<dyn Error>> {
        let file_content = fs::read_to_string(file)?;

        Ok(file_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::NodeId;

    #[test]
    fn test_read_file() {
        let file = Fest::read_file("fest251.xml");
        assert!(file.is_ok());
    }

    #[test]
    fn test_hentetdato() {
        let fest = Fest::new("fest251.xml").unwrap();
        let date = fest.delivery_date();

        //let res = file.delivery_date().unwrap().unwrap();
        assert_eq!(date.date(), "2024-09-09T14:21:28");
    }

    #[test]
    fn test_metadata() {
        let fest = Fest::new("fest251.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        let node = content.get_node(NodeId::new(701764)).unwrap();
        let metadata = Metadata::new(&node);

        assert_eq!(metadata.id, "ID_F994748F-3A21-4FC3-9964-DBE097924A75");
        assert_eq!(metadata.time, "2024-04-21T00:51:31");
    }

    #[test]
    fn test_cs() {
        let fest = Fest::new("fest251.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        let node = content.get_node(NodeId::new(701764)).unwrap();
        let metadata = Metadata::new(&node);

        let cs = metadata.status;

        assert_eq!(cs.v, "A");
    }

    #[test]
    fn test_cv() {
        let fest = Fest::new("fest251.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        let node = content.get_node(NodeId::new(701764)).unwrap();

        let mut move_forward = None;

        // lets move the node into <Legemiddelpakning>
        for x in node.children() {
            if x.has_tag_name("Legemiddelpakning") {
                move_forward = Some(x.clone());
                break;
            }
        }

        let cv = Cv::new(&move_forward.unwrap(), "LegemiddelformKort");

        assert_eq!(cv.v, "32");
        assert_eq!(cv.s, "2.16.578.1.12.4.1.1.7448");
        assert_eq!(cv.dn, "Kapsel");
    }

    #[test]
    fn test_package() {
        let fest = Fest::new("fest251.xml").unwrap();

        let content = fest.content;
        let content = roxmltree::Document::parse(&content[0..]).unwrap();

        let node = content.get_node(NodeId::new(701764)).unwrap();

        let package = Package::new(&node).unwrap();

        assert_eq!(package.id, "ID_0138BA04-7B67-4FB5-B44D-7491336CAF20");
        assert_eq!(package.itemnum, "953335");
        assert_eq!(package.ean, "6430013130724");
    }


    #[test]
    fn test_fest_packages() {
        let fest = Fest::new("fest251.xml").unwrap();
        let packages = fest.packages();

        assert_eq!(packages.len(), 10473);
    }

    #[test]
    fn test_fest_find_package() {
        let fest = Fest::new("fest251.xml").unwrap();
        let packages = fest.packages();

        assert_eq!(packages.len(), 10473);

        let package = fest.find_package("061561").unwrap();
        assert_eq!(package.itemnum, "061561");
    }

   // #[test]
   // fn test_fest_find_no_generic() {
   //     let fest = Fest::new("fest251.xml").unwrap();
   //     let packages = fest.packages();

   //     assert_eq!(packages.len(), 10473);

   //     let package = fest.find_package("061561").unwrap();
   //     assert_eq!(package.itemnum, "061561");

   //     let result = fest.find_generic(&package);
   //     assert!(result.is_some());

   // }

    #[test]
    fn test_fest_find_generic() {
        let fest = Fest::new("fest251.xml").unwrap();
        let packages = fest.packages();

        assert_eq!(packages.len(), 10473);

        let package = fest.find_package("061561").unwrap();
        assert_eq!(package.itemnum, "061561");

        let result = fest.find_generic(&package);
        assert!(result.is_some());
    }

    #[test]
    fn test_fest_find_interation() {
        let fest = Fest::new("fest251.xml").unwrap();

        let package1 = fest.find_package("174532").unwrap();
        let package2 = fest.find_package("403119").unwrap();
        let package3 = fest.find_package("017646").unwrap();
        let package4 = fest.find_package("148460").unwrap();

        let check_interaction = vec![package1, package2, package3, package4];
        let interaction = fest.find_interaction(&check_interaction);

        assert!(interaction.is_some());
        assert_eq!(interaction.unwrap().len(), 3);
    }

}
