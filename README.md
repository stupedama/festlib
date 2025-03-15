# festlib - a rust library for parsing fest file from DMP


## Examples

### Find package
```
use festlib::Fest;

let fest = Fest::new("fest251.xml").expect("Could not open xml file");

let package = fest.find_package("061561");
```

### Interactions
```
use festlib::Fest;

let fest = Fest::new("fest251.xml").expect("Could not open xml file");

// First find the package we want to find generic products for
let package1 = fest.find_package("061561");
let package2 = fest.find_package("017701");

// Store packages into a vector for interaction test
let mut packages = Vec::new();

if let Some(p) = package1 {
    packages.push(p);
}

if let Some(p) = package2 {
    packages.push(p);
}

let interaction = fest.find_interaction(&packages);
```

### Generic products
```
use festlib::Fest;

let fest = Fest::new("fest251.xml").expect("Could not open xml file");

let package = fest.find_package("061561");

if let Some(p) = package {
    let generic = fest.find_generic(&p);
}
```

## tests
Before you run tests you need to download the fest file and store it
in the project directory.

### download fest file
```
wget https://www.legemiddelsok.no/_layouts/15/FESTmelding/fest251_inst.zip
unzip fest251.xml
```

### running tests
```
cargo test
```
