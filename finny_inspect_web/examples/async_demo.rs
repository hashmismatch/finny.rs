//! An async FSM on tokio, with concurrent regions and timers, inspected in the browser.
//!
//! `cargo run -p finny_inspect_web --example async_demo`, then open http://127.0.0.1:7878

use std::{sync::{Mutex, atomic::{AtomicU32, Ordering}}, time::Duration};

use finny::{FsmAsyncFactory, FsmEventQueueVec, FsmResult, decl::{BuiltFsm, FsmAsyncBuilder}, finny_fsm, timers::tokio::TimersTokio};
use finny_inspect_web::{Inspector, InspectorConfig};
use serde::Serialize;

/// The context of an async FSM is shared, it uses interior mutability.
#[derive(Default, Serialize)]
pub struct Downloads {
    completed: AtomicU32,
    failed: AtomicU32,
    log: Mutex<Vec<String>>
}

impl Downloads {
    fn log(&self, line: String) {
        let mut log = self.log.lock().unwrap();
        log.push(line);
        // keep the snapshots small
        if log.len() > 5 {
            log.remove(0);
        }
    }
}

#[derive(Default, Serialize)]
pub struct Idle;
#[derive(Default, Serialize)]
pub struct Downloading { url: String, attempt: u32 }
#[derive(Default, Serialize)]
pub struct Verifying;
#[derive(Default, Serialize)]
pub struct Online;
#[derive(Default, Serialize)]
pub struct Offline { since_tick: u64 }

#[derive(Clone, Debug, Serialize)]
pub struct Fetch { url: String, size_kb: u32 }
#[derive(Clone, Debug, Serialize)]
pub struct Finished { ok: bool }
#[derive(Clone, Debug, Serialize)]
pub struct Verified;
#[derive(Clone, Debug, Serialize)]
pub struct Retry;
#[derive(Clone, Debug, Serialize)]
pub struct LinkDown { tick: u64 }
#[derive(Clone, Debug, Serialize)]
pub struct LinkUp;

#[finny_fsm]
fn build_downloader(mut fsm: FsmAsyncBuilder<Downloader, Downloads>) -> BuiltFsm {
    fsm.serde();
    // the regions' actions run concurrently
    fsm.concurrent_regions();
    fsm.initial_states::<(Idle, Online)>();

    fsm.state::<Idle>()
        .on_event::<Fetch>()
        .transition_to::<Downloading>()
        .action(async |ev, ctx, _, downloading| {
            downloading.url = ev.url.clone();
            downloading.attempt = 1;
            ctx.log(format!("fetching {} ({} kB)", ev.url, ev.size_kb));
            tokio::time::sleep(Duration::from_millis(ev.size_kb as u64)).await;
        });

    fsm.state::<Downloading>()
        .on_event::<Retry>()
        .internal_transition()
        .guard(|_, _, states| {
            let d: &Downloading = states.as_ref();
            d.attempt < 3
        })
        .action(async |_, ctx, downloading| {
            downloading.attempt += 1;
            ctx.log(format!("retrying {}, attempt {}", downloading.url, downloading.attempt));
        });

    fsm.state::<Downloading>()
        .on_event::<Finished>()
        .transition_to::<Verifying>()
        .guard(|ev, _, _| ev.ok);

    fsm.state::<Downloading>()
        .on_event::<Finished>()
        .transition_to::<Idle>()
        .action(async |_, ctx, _, _| { ctx.failed.fetch_add(1, Ordering::SeqCst); });

    fsm.state::<Verifying>()
        .on_entry_start_timer(|_ctx, timer| {
            timer.timeout = Duration::from_millis(700);
            timer.cancel_on_state_exit = true;
        }, |_ctx, _state| Some(Verified.into()))
        .with_timer_ty::<VerifyTimer>();

    fsm.state::<Verifying>()
        .on_event::<Verified>()
        .transition_to::<Idle>()
        .action(async |_, ctx, _, _| {
            ctx.completed.fetch_add(1, Ordering::SeqCst);
            ctx.log("verified".into());
        });

    fsm.state::<Online>()
        .on_event::<LinkDown>()
        .transition_to::<Offline>()
        .action(async |ev, _, _, offline| { offline.since_tick = ev.tick; });

    fsm.state::<Offline>()
        .on_event::<LinkUp>()
        .transition_to::<Online>();

    fsm.build()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default());
    let server = inspector.clone();
    tokio::spawn(async move {
        if let Err(e) = server.serve("127.0.0.1:7878").await {
            eprintln!("The inspector failed: {}", e);
        }
    });
    println!("The Finny inspector is running at http://127.0.0.1:7878");

    let mut fsm = Downloader::new_with(Downloads::default(), FsmEventQueueVec::new(),
        inspector.attach::<Downloader>("downloader"), TimersTokio::new())?;
    fsm.start().await?;

    // events from another task
    let (tx, mut rx) = tokio::sync::mpsc::channel::<DownloaderEvents>(16);
    tokio::spawn(async move {
        let files = ["report.pdf", "photo.jpg", "backup.tar", "notes.txt"];
        for tick in 0u64.. {
            tokio::time::sleep(Duration::from_millis(900)).await;
            let file = files[(tick as usize / 5) % files.len()];
            let event: DownloaderEvents = match tick % 10 {
                0 | 5 => Fetch { url: format!("https://example.com/{}", file), size_kb: 120 + (tick % 7) as u32 * 40 }.into(),
                1 | 2 | 3 => Retry.into(),
                4 => Finished { ok: tick % 20 != 4 }.into(),
                6 => LinkDown { tick }.into(),
                7 => Finished { ok: true }.into(),
                8 => LinkUp.into(),
                _ => Retry.into()
            };
            if tx.send(event).await.is_err() {
                break;
            }
        }
    });

    fsm.run(&mut rx).await
}
