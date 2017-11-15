#[macro_export]
macro_rules! fail
{
    ($message: tt) =>
    {
        { eprint!("error: piggy: "); eprintln!($message); ::std::process::exit(1) }
    };

    ($message: tt, $($arg: expr),*) =>
    {
        { eprint!("error: piggy: "); eprintln!($message, $($arg),*); ::std::process::exit(1) }
    }
}

#[macro_export]
macro_rules! expect
{
    ($e: expr, $message: tt) =>
    {
        $e.unwrap_or_else(|_| { fail!($message) })
    };

    ($e: expr, $message: tt, $($arg: expr),*) =>
    {
        $e.unwrap_or_else(|_| { fail!($message, $($arg),*) })
    }
}


pub trait SafeUnwrap
{
    type Contained;

    fn safe_unwrap(self) -> Self::Contained;
}

impl<T> SafeUnwrap for Option<T>
{
    type Contained = T;

    fn safe_unwrap(self) -> Self::Contained
    {
        self.unwrap_or_else(|| unreachable!())
    }
}

impl<T, E> SafeUnwrap for Result<T, E>
{
    type Contained = T;

    fn safe_unwrap(self) -> Self::Contained
    {
        self.unwrap_or_else(|_| unreachable!())
    }
}

