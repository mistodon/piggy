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


pub mod data;

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

// TODO: This seems to erroneously count the start_date even if it's the wrong Day
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

// TODO: This seems to erroneously count the start_date even if it's the wrong Day
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

pub fn get_previous_day(day: Day, current_date: NaiveDate) -> Option<NaiveDate>
{
    use chrono::Datelike;

    let current_day = current_date.day();

    match current_day
    {
        d if day.0 <= d => current_date.with_day(day.0),
        _ => {
            let current_year = current_date.year();
            let (year, month) = match current_date.month()
            {
                1 => (current_year - 1, 12),
                n => (current_year, n - 1)
            };
            NaiveDate::from_ymd_opt(year, month, day.0)
        }
    }
}

pub fn get_next_day(day: Day, current_date: NaiveDate) -> Option<NaiveDate>
{
    get_previous_day(day, current_date).and_then(|d| same_day_next_month(d))
}

