## Finny web inspector

A web frontend for inspecting [Finny](https://crates.io/crates/finny) state machines while they
run:

* **A statechart of the machine**, in the style of PlantUML. It shows the regions, the
  sub-machines as composite states, the timers and the internal transitions. It is laid out with
  ELK, and you can drag the states and zoom. The current states are highlighted, as are the
  transitions of the inspected event and any guards that rejected it.
* **The history**. The backend keeps a snapshot after each event that the machine has fully
  handled, the last 100 by default. A UI that connects later can still trace the history back.
  A timeline scrubs through the snapshots, or you can follow the live ones.
* **Panels for inspecting**:
  * the values of the context and the states, with the changes since the previous snapshot
    highlighted
  * the event's payload and the dispatch result
  * the trace of the dispatch: guards, exits, actions, entries, sub-machines and timers
  * a searchable history, which a click on a transition filters

The backend is built with axum. The frontend has no build step: it's plain ES modules with
Preact, htm and Cytoscape.js, and all the files are embedded into the binary.

### Usage

```toml
[dependencies]
finny = { version = "0.3", features = ["serde"] }
finny_inspect_web = "0.1"
serde = { version = "1", features = ["derive"] }
```

```rust
use finny::{FsmEventQueueVec, FsmFactory, FsmResult, decl::{BuiltFsm, FsmBuilder}, finny_fsm, timers::std::TimersStd};
use finny_inspect_web::{Inspector, InspectorConfig};
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Ctx { count: usize }
#[derive(Default, Serialize)]
pub struct Idle;
#[derive(Clone, Debug, Serialize)]
pub struct Poke;

#[finny_fsm]
fn my_fsm(mut fsm: FsmBuilder<MyFsm, Ctx>) -> BuiltFsm {
    // Opt into the serialization of the context, the states and the events.
    fsm.serde();
    fsm.initial_state::<Idle>();
    fsm.state::<Idle>()
        .on_event::<Poke>()
        .self_transition()
        .action(|_, ctx, _| { ctx.count += 1; });
    fsm.build()
}

fn main() -> FsmResult<()> {
    let inspector = Inspector::new(InspectorConfig::default().history_len(100));
    // A background thread with its own runtime. Or `inspector.serve(addr).await`, or nest
    // `inspector.router()` into an existing axum application.
    inspector.spawn("127.0.0.1:7878").expect("Failed to start the inspector");

    let mut fsm = MyFsm::new_with(Ctx::default(), FsmEventQueueVec::new(),
        inspector.attach::<MyFsm>("main"), TimersStd::new())?;
    fsm.start()?;
    fsm.dispatch(Poke)?;
    Ok(())
}
```

Open http://127.0.0.1:7878. To combine the web inspector with other inspectors, such as
`InspectTracing`, use `finny::inspect::chain::InspectChain`.

* **Serialization.** `fsm.serde()` needs `serde::Serialize` on the context, the states and the
  events. A sub-machine has to opt in too. FSMs that don't opt in are still inspected: you get
  the diagram, the current states, the traces and the history, just without the values.
* **Async FSMs.** They work the same way. Their context is shared through an `Arc`, and its
  interior mutability (atomics, mutexes) serializes with serde.
* **Overhead.** Each dispatched event is serialized once, on the FSM's thread, and shared with
  all the connected clients. The inspector is meant for development.

### Examples

```sh
cargo run -p finny_inspect_web --example traffic_light   # sync, timers, regions, a sub-machine
cargo run -p finny_inspect_web --example async_demo      # async, concurrent regions
```

### API

| Endpoint | |
|---|---|
| `GET /api/instances` | The attached FSM instances. |
| `GET /api/instances/stream` | Server-sent `instances` events, whenever the list changes. |
| `GET /api/instances/{id}/meta` | The FSM's description, `{ info, plantuml }`. |
| `GET /api/instances/{id}/snapshots?after={seq}` | The kept snapshots. |
| `GET /api/instances/{id}/stream?after={seq}` | Server-sent `snapshot` events: the kept snapshots after `seq` (or the `Last-Event-ID` header), followed by the new ones. |

### Working on the frontend

Set `FINNY_INSPECT_ASSETS_DIR=finny_inspect_web/assets` and the files are served from the disk,
so a reload picks up the changes. The vendored libraries and their versions are listed in
[`assets/vendor/README.md`](assets/vendor/README.md). Note that elkjs is EPL-2.0 licensed.

License: MIT OR Apache-2.0, except for the vendored libraries in `assets/vendor`, which keep their
own licenses.
