//! The inspected FSM instances and their recorded history.

use std::{collections::VecDeque, sync::{Arc, Mutex, MutexGuard, Weak}, time::{SystemTime, UNIX_EPOCH}};

use finny::meta::{FsmMeta, plantuml::to_plantuml};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::{InspectWeb, InspectorConfig, snapshot::{ActiveStates, Snapshot}};

/// Collects the snapshots of the attached FSMs and serves them to the web frontend.
///
/// ```no_run
/// # use finny_inspect_web::{Inspector, InspectorConfig};
/// let inspector = Inspector::new(InspectorConfig::default().history_len(200));
/// let addr = inspector.spawn("127.0.0.1:7878").unwrap();
/// println!("Inspect the FSMs at http://{}", addr);
/// ```
#[derive(Clone)]
pub struct Inspector {
    pub(crate) registry: Arc<Registry>
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new(InspectorConfig::default())
    }
}

impl Inspector {
    pub fn new(config: InspectorConfig) -> Self {
        let (changes, _) = broadcast::channel(16);
        Inspector {
            registry: Arc::new(Registry {
                config,
                instances: Mutex::new(Vec::new()),
                changes
            })
        }
    }

    pub fn config(&self) -> &InspectorConfig {
        &self.registry.config
    }

    /// Registers an FSM instance and returns its inspector, to be passed to the FSM's
    /// `new_with` constructor, possibly chained with other inspectors with `InspectChain`. The
    /// name identifies the instance in the frontend, a suffix is added when it's already taken.
    pub fn attach<F: FsmMeta>(&self, name: &str) -> InspectWeb {
        let info = F::fsm_info();
        let meta_json = serde_json::to_string(&MetaResponse { plantuml: to_plantuml(&info), info: &info })
            .expect("The FSM's description can always be serialized");

        let instance = {
            let mut instances = lock(&self.registry.instances);
            let mut id = name.to_string();
            let mut n = 1;
            while instances.iter().any(|i| i.id == id) {
                n += 1;
                id = format!("{}#{}", name, n);
            }

            let capacity = self.registry.config.history_len.max(16);
            let (tx, _) = broadcast::channel(capacity);
            let instance = Arc::new(FsmInstance {
                id,
                fsm_id: info.id.clone(),
                type_name: info.type_name.clone(),
                meta_json: meta_json.into(),
                history_len: self.registry.config.history_len,
                attached_at_ms: now_ms(),
                state: Mutex::new(InstanceState {
                    ring: VecDeque::new(),
                    next_seq: 1,
                    active: Vec::new(),
                    attached: true
                }),
                tx,
                registry: Arc::downgrade(&self.registry)
            });
            instances.push(instance.clone());
            instance
        };

        self.registry.notify_changed();
        InspectWeb::new_root(instance)
    }

    /// The attached instances, in the order of attaching.
    pub fn instances(&self) -> Vec<InstanceSummary> {
        lock(&self.registry.instances).iter().map(|i| i.summary()).collect()
    }

    /// The snapshots that are currently kept for the instance.
    pub fn snapshots(&self, instance_id: &str) -> Option<Vec<Snapshot>> {
        let instance = self.registry.instance(instance_id)?;
        let state = instance.lock_state();
        Some(state.ring.iter().map(|s| s.snapshot.clone()).collect())
    }
}

#[derive(Serialize)]
struct MetaResponse<'a> {
    info: &'a finny::meta::FsmInfo,
    plantuml: String
}

pub(crate) struct Registry {
    pub(crate) config: InspectorConfig,
    instances: Mutex<Vec<Arc<FsmInstance>>>,
    /// Notified when the list of instances changes.
    pub(crate) changes: broadcast::Sender<()>
}

impl Registry {
    pub(crate) fn instance(&self, id: &str) -> Option<Arc<FsmInstance>> {
        lock(&self.instances).iter().find(|i| i.id == id).cloned()
    }

    pub(crate) fn summaries(&self) -> Vec<InstanceSummary> {
        lock(&self.instances).iter().map(|i| i.summary()).collect()
    }

    fn notify_changed(&self) {
        let _ = self.changes.send(());
    }
}

/// An attached FSM instance, as listed by the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct InstanceSummary {
    pub id: String,
    /// The id of the FSM's type.
    pub fsm: String,
    pub type_name: String,
    /// False once the FSM, and its inspector, were dropped. Its history remains available.
    pub attached: bool,
    pub attached_at_ms: u64,
    pub last_seq: Option<u64>,
    pub snapshots: usize,
    pub history_len: usize
}

/// A snapshot along with its serialized form, that is shared by all the connected clients.
#[derive(Clone)]
pub(crate) struct StoredSnapshot {
    pub(crate) seq: u64,
    pub(crate) json: Arc<str>,
    snapshot: Snapshot
}

pub(crate) struct FsmInstance {
    pub(crate) id: String,
    fsm_id: String,
    type_name: String,
    /// `{ info, plantuml }`
    pub(crate) meta_json: Arc<str>,
    history_len: usize,
    attached_at_ms: u64,
    state: Mutex<InstanceState>,
    /// The published snapshots. Sent while holding the state's lock, so a subscriber that
    /// copies the ring under the same lock sees every snapshot exactly once.
    tx: broadcast::Sender<StoredSnapshot>,
    registry: Weak<Registry>
}

pub(crate) struct InstanceState {
    pub(crate) ring: VecDeque<StoredSnapshot>,
    next_seq: u64,
    /// The last known current states of the machine and of its sub-machines. A sub-machine's
    /// states only change while it handles an event, which reports them.
    active: Vec<ActiveStates>,
    attached: bool
}

impl FsmInstance {
    pub(crate) fn lock_state(&self) -> MutexGuard<'_, InstanceState> {
        lock(&self.state)
    }

    fn summary(&self) -> InstanceSummary {
        let state = self.lock_state();
        InstanceSummary {
            id: self.id.clone(),
            fsm: self.fsm_id.clone(),
            type_name: self.type_name.clone(),
            attached: state.attached,
            attached_at_ms: self.attached_at_ms,
            last_seq: state.ring.back().map(|s| s.seq),
            snapshots: state.ring.len(),
            history_len: self.history_len
        }
    }

    pub(crate) fn set_active(&self, path: &[String], states: Vec<Option<String>>) {
        let mut state = self.lock_state();
        match state.active.iter_mut().find(|a| a.path == path) {
            Some(a) => a.states = states,
            None => {
                state.active.push(ActiveStates { path: path.to_vec(), states });
                // the parents first
                state.active.sort_by(|a, b| a.path.len().cmp(&b.path.len()).then_with(|| a.path.cmp(&b.path)));
            }
        }
    }

    /// Records the snapshot, `seq` is assigned here. The current states of the sub-machines are
    /// added.
    pub(crate) fn publish(&self, mut snapshot: Snapshot) {
        let mut state = self.lock_state();
        snapshot.seq = state.next_seq;
        state.next_seq += 1;
        snapshot.active = state.active.clone();

        let json = match serde_json::to_string(&snapshot) {
            Ok(json) => json,
            Err(e) => {
                tracing::warn!(error = %e, instance = %self.id, "Failed to serialize the FSM's snapshot");
                return;
            }
        };

        let stored = StoredSnapshot { seq: snapshot.seq, json: json.into(), snapshot };
        if self.history_len > 0 {
            while state.ring.len() >= self.history_len {
                state.ring.pop_front();
            }
            state.ring.push_back(stored.clone());
        }
        // no subscribers is fine
        let _ = self.tx.send(stored);
    }

    /// The snapshots after `after`, and the receiver of the upcoming ones.
    pub(crate) fn subscribe(&self, after: Option<u64>) -> (Vec<StoredSnapshot>, broadcast::Receiver<StoredSnapshot>) {
        let state = self.lock_state();
        let rx = self.tx.subscribe();
        let backlog = state.ring.iter().filter(|s| Some(s.seq) > after).cloned().collect();
        (backlog, rx)
    }

    pub(crate) fn detach(&self) {
        self.lock_state().attached = false;
        if let Some(registry) = self.registry.upgrade() {
            registry.notify_changed();
        }
    }
}

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic in an FSM's action while we hold the lock doesn't corrupt the history.
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}
