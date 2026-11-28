fn main() {
    glib_build_tools::compile_resources(&["data"], "data/eva.gresource.xml", "eva.gresource");
}
