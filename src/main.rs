extern crate ansi_term;
extern crate chrono;
extern crate clap;
extern crate serde;
extern crate serde_yaml;

#[macro_use]
extern crate piggy;


use std::path::{ Path };
use serde::{ Serialize, Deserialize };
use piggy::data::*;


struct AppConfig
{
    pub currency: String
}

impl Default for AppConfig
{
    fn default() -> Self
    {
        AppConfig { currency: "£".to_owned() }
    }
}


fn main()
{
    use clap::{ App, SubCommand, Arg, AppSettings };

    let app = App::new("piggy")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Tool for tracking monthly spending")
        .settings(&[
            AppSettings::VersionlessSubcommands
        ])
        .subcommand(
            SubCommand::with_name("add")
                .about("Add some money into the piggy bank")
                .arg(
                    Arg::with_name("amount")
                        .help("The amount of money to add")
                        .required(true)
                        .takes_value(true)
                        .validator(is_f64)
                    )
                .arg(
                    Arg::with_name("cause")
                        .help("The source of the money")
                        .required(true)
                        .takes_value(true)
                    )
                .arg(
                    Arg::with_name("on")
                        .help("The date the money was added. Default is today")
                        .long("on")
                        .takes_value(true)
                        .validator(is_date)
                    )
                )
        .subcommand(
            SubCommand::with_name("spend")
                .about("Spend some money from the piggy bank")
                .arg(
                    Arg::with_name("amount")
                        .help("The amount of money spent")
                        .required(true)
                        .takes_value(true)
                        .validator(is_f64)
                    )
                .arg(
                    Arg::with_name("cause")
                        .help("The reason for spending the money")
                        .required(true)
                        .takes_value(true)
                    )
                .arg(
                    Arg::with_name("on")
                        .help("The date the money was spent. Default is today")
                        .long("on")
                        .takes_value(true)
                        .validator(is_date)
                    )
                )
        .subcommand(
            SubCommand::with_name("balance")
                .about("Display the balance on any given date")
                .arg(
                    Arg::with_name("on")
                        .help("The date to check the balance for. Default is today")
                        .long("on")
                        .takes_value(true)
                        .validator(is_date)
                    )
                );


    let dotfile = {
        use std::path::PathBuf;

        PathBuf::from("./.piggy")
    };

    let config = AppConfig::default();

    let mut bank: PiggyBank = {

        if !dotfile.exists()
        {
            write_file(&dotfile, &PiggyBank::default());
        }

        read_file(&dotfile)
    };

    let mut bank_modified = false;

    let today = Utc::today().naive_utc();

    let matches = app.get_matches();

    match matches.subcommand()
    {
        (command, Some(matches)) if command == "add" || command == "spend" =>
        {
            let amount: f64 = matches.value_of("amount").unwrap().parse().unwrap();
            let amount = if command == "add" { amount } else { -amount };
            let cause = matches.value_of("cause").unwrap().to_owned();
            let date = match matches.value_of("on")
            {
                Some(date) => piggy::parse_date_unchecked(date),
                None => today
            };
            bank.transactions.push(Transaction { amount, cause, date: date.into() });
            bank_modified = true;
        },
        ("balance", Some(matches)) =>
        {
            let date = match matches.value_of("on")
            {
                Some(date) => piggy::parse_date_unchecked(date),
                None => today
            };
            display_balance(&bank, date, &config);
        }
        _ => display_balance(&bank, today, &config)
    }

    if bank_modified
    {
        bank.transactions.sort_by_key(|acc| acc.date.as_naive().unwrap());
        write_file(&dotfile, &bank);
    }
}


fn is_f64(s: String) -> Result<(), String>
{
    if s.parse::<f64>().is_ok() { Ok(()) } else { Err("Expected a decimal value".to_owned()) }
}

fn is_date(s: String) -> Result<(), String>
{
    use std::str::FromStr;

    if NaiveDate::from_str(&s).is_ok() { Ok(()) } else { Err("Expected a date (yyyy-mm-dd)".to_owned()) }
}


fn read_file<T>(path: &Path) -> T
where for <'de>
    T: Deserialize<'de>
{
    use std::fs::File;

    let file = expect!(File::open(path), "Failed to open {:?}", path);
    expect!(serde_yaml::from_reader(file), "Failed to parse file {:?}", path)
}

fn write_file<T: Serialize>(path: &Path, data: &T)
{
    use std::fs::File;

    let file = expect!(File::create(path), "Failed to open {:?}", path);
    expect!(serde_yaml::to_writer(file, data), "Failed to write file {:?}", path);
}

fn display_balance(bank: &PiggyBank, date: NaiveDate, config: &AppConfig)
{
    use ansi_term::Color;

    let balance = piggy::balance_on_date(&bank, date);
    let balance_string = Color::Fixed(15).paint("Balance: ");
    let value_color = if balance < 0.0 { Color::Fixed(9) } else { Color::Fixed(10) };
    let value_string = value_color.paint(format!("{}{}", &config.currency, balance));
    println!("{}{}", balance_string, value_string);
    println!("{:?}", bank);
}
