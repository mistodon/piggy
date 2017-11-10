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
        pub transactions: Vec<Transaction>,
        pub monthly_transactions: Vec<MonthlyTransaction>
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Transaction
    {
        pub amount: f64,
        pub cause: String,
        pub date: Date
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MonthlyTransaction
    {
        pub amount: f64,
        pub cause: String,
        pub day: u32,
        pub start_date: Date,
        pub end_date: Option<Date>
    }
}


use data::*;


pub struct MonthlyIter
{
    date: Option<NaiveDate>
}

impl Iterator for MonthlyIter
{
    type Item = NaiveDate;

    fn next(&mut self) -> Option<NaiveDate>
    {
        let value = self.date;
        self.date = self.date.and_then(|date| same_day_next_month(date));
        value
    }
}


pub struct MonthlyTransactionIter<'a>
{
    transaction: &'a MonthlyTransaction,
    dates: MonthlyIter
}

impl<'a> Iterator for MonthlyTransactionIter<'a>
{
    type Item = Transaction;

    fn next(&mut self) -> Option<Transaction>
    {
        let date = self.dates.next();

        match date
        {
            None => None,
            Some(date) if date < self.transaction.start_date.0 => None,
            Some(date) => {
                if let Some(end) = self.transaction.end_date
                {
                    if date > end.0
                    {
                        return None;
                    }
                }

                Some(Transaction { date: Date(date), amount: self.transaction.amount, cause: self.transaction.cause.clone() })
            }
        }
    }
}


pub fn parse_date_unchecked(date: &str) -> NaiveDate
{
    use std::str::FromStr;

    NaiveDate::from_str(date).unwrap()
}


// TODO: Clean up DRY fail between on/before
pub fn balance_on_date(bank: &PiggyBank, date: NaiveDate) -> f64
{
    let subtotal: f64 = bank.transactions
        .iter()
        .take_while(|acc| acc.date.0 <= date)
        .map(|acc| acc.amount)
        .sum();
    subtotal + monthly_transactions_on_date(bank, date)
}

pub fn balance_before_date(bank: &PiggyBank, date: NaiveDate) -> f64
{
    let subtotal: f64 = bank.transactions
        .iter()
        .take_while(|acc| acc.date.0 < date)
        .map(|acc| acc.amount)
        .sum();
    subtotal + monthly_transactions_before_date(bank, date)
}

fn monthly_transactions_on_date(bank: &PiggyBank, date: NaiveDate) -> f64
{
    let mut total = 0.0;

    for transaction in &bank.monthly_transactions
    {
        let dates = MonthlyIter { date: Some(transaction.start_date.0) };
        for next in dates
        {
            if next > date
            {
                break
            }
            total += transaction.amount;
        }
    }

    total
}

fn monthly_transactions_before_date(bank: &PiggyBank, date: NaiveDate) -> f64
{
    let mut total = 0.0;

    for transaction in &bank.monthly_transactions
    {
        let dates = MonthlyIter { date: Some(transaction.start_date.0) };
        for next in dates
        {
            if next >= date
            {
                break
            }
            total += transaction.amount;
        }
    }

    total
}

pub fn same_day_next_month(date: NaiveDate) -> Option<NaiveDate>
{
    use chrono::Datelike;

    let day = date.day();
    if day > 28
    {
        return None;
    }

    let current_year = date.year();
    let (year, month) = match date.month()
    {
        12 => (current_year + 1, 1),
        n => (current_year, n + 1)
    };
    Some(NaiveDate::from_ymd(year, month, day))
}

pub fn get_previous_day(day: u32, current_date: NaiveDate) -> Option<NaiveDate>
{
    use chrono::Datelike;

    let current_day = current_date.day();

    match current_day
    {
        d if day <= d => current_date.with_day(day),
        _ => {
            let current_year = current_date.year();
            let (year, month) = match current_date.month()
            {
                1 => (current_year - 1, 12),
                n => (current_year, n - 1)
            };
            NaiveDate::from_ymd_opt(year, month, day)
        }
    }
}

pub fn get_next_day(day: u32, current_date: NaiveDate) -> Option<NaiveDate>
{
    get_previous_day(day, current_date).and_then(|d| same_day_next_month(d))
}

