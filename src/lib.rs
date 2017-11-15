extern crate chrono;
extern crate serde;

#[macro_use]
extern crate serde_derive;

pub mod data;
pub mod failure;

use data::*;
use failure::SafeUnwrap;



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

pub fn get_previous_day(day: Day, current_date: NaiveDate) -> NaiveDate
{
    use chrono::Datelike;

    let current_day = current_date.day();

    match current_day
    {
        d if day.day() <= d => current_date.with_day(day.day()).safe_unwrap(),
        _ => {
            let current_year = current_date.year();
            let (year, month) = match current_date.month()
            {
                1 => (current_year - 1, 12),
                n => (current_year, n - 1)
            };
            NaiveDate::from_ymd(year, month, day.day())
        }
    }
}

pub fn get_next_day(day: Day, current_date: NaiveDate) -> NaiveDate
{
    let prev_day = get_previous_day(day, current_date);
    same_day_next_month(prev_day).safe_unwrap()
}

pub fn transactions_by_date(bank: &PiggyBank, date: NaiveDate) -> Vec<Transaction>
{
    let mut transactions: Vec<Transaction> = bank.transactions.iter().filter(|t| t.date.0 <= date).map(Clone::clone).collect();

    for monthly in &bank.monthly_transactions
    {
        let mut occurrence = {
            let occurrence = get_previous_day(monthly.day, monthly.start_date.0);

            // TODO: This is kind of lame. We want first occurrence of monthly transaction,
            // but start_date may or may not be an occurrence.
            match occurrence < monthly.start_date.0
            {
                true => get_next_day(monthly.day, occurrence),
                false => occurrence
            }
        };

        loop
        {
            if let Some(end_date) = monthly.end_date
            {
                if occurrence > end_date.0
                {
                    break;
                }
            }

            if occurrence > date
            {
                break;
            }

            transactions.push(
                Transaction
                { 
                    amount: monthly.amount,
                    cause: monthly.cause.to_owned(),
                    date: Date(occurrence)
                });

            occurrence = same_day_next_month(occurrence).safe_unwrap();
        }
    }

    transactions.sort_by_key(|t| t.date);

    transactions
}


pub fn monthlies_conflict(t0: &MonthlyTransaction, t1: &MonthlyTransaction) -> bool
{
    let start0 = t0.start_date;
    let start1 = t1.start_date;

    match (t0.end_date, t1.end_date)
    {
        (Some(end0), Some(end1)) => (start0 < end1) && (start1 < end0),
        (None, Some(end1)) => start0 < end1,
        (Some(end0), None) => start1 < end0,
        (None, None) => true
    }
}
