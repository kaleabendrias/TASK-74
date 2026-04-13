use gloo_storage::{LocalStorage, Storage};

const SESSION_KEY: &str = "tourism_session_token";
const CSRF_KEY: &str = "tourism_csrf_token";

pub fn store_tokens(session_token: &str, csrf_token: &str) {
    LocalStorage::set(SESSION_KEY, session_token).ok();
    LocalStorage::set(CSRF_KEY, csrf_token).ok();
}

pub fn get_session_token() -> Option<String> {
    LocalStorage::get::<String>(SESSION_KEY).ok()
}

pub fn get_csrf_token() -> Option<String> {
    LocalStorage::get::<String>(CSRF_KEY).ok()
}

pub fn clear_tokens() {
    LocalStorage::delete(SESSION_KEY);
    LocalStorage::delete(CSRF_KEY);
}

pub fn is_authenticated() -> bool {
    get_session_token().is_some()
}
