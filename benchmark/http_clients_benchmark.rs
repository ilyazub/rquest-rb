use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use futures::future::join_all;
use std::error::Error;

const REQUEST_COUNT: usize = 10;
const URL: &str = "https://serpapi.com/robots.txt";

fn reqwest_blocking() -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    
    for _ in 0..REQUEST_COUNT {
        let _resp = client.get(URL).send()?;
    }
    
    Ok(())
}

async fn reqwest_async() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    
    let requests = (0..REQUEST_COUNT).map(|_| {
        let client = client.clone();
        async move {
            client.get(URL).send().await?;
            Ok::<(), Box<dyn Error>>(())
        }
    });
    
    join_all(requests).await;
    Ok(())
}

async fn rquest_async() -> Result<(), Box<dyn Error>> {
    let client = rquest::Client::builder()
        .emulation(rquest_util::Emulation::Chrome134)
        .build()?;

    let requests = (0..REQUEST_COUNT).map(|_| {
        let client = client.clone();
        async move {
            client.get(URL).send().await?;
            Ok::<(), Box<dyn Error>>(())
        }
    });
    
    join_all(requests).await;
    Ok(())
}

fn bench_http_clients(c: &mut Criterion) {
    let mut group = c.benchmark_group("HTTP Clients - 10 GET requests");
    
    group.bench_function(BenchmarkId::new("reqwest", "blocking"), |b| {
        b.iter(|| {
            reqwest_blocking().unwrap();
        });
    });
    
    group.bench_function(BenchmarkId::new("reqwest", "async"), |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                reqwest_async().await.unwrap();
            });
        });
    });
    
    group.bench_function(BenchmarkId::new("rquest", "async"), |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                rquest_async().await.unwrap();
            });
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_http_clients);
criterion_main!(benches); 