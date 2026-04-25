use pdfcmp_algo::compare::compare_dirs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: run <all_dir> <sent_dir>");
        std::process::exit(1);
    }
    let r = compare_dirs(Path::new(&args[1]), Path::new(&args[2]));
    println!(
        "左 {} / 右 {} / 缺失 {} / 多余 {}",
        r.stats.total_left, r.stats.total_right, r.stats.missing_count, r.stats.extra_count
    );
    println!("--- 缺失 ---");
    for f in &r.missing {
        println!("  {}", f.full_name);
    }
    println!("--- 多余 ---");
    for f in &r.extra {
        println!("  {}", f.full_name);
    }
}
