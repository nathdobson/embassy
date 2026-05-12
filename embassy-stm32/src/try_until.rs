//! Provides a function to try until something is true

#[cfg(feature = "time")]
/// Function to try until something is true
#[allow(unused)]
pub async fn try_until(mut func: impl AsyncFnMut() -> bool, micros: u64) -> Result<(), ()> {
    use embassy_time::{Duration, Ticker};

    let duration = Duration::from_micros(micros);
    let tick = Duration::from_millis(1);
    let mut ticker = Ticker::every(tick);
    let ticks = duration.as_ticks() / tick.as_ticks();

    for _ in 0..ticks {
        if func().await {
            return Ok(());
        }

        ticker.next().await;
    }

    Err(())
}

#[cfg(not(feature = "time"))]
/// Function to try until something is true
#[allow(unused)]
pub async fn try_until(mut func: impl AsyncFnMut() -> bool, micros: u64) -> Result<(), ()> {
    use embassy_futures::yield_now;

    let ticks = micros / 1_000;

    for _ in 0..ticks {
        if func().await {
            return Ok(());
        }

        crate::block_for_us(1_000);
        yield_now().await;
    }

    Err(())
}
