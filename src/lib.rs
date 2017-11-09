extern crate chrono;
extern crate serde;

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
    use serde::{ Serialize, Deserialize, Serializer, Deserializer };

    #[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
    pub struct Date(pub NaiveDate);

    impl Serialize for Date
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer
        {
            self.0.to_string().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Date
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>
        {
            use std::str::FromStr;
            use serde::de::{ Error, Unexpected };

            let date_string: String = Deserialize::deserialize(deserializer)?;
            
            match NaiveDate::from_str(&date_string)
            {
                Ok(date) => Ok(Date(date)),
                Err(_) => Err(
                    D::Error::invalid_value(
                        Unexpected::Str(&date_string), &"a valid date of the form \"yyyy-mm-dd\""))
            }
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
    let date = Date(date);
    bank.transactions.iter().take_while(|acc| acc.date <= date).map(|acc| acc.amount).sum()
}

pub fn balance_before_date(bank: &PiggyBank, date: NaiveDate) -> f64
{
    let date = Date(date);
    bank.transactions.iter().take_while(|acc| acc.date < date).map(|acc| acc.amount).sum()
}

pub fn get_previous_payday(payday: u32, current_date: NaiveDate) -> NaiveDate
{
    use chrono::Datelike;

    assert!(payday > 0 && payday <= 28);

    let current_day = current_date.day();

    if current_day > payday
    {
        current_date.with_day(payday).unwrap()
    }
    else
    {
        let current_year = current_date.year();
        let (year, month) = match current_date.month()
        {
            1 => (current_year - 1, 12),
            n => (current_year, n - 1)
        };
        NaiveDate::from_ymd(year, month, payday)
    }
}

pub fn get_next_payday(payday: u32, current_date: NaiveDate) -> NaiveDate
{
    use chrono::Datelike;

    assert!(payday > 0 && payday <= 28);

    let prev_payday = get_previous_payday(payday, current_date);

    let current_year = prev_payday.year();
    let (year, month) = match prev_payday.month()
    {
        12 => (current_year + 1, 1),
        n => (current_year, n + 1)
    };

    NaiveDate::from_ymd(year, month, payday)
}

