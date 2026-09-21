use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureKind {
    Screen,
    Select,
    Window,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CaptureState {
    Capturing,
    Ready { path: String },
    Cancelled,
    Failed { error: String },
}

#[derive(Clone, Debug, Serialize)]
pub struct CaptureTask {
    pub id: String,
    pub kind: CaptureKind,
    #[serde(flatten)]
    pub state: CaptureState,
}

pub struct TaskLog {
    prefix: String,
    sequence: u64,
    tasks: VecDeque<CaptureTask>,
}

impl Default for TaskLog {
    fn default() -> Self {
        Self {
            prefix: format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("System clock predates Unix epoch")
                    .as_nanos()
            ),
            sequence: 0,
            tasks: VecDeque::new(),
        }
    }
}

impl TaskLog {
    pub fn begin(&mut self, kind: CaptureKind) -> (CaptureTask, bool) {
        if let Some(task) = self
            .tasks
            .back()
            .filter(|task| task.state == CaptureState::Capturing)
        {
            return (task.clone(), false);
        }
        self.sequence += 1;
        let task = CaptureTask {
            id: format!("{}-{}", self.prefix, self.sequence),
            kind,
            state: CaptureState::Capturing,
        };
        if self.tasks.len() == 32 {
            self.tasks.pop_front();
        }
        self.tasks.push_back(task.clone());
        (task, true)
    }

    pub fn finish(&mut self, id: &str, state: CaptureState) -> Result<CaptureTask, String> {
        if state == CaptureState::Capturing {
            return Err("Expected a terminal capture state".into());
        }
        let task = self
            .tasks
            .iter_mut()
            .find(|task| task.id == id)
            .ok_or("Capture task not found")?;
        if task.state != CaptureState::Capturing {
            return Err("Capture task already finished".into());
        }
        task.state = state;
        Ok(task.clone())
    }

    pub fn get(&self, id: Option<&str>) -> Option<CaptureTask> {
        match id {
            Some(id) => self.tasks.iter().find(|task| task.id == id).cloned(),
            None => self.tasks.back().cloned(),
        }
    }
}

#[derive(Default)]
pub struct CaptureTasks(pub Mutex<TaskLog>);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ipc_contract_rejects_unknown_modes_and_exposes_terminal_state() {
        assert!(serde_json::from_str::<CaptureKind>("\"unknown\"").is_err());
        let task = CaptureTask {
            id: "test-task".into(),
            kind: CaptureKind::Select,
            state: CaptureState::Cancelled,
        };
        assert_eq!(
            serde_json::to_value(task).unwrap(),
            serde_json::json!({
                "id": "test-task", "kind": "select", "status": "cancelled"
            })
        );
        assert!(TaskLog::default().get(Some("missing")).is_none());
    }
    #[test]
    fn duplicate_requests_share_active_task() {
        let mut log = TaskLog::default();
        let (first, started) = log.begin(CaptureKind::Select);
        assert!(started);
        let (second, started) = log.begin(CaptureKind::Screen);
        assert!(!started);
        assert_eq!(first.id, second.id);
        assert_eq!(second.kind, CaptureKind::Select);
    }
    #[test]
    fn terminal_states_allow_next_task_and_remain_queryable() {
        let mut log = TaskLog::default();
        for state in [
            CaptureState::Cancelled,
            CaptureState::Failed {
                error: "denied".into(),
            },
            CaptureState::Ready {
                path: "/tmp/capture.png".into(),
            },
        ] {
            let (task, started) = log.begin(CaptureKind::Window);
            assert!(started);
            log.finish(&task.id, state.clone()).unwrap();
            assert_eq!(log.get(Some(&task.id)).unwrap().state, state);
        }
    }
    #[test]
    fn stale_completion_cannot_overwrite_result() {
        let mut log = TaskLog::default();
        let (old, _) = log.begin(CaptureKind::Screen);
        log.finish(&old.id, CaptureState::Cancelled).unwrap();
        let (new, _) = log.begin(CaptureKind::Screen);
        assert_ne!(old.id, new.id);
        assert!(log
            .finish(
                &old.id,
                CaptureState::Failed {
                    error: "late".into()
                }
            )
            .is_err());
        assert_eq!(log.get(None).unwrap().id, new.id);
        assert_eq!(log.get(None).unwrap().state, CaptureState::Capturing);
    }
    #[test]
    fn retention_is_bounded_without_reusing_ids() {
        let mut log = TaskLog::default();
        let (first, _) = log.begin(CaptureKind::Screen);
        log.finish(&first.id, CaptureState::Cancelled).unwrap();
        for _ in 0..40 {
            let (task, _) = log.begin(CaptureKind::Screen);
            log.finish(&task.id, CaptureState::Cancelled).unwrap();
        }
        assert_eq!(log.tasks.len(), 32);
        assert!(log.get(Some(&first.id)).is_none());
    }
}
