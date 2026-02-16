mod tui;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 2 {
        eprintln!("hextui expects 2 arguments\nusage: hextui [file]")
    }

    let path = std::path::Path::new(&args[1]);

    if !path.exists() {
       eprintln!("could not find file {}", path.display());
       return;
    }

    tui::run(path);
}
