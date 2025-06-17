// Copyright 2025 Oxide Computer Company

//! A simple program that spawns some Tokio tasks.
//!
//! Try running this program and then running `examples/print-all.d` with its
//! PID!

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_time().enable_io();
    let rt = tokio_dtrace::register_hooks(&mut builder)?.build()?;

    rt.block_on(async {
        tokio::spawn(async {
            loop {
                for secs in 0..10 {
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
                    });
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });

        tokio::signal::ctrl_c().await.unwrap();
    });

    Ok(())
}
