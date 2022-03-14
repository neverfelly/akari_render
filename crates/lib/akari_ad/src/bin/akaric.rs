use akari_ad::codegen::AdCodeGen;

extern crate syn;
fn main() {
    let s = std::fs::read_to_string("example/simple.rs").unwrap();
    // let f: syn::File = syn::parse_str(&s).unwrap();
    //
    let f = akari_ad::parse::parse_str(&s);
    println!("{:#?}", f);
    let mut cg = AdCodeGen::new();
    println!("{}", cg.gen_forward(&f));
}
