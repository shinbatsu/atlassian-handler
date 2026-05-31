use std::process;

use clap::Parser;

mod license;

#[derive(Parser, Debug)]
#[command(name = "handler", version = "1.0.0", about = "Atlassian License Generator")]
struct Cli {
    #[arg(short = 'm', long = "mail", default_value = "")]
    mail: String,

    #[arg(short = 'n', long = "name", default_value = "")]
    name: String,

    #[arg(short = 'o', long = "org", default_value = "")]
    org: String,

    #[arg(short = 'p', long = "product", default_value = "")]
    product: String,

    #[arg(short = 's', long = "server-id", default_value = "")]
    server_id: String,

    #[arg(short = 'd', long = "data-center", default_value_t = false)]
    data_center: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.mail.is_empty() || cli.org.is_empty() || cli.product.is_empty() || cli.server_id.is_empty() {
        eprintln!("Error: missing required arguments");
        eprintln!("Use -h for help");
        print_usage();
        process::exit(1);
    }

    let name = if cli.name.is_empty() {
        cli.mail.clone()
    } else {
        cli.name
    };

    let product_cfg = match license::products::get_product_config(&cli.product) {
        Some(cfg) => cfg,
        None => {
            eprintln!("Error: unknown product '{}'", cli.product);
            print_usage();
            process::exit(1);
        }
    };

    let mut license_data = license::generator::LicenseData::new(
        name,
        cli.mail,
        cli.server_id,
        cli.org,
        cli.data_center,
        product_cfg.name,
    );

    match license_data.generate() {
        Ok(result) => println!("{}", result),
        Err(e) => {
            eprintln!("Error generating license: {}", e);
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("\nSupported products:");
    let products = license::products::get_products();
    let mut keys: Vec<&String> = products.keys().collect();
    keys.sort();
    for key in keys {
        eprintln!("  {:<15} {}", format!("{}:", key), products[key]);
    }
    eprintln!();
    eprintln!("Example:");
    eprintln!("  handler -m admin@example.com -n \"Admin User\" -o \"My Company\" -p crowd -s ABCD-1234-EFGH-5678 -d");
}