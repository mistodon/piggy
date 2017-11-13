extern crate ansi_term;
extern crate chrono;
extern crate serde;
extern crate serde_yaml;

extern crate structopt;

#[macro_use]
extern crate structopt_derive;

#[macro_use]
extern crate piggy;


use std::path::{ Path };
use serde::{ Serialize, Deserialize };
use piggy::data::*;


struct AppConfig
{
    pub currency: String,
    pub payday: Day
}

impl Default for AppConfig
{
    fn default() -> Self
    {
        AppConfig 
        { 
            currency: "£".to_owned(),
            payday: Day::new(25).unwrap()
        }
    }
}


// TODO: Prevent duplicate monthly transactions
// TODO: Add `end` subcommand to stop a monthly transaction
// TODO: Add `config` subcommand to change payday, currency, etc.
#[derive(StructOpt)]
#[structopt()]
struct Piggy
{
    #[structopt(short = "f", long = "file", help = "The .piggy file to use. Defaults to ./.piggy then ~/.piggy.")]
    file: Option<String>,

    #[structopt(subcommand)]
    subcommand: Option<PiggySubcommand>
}

#[derive(StructOpt)]
enum PiggySubcommand
{
    #[structopt(name = "add", about = "Add some money to the piggy bank.")]
    Add
    {
        #[structopt(help = "The amount of money to add.")]
        amount: f64,

        #[structopt(help = "The source of the money.")]
        cause: String,

        #[structopt(long = "on", help = "The date the money was added.", default_value = "today")]
        on: Date,

        #[structopt(short = "m", long = "monthly", help = "Add this amount of money this day every month.")]
        monthly: Option<Day>
    },

    #[structopt(name = "spend", about = "Spend some money from the piggy bank")]
    Spend
    {
        #[structopt(help = "The amount of money to spend.")]
        amount: f64,

        #[structopt(help = "The reason for spending the money.")]
        cause: String,

        #[structopt(long = "on", help = "The date the money was spent.", default_value = "today")]
        on: Date,

        #[structopt(short = "m", long = "monthly", help = "Spend this amount of money this day every month.")]
        monthly: Option<Day>
    },

    #[structopt(name = "balance", about = "Display the balance on a certain date")]
    Balance
    {
        #[structopt(long = "on", help = "The date to check the balance for.", default_value = "today")]
        on: Date
    },

    #[structopt(name = "set-balance", about = "Add or spend enough to set the balance to the given value")]
    SetBalance
    {
        #[structopt(help = "The new balance.")]
        amount: f64,

        #[structopt(help = "The reason for adjusting the balance.", default_value = "Set balance")]
        cause: String,

        #[structopt(long = "on", help = "The date to set the balance on.", default_value = "today")]
        on: Date
    }
}


fn main()
{
    use structopt::StructOpt;

    let command = Piggy::from_args();

    let dotfile = {
        if let Some(path) = command.file
        {
            use std::path::PathBuf;

            PathBuf::from(path)
        }
        else
        {
            let here: &Path = "./.piggy".as_ref();

            if here.exists()
            {
                here.to_owned()
            }
            else
            {
                use std::env;
                let mut home = expect!(env::home_dir().ok_or(()), "Failed to find home directory");
                home.push(".piggy");
                home
            }
        }
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
    let mut date_to_report = today;

    match command.subcommand
    {
        // TODO: Fix DRY fail between add/spend
        Some(PiggySubcommand::Add { amount, cause, on, monthly }) =>
        {
            let date = on;
            match monthly
            {
                Some(day) => {
                    bank.monthly_transactions.push(MonthlyTransaction { amount, cause, day, start_date: date, end_date: None })
                }
                None => bank.transactions.push(Transaction { amount, cause, date })
            }
            bank_modified = true;
        },

        Some(PiggySubcommand::Spend { amount, cause, on, monthly }) =>
        {
            let date = on;
            let amount = -amount;
            match monthly
            {
                Some(day) => {
                    bank.monthly_transactions.push(MonthlyTransaction { amount, cause, day, start_date: date, end_date: None })
                }
                None => bank.transactions.push(Transaction { amount, cause, date })
            }
            bank_modified = true;
        },

        Some(PiggySubcommand::SetBalance { amount, cause, on }) =>
        {
            let current_balance: f64 = piggy::transactions_by_date(&bank, on.0).iter().map(|t| t.amount).sum();
            let change = amount - current_balance;
            bank.transactions.push(Transaction { amount: change, cause, date: on });
            bank_modified = true;
        },

        Some(PiggySubcommand::Balance { on, .. }) => date_to_report = on.0,

        None => ()
    }

    if bank_modified
    {
        bank.transactions.sort_by_key(|acc| acc.date);
        write_file(&dotfile, &bank);
    }

    display_monthly_account(&bank, date_to_report, &config);
    display_balance(&bank, date_to_report, &config);
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
    
    let balance: f64 = piggy::transactions_by_date(bank, date).iter().map(|t| t.amount).sum();
    let balance_string = Color::Fixed(15).paint("Balance: ");
    let value_color = if balance < 0.0 { Color::Fixed(9) } else { Color::Fixed(10) };
    let value_string = value_color.paint(format!("{}{:.2}", &config.currency, balance));
    println!("{}{}", balance_string, value_string);
}


fn display_monthly_account(bank: &PiggyBank, date: NaiveDate, config: &AppConfig)
{
    use ansi_term::Color;

    let prev_payday = piggy::get_previous_day(config.payday, date);
    let next_payday = piggy::get_next_day(config.payday, date);

    let transactions = piggy::transactions_by_date(bank, next_payday);

    let mut working_balance = transactions.iter()
        .filter(|t| t.date.0 < prev_payday)
        .map(|t| t.amount)
        .sum();

    let white = Color::Fixed(15);
    let grey = Color::Fixed(8);
    let red = Color::Fixed(9);
    let green = Color::Fixed(10);
    let blue = Color::Fixed(12);

    let format_money = |amount: f64, pos_op|
    {
        let (color, sign, signum) = match amount
        {
            n if n < 0.0 => (red, '-', -1.0),
            _ => (green, pos_op, 1.0)
        };
        let text = format!("{}{}{:.2}", sign, &config.currency, amount * signum);
        color.paint(format!("{: >10}", text))
    };

    let mut today_shown = false;

    for transaction in transactions.iter().filter(|t| t.date.0 >= prev_payday)
    {
        match transaction.date.0
        {
            d if d == date => today_shown = true,
            d if d > date && !today_shown =>
            {
                println!("{}:{: >15}{}", blue.paint(date.to_string()), "", format_money(working_balance, ' '));
                today_shown = true;
            },
            _ => ()
        }

        working_balance += transaction.amount;

        let date_color = match transaction.date.0
        {
            d if d < date => white,
            d if d > date => grey,
            _ => blue
        };
        let date_label = date_color.paint(transaction.date.0.to_string());
        let delta_label = format_money(transaction.amount, '+');
        let balance_label = format_money(working_balance, ' ');

        println!("{}: {} -> {} - {}",
                 date_label,
                 delta_label,
                 balance_label,
                 transaction.cause);
    }
}
