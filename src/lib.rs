extern crate chrono;

#[macro_use]
extern crate serde_derive;


#[macro_export]
macro_rules! expect
{
    ($e: expr, $message: tt, $($arg: expr),*) =>
    {
        $e.unwrap_or_else(|_| { eprint!("error: piggy: "); eprintln!($message, $($arg),*); ::std::process::exit(1) })
    }
}


pub mod data
{
    pub use chrono::{ NaiveDate, Utc };


    #[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Date(String);

    impl From<NaiveDate> for Date
    {
        fn from(date: NaiveDate) -> Date
        {
            Date(format!("{}", date))
        }
    }

    impl Date
    {
        pub fn as_naive(&self) -> Option<NaiveDate>
        {
            use std::str::FromStr;

            NaiveDate::from_str(&self.0).ok()
        }
    }


    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct PiggyBank
    {
        pub transactions: Vec<Transaction>
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Transaction
    {
        pub amount: f64,
        pub cause: String,
        pub date: Date
    }
}


use data::*;


pub fn parse_date_unchecked(date: &str) -> NaiveDate
{
    use std::str::FromStr;

    NaiveDate::from_str(date).unwrap()
}

pub fn balance_on_date(bank: &PiggyBank, date: NaiveDate) -> f64
{
    let date: Date = date.into();
    bank.transactions.iter().take_while(|acc| acc.date <= date).map(|acc| acc.amount).sum()
}

