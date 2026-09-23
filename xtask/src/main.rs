use {
    std::{env, fs, path::PathBuf},
    tiny_skia::Transform,
    usvg::{Options, Tree},
};

use std::error::Error;

fn png(tree: &Tree, size: u32) -> Result<(), Box<dyn Error>> {
    let svg_size = tree.size();
    let scale = (size as f32 / svg_size.width()).min(size as f32 / svg_size.height());
    let transform = Transform::from_scale(scale, scale);
    let mut pixmap = match tiny_skia::Pixmap::new(size, size) {
        Some(p) => p,
        None => return Err(String::from("Error creating png").into()),
    };
    resvg::render(tree, transform, &mut pixmap.as_mut());
    let sizedir = format!("{}x{}", size, size);
    let outdir: PathBuf = [
        "target", "dist", "share", "icons", "hicolor", &sizedir, "apps",
    ]
    .iter()
    .collect();
    if !outdir.exists() {
        fs::create_dir_all(&outdir)?;
    }
    let mut outfile = outdir;
    outfile.push("eva.png");
    let infile: PathBuf = ["data", "eva.svg"].iter().collect();
    println!("    {} -> {}", infile.display(), outfile.display());
    pixmap.save_png(outfile)?;
    Ok(())
}

fn iconvert() -> Result<(), Box<dyn Error>> {
    println!("Creating png icons from svg:");
    let infile: PathBuf = ["data", "eva.svg"].iter().collect();
    let data = fs::read(&infile)?;
    let tree = Tree::from_data(&data, &Options::default())?;
    for size in [128, 64, 48, 32] {
        png(&tree, size)?;
    }
    Ok(())
}

fn copy_data() -> Result<(), Box<dyn Error>> {
    println!("Copying data files:");
    let appdir: PathBuf = ["target", "dist", "share", "applications"].iter().collect();
    if !appdir.exists() {
        fs::create_dir_all(&appdir)?;
    }
    let mut outfile = appdir;
    outfile.push("org.hitchhiker-linux.eva.desktop");
    let infile: PathBuf = ["data", "org.hitchhiker-linux.eva.desktop"]
        .iter()
        .collect();
    fs::copy(&infile, &outfile)?;
    println!("    {} -> {}", infile.display(), outfile.display());
    let icondir: PathBuf = [
        "target", "dist", "share", "icons", "hicolor", "scalable", "apps",
    ]
    .iter()
    .collect();
    if !icondir.exists() {
        fs::create_dir_all(&icondir)?;
    }
    let mut outfile = icondir;
    outfile.push("eva.svg");
    let infile: PathBuf = ["data", "eva.svg"].iter().collect();
    fs::copy(&infile, &outfile)?;
    println!("    {} -> {}", infile.display(), outfile.display());
    Ok(())
}

fn copy_bin() -> Result<(), Box<dyn Error>> {
    println!("Copying binary:");
    let bindir: PathBuf = ["target", "dist", "bin"].iter().collect();
    if !bindir.exists() {
        fs::create_dir_all(&bindir)?;
    }
    let mut outfile = bindir;
    outfile.push("eva");
    let infile: PathBuf = ["target", "release", "eva"].iter().collect();
    if !infile.exists() {
        eprintln!("Error: you must run \"cargo build --release\" first");
    }
    fs::copy(&infile, &outfile)?;
    println!("    {} -> {}", infile.display(), outfile.display());
    Ok(())
}

fn usage() {
    println!("Usage: xtask dist");
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
        return Ok(());
    }
    if &args[1] == "dist" {
        let outdir: PathBuf = ["target", "dist"].iter().collect();
        if outdir.exists() {
            fs::remove_dir_all(&outdir)?;
        }
        copy_bin()?;
        copy_data()?;
        iconvert()?;
    } else {
        usage();
    }
    Ok(())
}
