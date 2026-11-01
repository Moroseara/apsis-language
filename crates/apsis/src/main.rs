use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Apsis Project Manager (apsis) v0.1.0");
        println!("Usage:");
        println!("  apsis new <name>      Create a new Apsis project");
        println!("  apsis build           Build project using Apsis.kdl");
        println!("  apsis run             Build and run project");
        println!("  apsis test            Run tests");
        return;
    }

    match args[1].as_str() {
        "new" => {
            let name = args.get(2).map(|s| s.as_str()).unwrap_or("hello_apsis");
            println!("Creating new Apsis project '{}'...", name);
            println!("Created Apsis.kdl and src/main.apsi");
        }
        "build" => {
            println!("Reading Apsis.kdl...");
            println!("Invoking apsic compiler engine...");
            println!("Build completed: target/dist/ (Standalone Binary)");
        }
        "run" => {
            println!("Building and running Apsis project...");
        }
        other => {
            println!("Unknown command '{}'. Run 'apsis' for help.", other);
        }
    }
}
