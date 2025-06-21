fn main() {
    //lalrpop::Configuration::new()
    //    .force_build(true);
    lalrpop::process_src().unwrap();
}
