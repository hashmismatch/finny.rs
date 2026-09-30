use axum::{body::Body, http::{Request, StatusCode}};
use finny::{FsmEventQueueVec, FsmFactory, FsmResult, FsmTimersNull, decl::{BuiltFsm, FsmAsyncBuilder, FsmBuilder}, finny_fsm, inspect::{chain::InspectChain, null::InspectNull}};
use finny_inspect_web::{Inspector, InspectorConfig, snapshot::{EventRecord, TraceKind}};
use http_body_util::BodyExt;
use serde::Serialize;
use serde_json::{Value, json};
use tower::ServiceExt;

#[derive(Default, Serialize)]
pub struct Idle;
#[derive(Default, Serialize)]
pub struct Done { n: u32 }
#[derive(Clone, Debug, Serialize)]
pub struct Go { ok: bool }
#[derive(Clone, Debug, Serialize)]
pub struct Unknown;
#[derive(Clone, Debug, Serialize)]
pub struct Next;
#[derive(Default, Serialize)]
pub struct Ctx { gos: u32 }

#[finny_fsm]
fn build_machine(mut fsm: FsmBuilder<Machine, Ctx>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<Idle>();
    fsm.state::<Idle>()
        .on_event::<Go>()
        .transition_to::<Sub>()
        .guard(|ev, _, _| ev.ok)
        .action(|_, ctx, _, _| { ctx.gos += 1; });
    fsm.sub_machine::<Sub>()
        .on_event::<Go>()
        .transition_to::<Done>();
    fsm.state::<Done>()
        .on_event::<Unknown>()
        .internal_transition()
        .guard(|_, _, _| false);
    fsm.build()
}

#[derive(Default, Serialize)]
pub struct SubA;
#[derive(Default, Serialize)]
pub struct SubB;

#[finny_fsm]
fn build_sub(mut fsm: FsmBuilder<Sub, ()>) -> BuiltFsm {
    fsm.serde();
    fsm.initial_state::<SubA>();
    fsm.state::<SubA>().on_event::<Next>().transition_to::<SubB>();
    fsm.state::<SubB>();
    fsm.build()
}

#[derive(Default)]
pub struct Plain;

#[finny_fsm]
fn build_plain(mut fsm: FsmBuilder<PlainMachine, ()>) -> BuiltFsm {
    fsm.initial_state::<Plain>();
    fsm.state::<Plain>();
    fsm.build()
}

fn machine(inspector: &Inspector, name: &str) -> FsmResult<finny::FsmFrontend<Machine, FsmEventQueueVec<Machine>, finny_inspect_web::InspectWeb, FsmTimersNull>> {
    Machine::new_with(Ctx::default(), FsmEventQueueVec::new(), inspector.attach::<Machine>(name), FsmTimersNull)
}

#[test]
fn test_a_snapshot_after_each_event() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default());
    let mut fsm = machine(&inspector, "main")?;

    fsm.start()?;
    fsm.dispatch(Go { ok: true })?;
    let ev: SubEvents = Next.into();
    fsm.dispatch(ev)?;

    let snapshots = inspector.snapshots("main").unwrap();
    assert_eq!(vec![1, 2, 3], snapshots.iter().map(|s| s.seq).collect::<Vec<_>>());
    assert!(matches!(snapshots[0].event, EventRecord::Start));

    // the guarded transition into the sub-machine
    let go = &snapshots[1];
    assert!(go.error.is_none());
    match &go.event {
        EventRecord::Event { name, value } => {
            assert_eq!("Go", name);
            assert_eq!(Some(json!({ "Go": { "ok": true } })), *value);
        },
        e => panic!("{:?}", e)
    }
    let kinds: Vec<String> = go.trace.iter().map(|t| match &t.kind {
        TraceKind::Guard { result, .. } => format!("{}:guard {}", t.depth, result),
        TraceKind::Transition { .. } => format!("{}:transition", t.depth),
        TraceKind::StateExit { state } => format!("{}:exit {}", t.depth, state),
        TraceKind::StateEnter { state } => format!("{}:enter {}", t.depth, state),
        TraceKind::Action { .. } => format!("{}:action", t.depth),
        TraceKind::SubMachine { .. } => format!("{}:sub", t.depth),
        TraceKind::Event { .. } => format!("{}:event", t.depth),
        k => format!("{}:{:?}", t.depth, k)
    }).collect();
    assert_eq!(vec![
        "0:guard true", "0:transition", "1:exit Idle", "1:action", "1:enter Sub",
        // the sub-machine is started
        "0:sub", "1:event", "2:transition", "3:enter SubA"
    ], kinds);

    assert_eq!(json!([
        { "path": [], "states": ["Sub"] },
        { "path": [std::any::type_name::<Sub>()], "states": ["SubA"] }
    ]), serde_json::to_value(&go.active).unwrap());
    assert_eq!(json!({ "gos": 1 }), go.values.as_ref().unwrap()["context"]);

    // forwarded into the sub-machine, which transitions
    let next = &snapshots[2];
    assert_eq!(Some("SubB".to_string()), next.active[1].states[0]);
    assert_eq!(json!(["SubB"]), next.values.as_ref().unwrap()["states"]["sub"]["current_states"]);
    Ok(())
}

#[test]
fn test_failed_dispatches_are_recorded() -> FsmResult<()> {
    let inspector = Inspector::default();
    let mut fsm = machine(&inspector, "main")?;
    fsm.start()?;

    // the guard rejects it
    assert!(fsm.dispatch(Go { ok: false }).is_err());
    let snapshots = inspector.snapshots("main").unwrap();
    let rejected = snapshots.last().unwrap();
    assert_eq!(Some("NoTransition".to_string()), rejected.error);
    assert!(matches!(rejected.trace[0].kind, TraceKind::Guard { result: false, .. }));
    assert_eq!(vec![Some("Idle".to_string())], rejected.active[0].states);
    Ok(())
}

#[test]
fn test_the_history_is_bounded() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default().history_len(5));
    let mut fsm = machine(&inspector, "main")?;
    fsm.start()?;
    for _ in 0..10 {
        let _ = fsm.dispatch(Go { ok: false });
    }

    let snapshots = inspector.snapshots("main").unwrap();
    assert_eq!(vec![7, 8, 9, 10, 11], snapshots.iter().map(|s| s.seq).collect::<Vec<_>>());
    Ok(())
}

#[test]
fn test_instances() -> FsmResult<()> {
    let inspector = Inspector::default();
    let a = machine(&inspector, "fsm")?;
    let b = machine(&inspector, "fsm")?;
    // chained with another inspector, and without the serialization
    let mut plain = PlainMachine::new_with((), FsmEventQueueVec::new(),
        InspectChain::new_pair(inspector.attach::<PlainMachine>("plain"), InspectNull::new()), FsmTimersNull)?;
    plain.start()?;

    let instances = inspector.instances();
    assert_eq!(vec!["fsm", "fsm#2", "plain"], instances.iter().map(|i| i.id.as_str()).collect::<Vec<_>>());
    assert!(instances.iter().all(|i| i.attached));
    assert_eq!("PlainMachine", instances[2].fsm);

    let snapshots = inspector.snapshots("plain").unwrap();
    assert!(snapshots[0].values.is_none());
    assert_eq!(vec![Some("Plain".to_string())], snapshots[0].active[0].states);

    drop(a);
    assert!(!inspector.instances()[0].attached);
    drop(b);
    Ok(())
}

async fn get(inspector: &Inspector, uri: &str) -> (StatusCode, String) {
    let response = inspector.router().oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn test_api() -> FsmResult<()> {
    let inspector = Inspector::default();
    let mut fsm = machine(&inspector, "main")?;
    fsm.start()?;
    fsm.dispatch(Go { ok: true })?;

    let (status, body) = get(&inspector, "/api/instances").await;
    assert_eq!(StatusCode::OK, status);
    let instances: Value = serde_json::from_str(&body).unwrap();
    assert_eq!("main", instances[0]["id"]);
    assert_eq!(2, instances[0]["last_seq"]);

    let (_, body) = get(&inspector, "/api/instances/main/meta").await;
    let meta: Value = serde_json::from_str(&body).unwrap();
    assert_eq!("Machine", meta["info"]["id"]);
    assert!(meta["plantuml"].as_str().unwrap().contains("Idle --> Sub : Go [guard]"));

    let (_, body) = get(&inspector, "/api/instances/main/snapshots").await;
    let snapshots: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(2, snapshots.as_array().unwrap().len());

    let (_, body) = get(&inspector, "/api/instances/main/snapshots?after=1").await;
    let snapshots: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json!([2]), json!(snapshots.as_array().unwrap().iter().map(|s| s["seq"].clone()).collect::<Vec<_>>()));

    let (status, _) = get(&inspector, "/api/instances/nope/meta").await;
    assert_eq!(StatusCode::NOT_FOUND, status);

    let (status, body) = get(&inspector, "/").await;
    assert_eq!(StatusCode::OK, status);
    assert!(body.contains("<html"));
    let (status, _) = get(&inspector, "/assets/../Cargo.toml").await;
    assert_eq!(StatusCode::NOT_FOUND, status);
    Ok(())
}

/// Reads the server-sent events until `n` of them were received.
async fn read_events(body: Body, n: usize) -> Vec<(String, String, String)> {
    let mut stream = body.into_data_stream();
    let mut buffer = String::new();
    let mut events = vec![];
    use futures_util::StreamExt;
    while events.len() < n {
        let chunk = tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await
            .expect("timeout").expect("the stream ended").unwrap();
        buffer.push_str(std::str::from_utf8(&chunk).unwrap());
        while let Some(end) = buffer.find("\n\n") {
            let raw: String = buffer.drain(..end + 2).collect();
            let (mut event, mut id, mut data) = (String::new(), String::new(), String::new());
            for line in raw.lines() {
                if let Some(v) = line.strip_prefix("event: ") { event = v.into(); }
                if let Some(v) = line.strip_prefix("id: ") { id = v.into(); }
                if let Some(v) = line.strip_prefix("data: ") { data = v.into(); }
            }
            if !event.is_empty() {
                events.push((event, id, data));
            }
        }
    }
    events
}

#[tokio::test]
async fn test_stream_replays_and_follows() -> FsmResult<()> {
    let inspector = Inspector::default();
    let mut fsm = machine(&inspector, "main")?;
    fsm.start()?;
    fsm.dispatch(Go { ok: false }).ok();
    fsm.dispatch(Go { ok: false }).ok();

    // a reconnecting client already has the first snapshot
    let request = Request::get("/api/instances/main/stream").header("Last-Event-ID", "1").body(Body::empty()).unwrap();
    let response = inspector.router().oneshot(request).await.unwrap();
    assert_eq!(StatusCode::OK, response.status());

    fsm.dispatch(Go { ok: true })?;

    let events = read_events(response.into_body(), 3).await;
    assert_eq!(vec!["2", "3", "4"], events.iter().map(|e| e.1.as_str()).collect::<Vec<_>>());
    assert!(events.iter().all(|e| e.0 == "snapshot"));
    let last: Value = serde_json::from_str(&events[2].2).unwrap();
    assert_eq!(4, last["seq"]);
    assert!(last["error"].is_null());
    Ok(())
}

#[tokio::test]
async fn test_instances_stream() -> FsmResult<()> {
    let inspector = Inspector::default();
    let _a = machine(&inspector, "a")?;
    let response = inspector.router().oneshot(Request::get("/api/instances/stream").body(Body::empty()).unwrap()).await.unwrap();
    let _b = machine(&inspector, "b")?;

    let events = read_events(response.into_body(), 2).await;
    let first: Value = serde_json::from_str(&events[0].2).unwrap();
    let second: Value = serde_json::from_str(&events[1].2).unwrap();
    assert_eq!(1, first.as_array().unwrap().len());
    assert_eq!(2, second.as_array().unwrap().len());
    Ok(())
}

#[derive(Default, Serialize)]
pub struct AsyncCtx { hits: std::sync::atomic::AtomicU32 }
#[derive(Default, Serialize)]
pub struct Left;
#[derive(Default, Serialize)]
pub struct LeftDone;
#[derive(Default, Serialize)]
pub struct Right;
#[derive(Default, Serialize)]
pub struct RightDone;

#[finny_fsm]
fn build_async_machine(mut fsm: FsmAsyncBuilder<AsyncMachine, AsyncCtx>) -> BuiltFsm {
    fsm.serde();
    fsm.concurrent_regions();
    fsm.initial_states::<(Left, Right)>();
    fsm.state::<Left>().on_event::<Next>().transition_to::<LeftDone>()
        .action(async |_, ctx, _, _| {
            tokio::task::yield_now().await;
            ctx.hits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
    fsm.state::<LeftDone>();
    fsm.state::<Right>().on_event::<Next>().transition_to::<RightDone>()
        .action(async |_, ctx, _, _| {
            ctx.hits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
    fsm.state::<RightDone>();
    fsm.build()
}

#[tokio::test]
async fn test_async_concurrent_regions() -> FsmResult<()> {
    use finny::FsmAsyncFactory;

    let inspector = Inspector::default();
    let mut fsm = AsyncMachine::new_with(AsyncCtx::default(), FsmEventQueueVec::new(), inspector.attach::<AsyncMachine>("async"), FsmTimersNull)?;
    fsm.start().await?;
    fsm.dispatch(Next).await?;

    let snapshots = inspector.snapshots("async").unwrap();
    let next = snapshots.last().unwrap();
    assert_eq!(vec![Some("LeftDone".to_string()), Some("RightDone".to_string())], next.active[0].states);
    assert_eq!(json!({ "hits": 2 }), next.values.as_ref().unwrap()["context"]);
    // both regions' transitions, each with its exit, action and entry
    let transitions = next.trace.iter().filter(|t| matches!(t.kind, TraceKind::Transition { .. })).count();
    let actions = next.trace.iter().filter(|t| matches!(t.kind, TraceKind::Action { .. })).count();
    assert_eq!((2, 2), (transitions, actions));
    Ok(())
}
