use embedded_time::clock::{Clock, Error};
use embedded_time::Instant;
use embedded_time::fraction::Fraction;
use embedded_time::duration::{Milliseconds, Seconds, Duration};
use embedded_time::duration::Generic;
use core::convert::{TryInto, TryFrom};
use cortex_m::interrupt::{Mutex, CriticalSection};
use embedded_time::fixed_point::FixedPoint;
use core::borrow::BorrowMut;
use core::cell::Cell;
use cortex_m::interrupt;
use core::marker::PhantomData;

pub struct Storage<Clock>
    where
        Clock: embedded_time::Clock,
{
    instant: Mutex<Cell<Option<Instant<Clock>>>>,
}

impl<Clock> Storage<Clock>
    where
        Clock: embedded_time::Clock,
{
    fn tick<Dur>(&self, duration: Dur)
        where
            Dur: Duration + FixedPoint,
            Clock::T: TryFrom<Dur::T>,
    {
        unsafe {
            interrupt::free(|cs| {
                let instant = self.instant.borrow(&cs);
                let i = instant.get();

                instant.replace(Some(
                    if i.is_some() {
                        i.unwrap().checked_add(duration).unwrap()
                    } else {
                        Instant::new(Clock::T::from(0u32))
                    }
                ));
            });
        }
    }

    fn get(&self) -> Result<Instant<Clock>, Error> {
        unsafe {
            interrupt::free(|cs| {
                let instant = self.instant.borrow(&cs).get();
                if instant.is_some() {
                    Ok(instant.unwrap())
                } else {
                    Err(Error::NotRunning)
                }
            })
        }
    }
}

pub struct Ticker<'a, Clock: embedded_time::Clock, Timer, IrqClearer: Fn(&mut Timer)>
{
    timer: Timer,
    irq_clearer: IrqClearer,
    instant: &'a Storage<Clock>,
}

impl<'a, Clock: embedded_time::Clock, Timer, IrqClearer: Fn(&mut Timer)> Ticker<'a, Clock, Timer, IrqClearer> {
    fn new(timer: Timer, irq_clearer: IrqClearer, storage: &'a Storage<Clock>) -> Self {
        Self {
            timer,
            irq_clearer,
            instant: &storage,
        }
    }

    pub fn tick(&mut self) {
        self.instant.tick(Milliseconds(250u32));
        //self.instant.tick();
        (self.irq_clearer)(&mut self.timer);
    }
}

macro_rules! clock {
    ($name:ident, $ticker_type:ident, $dur:expr, $scaling_factor:expr) => {

        pub struct $name
        {
            instant: Storage<Self>,
        }

        impl $name {
            pub const fn new() -> Self
            {
                Self {
                    instant: Storage {
                        instant: cortex_m::interrupt::Mutex::new(core::cell::Cell::new(Option::None)),
                    },
                }
            }

            pub fn ticker<Timer, IrqClearer: Fn(&mut Timer)>(&self, timer: Timer, irq_clearer: IrqClearer) -> $ticker_type<Self, Timer, IrqClearer> {
                let i = &self.instant;
                $ticker_type::new(timer, irq_clearer, i)
            }
        }

        impl Clock for $name {
            type T = u32;
            const SCALING_FACTOR: Fraction = $scaling_factor;

            fn try_now(&self) -> Result<Instant<Self>, Error> {
                //Ok(self.instant.get())
                self.instant.get()
            }
        }
        ticker!($ticker_type, $dur);
    }
}

macro_rules! ticker {
    ($name:ident, $tick:expr) => {
        pub struct $name<'a, Clock: embedded_time::Clock, Timer, IrqClearer: Fn(&mut Timer)>
        {
            timer: Timer,
            irq_clearer: IrqClearer,
            instant: &'a Storage<Clock>,
        }

        impl<'a, Clock: embedded_time::Clock, Timer, IrqClearer: Fn(&mut Timer)> $name<'a, Clock, Timer, IrqClearer> {
            fn new(timer: Timer, irq_clearer: IrqClearer, storage: &'a Storage<Clock>) -> Self {
                Self {
                    timer,
                    irq_clearer,
                    instant: &storage,
                }
            }

            pub fn tick(&mut self) {
                //self.instant.tick(Milliseconds(250u32));
                self.instant.tick( $tick );
                (self.irq_clearer)(&mut self.timer);
            }
        }
    }
}

clock!(MillisecondsClock1, MillisecondsTicker1, Milliseconds(1u32), Fraction::new(1,1000));
clock!(MillisecondsClock2, MillisecondsTicker2, Milliseconds(2u32), Fraction::new(1,500));
clock!(MillisecondsClock5, MillisecondsTicker5, Milliseconds(5u32), Fraction::new(1,200));
clock!(MillisecondsClock10, MillisecondsTicker10, Milliseconds(10u32), Fraction::new(1,100));
clock!(MillisecondsClock25, MillisecondsTicker25, Milliseconds(10u32), Fraction::new(1,50));
clock!(MillisecondsClock50, MillisecondsTicker50, Milliseconds(50u32), Fraction::new(1,20));
clock!(MillisecondsClock100, MillisecondsTicker100, Milliseconds(100u32), Fraction::new(1,10));
clock!(MillisecondsClock200, MillisecondsTicker200, Milliseconds(200u32), Fraction::new(1,5));
clock!(MillisecondsClock250, MillisecondsTicker250, Milliseconds(250u32), Fraction::new(1,4));
clock!(MillisecondsClock500, MillisecondsTicker500, Milliseconds(500u32), Fraction::new(1,2));
clock!(SecondsClock1, SecondsTicker1, Seconds(1u32), Fraction::new(1,1));
clock!(SecondsClock30, SecondsTicker30, Seconds(30u32), Fraction::new(30,1));
clock!(SecondsClock60, SecondsTicker60, Seconds(60u32), Fraction::new(60,1));

