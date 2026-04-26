fn main() {
    let _a: Vec<i32> = Vec::new();
    let mut _b = Vec::new();
    _b.push(1);
    
    // With explicit type, should NOT be linted
    let _c = Vec::<u32>::new();

    // From std::vec::Vec, should trigger
    let _d: Vec<String> = std::vec::Vec::new();
}
