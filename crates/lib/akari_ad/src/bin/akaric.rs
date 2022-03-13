extern crate syn;
fn main() {
    let s = std::fs::read_to_string("example/simple.rs").unwrap();
    let f: syn::File = syn::parse_str(&s).unwrap();
    println!("{:#?}", f);
}
