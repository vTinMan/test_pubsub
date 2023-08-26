use log::debug;
use once_cell::sync::Lazy;
use uuid::Uuid;
use std::{ time::Instant, sync::Mutex, sync::MutexGuard,
           collections::HashMap, collections::linked_list::LinkedList };

static TIMEOUT: u64 = 30;
static DEAD_TIMEOUT: u64 = 90;

#[derive(Clone, PartialEq)]
pub enum StateKind {
    Pending,
    Done,
    Timeout
}

#[derive(Clone)]
pub struct SessionData {
    pub state: StateKind,
    pub message: Option<String>,
    created_at: Instant,
}

static SESSION_HM: Lazy<Mutex<HashMap<String, Uuid>>> =
    Lazy::new(|| { Mutex::new(HashMap::new()) });
fn get_sessions()-> MutexGuard<'static, HashMap<String, Uuid>> {
    SESSION_HM.lock().unwrap()
}


static STATE_HM: Lazy<Mutex<HashMap<Uuid, SessionData>>> =
    Lazy::new(|| { Mutex::new(HashMap::new()) });
fn get_states()-> MutexGuard<'static, HashMap<Uuid, SessionData>> {
    STATE_HM.lock().unwrap()
}


static STATE_LIST: Lazy<Mutex<LinkedList<Uuid>>> =
    Lazy::new(|| { Mutex::new(LinkedList::new()) });
fn get_state_list()-> MutexGuard<'static, LinkedList<Uuid>> {
    STATE_LIST.lock().unwrap()
}



pub fn get_session(uid: &Uuid)-> Option<SessionData> {
    Some(get_states().get(uid)?.clone())
}

pub fn fetch_session_id(path: &String)-> Uuid {
    let current_uid = get_pending_session(path);

    match current_uid {
        Some(uid) => uid,
        None => create_pending_session(path)
    }
}

pub fn finalize_session(uid: &Uuid, message: &String) {
    let mut states = get_states();
    let session = match states.get_mut(uid) { Some(v) => v, None => return };

    session.state = StateKind::Done;
    session.message = Some(message.clone());
}

fn get_pending_session(path: &String)-> Option<Uuid> {
    let sessions = get_sessions();
    let current_uid = sessions.get(path)?;
    let states = get_states();
    let session = states.get(current_uid)?;
    let duration = session.created_at.elapsed().as_secs();
    if duration > TIMEOUT { return None }
    match session.state {
        StateKind::Pending => Some(current_uid.clone()),
        _ => None
    }
}

fn create_pending_session(path: &String)-> Uuid {
    let _ = check_expired_sessions();
    let new_uuid = Uuid::new_v4();
    get_sessions().insert(path.to_string(), new_uuid);
    let new_session = SessionData{ state: StateKind::Pending,
                                   message: None,
                                   created_at: Instant::now() };
    get_states().insert(new_uuid, new_session);
    get_state_list().push_back(new_uuid);
    new_uuid
}

// to free a memory
fn check_expired_sessions()-> u32 {
    let mut list = get_state_list();
    let mut states = get_states();
    let mut counter = 0;

    for _ in 0..5 {
        let uid = match list.front() { Some(v) => v.clone(), None => break };
        let session = match states.get_mut(&uid) { Some(v) => v, None => break };
        let duration = session.created_at.elapsed().as_secs();

        if duration < TIMEOUT {
            break;
        } else if duration < DEAD_TIMEOUT && session.state == StateKind::Pending {
            session.state = StateKind::Timeout;
        } else if duration >= DEAD_TIMEOUT {
            let _ = list.pop_front();
            states.remove(&uid);
	        counter += 1;
        }
    }
    debug!("  checking expired sessions");
    debug!("    list len: {}, {}", list.len(), counter);
    debug!("    hash len: {}", states.len());
    counter
}
