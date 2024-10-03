use actix_session::Session;
use actix_session::SessionExt;
use actix_session::{SessionGetError, SessionInsertError};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;
pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";
    pub fn renew(&self) {
        self.0.renew();
    }
    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }
    pub fn get_user_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get(Self::USER_ID_KEY)
    }
    pub fn log_out(self) {
        self.0.purge()
    }
    const ADMIN_ID_KEY: &'static str = "admin_id";
    pub fn admin_renew(&self) {
        self.0.renew();
    }
    pub fn insert_admin_id(&self, admin_id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::ADMIN_ID_KEY, admin_id)
    }
    pub fn get_admin_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get(Self::ADMIN_ID_KEY)
    }
    pub fn admin_log_out(self) {
        self.0.purge()
    }
}

impl FromRequest for TypedSession {
    // This is a complicated way of saying
    // "We return the same error returned by the
    // implementation of `FromRequest` for `Session`".
    type Error = <Session as FromRequest>::Error;
    // Rust does not yet support the `async` syntax in traits.
    // From request expects a `Future` as return type to allow for extractors
    // that need to perform asynchronous operations (e.g. a HTTP call)
    // We do not have a `Future`, because we don't perform any I/O,
    // so we wrap `TypedSession` into `Ready` to convert it into a `Future` that
    // resolves to the wrapped value the first time it's polled by the executor.
    type Future = Ready<Result<TypedSession, Self::Error>>;
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
