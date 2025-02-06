use std::thread;
use std::time::Duration;
use tokio::fs;
use tokio::runtime::{Builder, Runtime};
use tokio::time::sleep;

fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(run(&rt));
    });

    handle.join().unwrap();
}

async fn run(rt: &Runtime) {
    rt.spawn(async {
        println!("future 1");
        let content = fs::read("Cargo.toml").await.unwrap();
        println!("content: {:?}", content.len());
    });

    rt.spawn(async {
        println!("future 2");
        let result = expensive_blocking_task("hello".to_string());
        println!("result: {:?}", result);
    });

    sleep(Duration::from_secs(1)).await;
}

fn expensive_blocking_task(s: String) -> String {
    // Simulate a blocking task
    thread::sleep(Duration::from_millis(800));
    blake3::hash(s.as_bytes()).to_string()
}
