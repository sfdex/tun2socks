use std::collections::HashMap;
use crate::thread_pool::worker::Worker;

static mut ROUTER: Option<HashMap<String, Worker>> = None;

pub fn insert(key: &str, worker: Worker) {
    unsafe {
        if ROUTER.is_none() {
            ROUTER = Some(HashMap::new());
        }
        
        ROUTER.as_mut().unwrap().insert(key.to_string(), worker);
    }
}

pub fn get(key: &str) -> Option<&'static Worker> {
    unsafe {
        if ROUTER.is_none() {
            ROUTER = Some(HashMap::new());
            return None;
        }

        ROUTER.as_ref().unwrap().get(key)
    }
}

pub fn delete(key: &str) {
    unsafe {
        if ROUTER.is_none() {
            ROUTER = Some(HashMap::new());
            return;
        }

        ROUTER.as_mut().unwrap().remove(key);
    }
}