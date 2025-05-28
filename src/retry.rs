use futures::Future;
use std::time::Duration;
use tokio::time::sleep;

pub async fn retry<Res, Err, Op, Fut>(
    op: Op,
    retries: Option<usize>,
    max_delay: Option<Duration>,
) -> Result<Res, Err>
where
    Op: Fn() -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    let mut attempt = 0;
    let mut current_delay = Duration::from_millis(100);
    let retries = retries.unwrap_or(3);
    let max_delay = max_delay.unwrap_or(Duration::from_secs(30));

    loop {
        match op().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= retries {
                    return Err(e);
                }

                current_delay = current_delay.mul_f64(2.0).min(max_delay);
                sleep(current_delay).await;
                attempt += 1;
            }
        }
    }
}
