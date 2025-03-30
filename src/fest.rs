use std::fs;
use std::error::Error;
use crate::types::{Package, Interaction, LastUpdate};
use crate::xml;

/// Container for the fest file
pub struct Fest {
    _filename: String,
    pub content: String, // TODO: remove the test, so we dont need pub
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
        self.packages().iter().find(|p| p.itemnum() == itemnum)
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
        let mut atc_codes: Vec<String> = packages.iter().map(|p| p.atc().v().clone()).collect();
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
        result.dedup_by_key(|r| r.id().clone());

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
        assert_eq!(package.itemnum(), "061561");
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
        assert_eq!(package.itemnum(), "061561");

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
