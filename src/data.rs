use std::str::FromStr;
use chrono::ParseError;
use serde::{ Serialize, Deserialize, Serializer, Deserializer };


pub use chrono::{ NaiveDate, Utc };


#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Date(pub NaiveDate);

impl FromStr for Date
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s
        {
            "today" => Ok(Date(Utc::today().naive_utc())),
            s => NaiveDate::from_str(s).map(|d| Date(d))
        }
    }
}

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


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Day(pub u32);

impl FromStr for Day
{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let day: u32 = s.parse().map_err(|_| "Expected day between 1 and 28")?;
        if day > 0 && day <= 28 { Ok(Day(day)) } else { Err("Expected day between 1 and 28") }
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
    pub day: Day,
    pub start_date: Date,
    pub end_date: Option<Date>
}
