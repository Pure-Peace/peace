use std::{collections::HashMap, ops::Deref, sync::Arc};

use chrono::{DateTime, Utc};
use peace_pb::services::bancho_state_rpc::{
    ConnectionInfo, CreateUserSessionRequest, UserQuery,
};
use tokio::sync::RwLock;
use uuid::Uuid;

/// User object representing a connected client.
#[derive(Debug, Default, Clone)]
pub struct User {
    /// Unique session ID of the user.
    pub session_id: String,
    /// Unique user ID.
    pub user_id: i32,
    /// User's username.
    pub username: String,
    /// User's username in unicode, if available.
    pub username_unicode: Option<String>,
    /// User's privileges level.
    pub privileges: i32,
    /// Information about the user's connection.
    pub connection_info: ConnectionInfo,
    /// The timestamp of when the user was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp of when the user was last active.
    pub last_active: DateTime<Utc>,
}

impl User {
    /// Update the last active timestamp to the current time.
    #[inline]
    pub fn update_active(&mut self) {
        self.last_active = Utc::now();
    }
}

impl From<CreateUserSessionRequest> for User {
    /// Convert a `CreateUserSessionRequest` into a `User`.
    #[inline]
    fn from(res: CreateUserSessionRequest) -> Self {
        let CreateUserSessionRequest {
            user_id,
            username,
            username_unicode,
            privileges,
            connection_info,
        } = res;

        Self {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            username,
            username_unicode,
            privileges,
            connection_info: connection_info.unwrap(),
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }
}

/// A struct representing a collection of user sessions
#[derive(Debug, Default, Clone)]
pub struct UserSessions {
    /// A hash map that maps session IDs to user data
    pub indexed_by_session_id: HashMap<String, Arc<RwLock<User>>>,
    /// A hash map that maps user IDs to user data
    pub indexed_by_user_id: HashMap<i32, Arc<RwLock<User>>>,
    /// A hash map that maps usernames to user data
    pub indexed_by_username: HashMap<String, Arc<RwLock<User>>>,
    /// A hash map that maps Unicode usernames to user data
    pub indexed_by_username_unicode: HashMap<String, Arc<RwLock<User>>>,
    /// The number of user sessions in the collection
    pub len: usize,
}

impl UserSessions {
    /// Creates a new user session or updates an existing one with the given user data.
    ///
    /// # Arguments
    ///
    /// * `user` - The user data for the session
    ///
    /// # Returns
    ///
    /// The session ID of the created or updated session.
    #[inline]
    pub async fn create(&mut self, user: User) -> String {
        // Delete any existing session with the same user ID
        self.delete(&UserQuery::UserId(user.user_id)).await;

        // Clone the relevant data from the user struct
        let (session_id, user_id, username, username_unicode) = (
            user.session_id.clone(),
            user.user_id,
            user.username.clone(),
            user.username_unicode.clone(),
        );

        // Create a new pointer to the user data
        let ptr = Arc::new(RwLock::new(user));

        // Insert the user data into the relevant hash maps
        self.indexed_by_session_id.insert(session_id.clone(), ptr.clone());
        self.indexed_by_user_id.insert(user_id, ptr.clone());
        self.indexed_by_username.insert(username, ptr.clone());
        username_unicode
            .and_then(|s| self.indexed_by_username_unicode.insert(s, ptr));

        // Increment the length of the collection
        self.len += 1;

        // Return the session ID of the created or updated session
        session_id
    }

    /// Deletes the user session matching the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to match the user session to delete
    ///
    /// # Returns
    ///
    /// An `Option` containing the `Arc<RwLock<User>>` for the deleted user session,
    /// or `None` if no matching session was found.
    #[inline]
    pub async fn delete(
        &mut self,
        query: &UserQuery,
    ) -> Option<Arc<RwLock<User>>> {
        // Retrieve the user data for the matching session
        self.delete_user(self.get(query)?.write().await)
    }

    /// Removes a user from all the indexes and returns the removed user.
    /// If the user was not found in any of the indexes, returns `None`.
    ///
    /// # Arguments
    ///
    /// * `user` - A reference to the user to remove.
    ///
    /// # Returns
    ///
    /// An `Option` that contains the removed user wrapped in an `Arc<RwLock<User>>`, or `None` if the user was not found.
    ///
    #[inline]
    pub fn delete_user(
        &mut self,
        user: impl Deref<Target = User>,
    ) -> Option<Arc<RwLock<User>>> {
        let mut removed = None;

        // Remove the user from the indexed_by_user_id map and update the removed variable if a user was removed.
        self.indexed_by_user_id
            .remove(&user.user_id)
            .and_then(|u| Some(removed = Some(u)));

        // Remove the user from the indexed_by_username map and update the removed variable if a user was removed.
        self.indexed_by_username
            .remove(&user.username)
            .and_then(|u| Some(removed = Some(u)));

        // Remove the user from the indexed_by_session_id map and update the removed variable if a user was removed.
        self.indexed_by_session_id
            .remove(&user.session_id)
            .and_then(|u| Some(removed = Some(u)));

        // Remove the user from the indexed_by_username_unicode map and update the removed variable if a user was removed.
        user.username_unicode
            .as_ref()
            .and_then(|s| self.indexed_by_username_unicode.remove(s))
            .and_then(|u| Some(removed = Some(u)));

        // Decrease the length of the map if a user was removed.
        if removed.is_some() {
            self.len -= 1;
        }

        removed
    }

    /// Retrieves a user based on the specified query.
    ///
    /// # Arguments
    ///
    /// * `query` - A reference to a `UserQuery` that specifies how to search for the user.
    ///
    /// # Returns
    ///
    /// An optional `Arc<RwLock<User>>` that contains the user if it was found.
    ///
    #[inline]
    pub fn get(&self, query: &UserQuery) -> Option<Arc<RwLock<User>>> {
        match query {
            UserQuery::UserId(user_id) => self.indexed_by_user_id.get(user_id),
            UserQuery::Username(username) => {
                self.indexed_by_username.get(username)
            },
            UserQuery::UsernameUnicode(username_unicode) => {
                self.indexed_by_username_unicode.get(username_unicode)
            },
            UserQuery::SessionId(session_id) => {
                self.indexed_by_session_id.get(session_id)
            },
        }
        .cloned()
    }

    /// Checks whether a user with the specified query exists.
    ///
    /// # Arguments
    ///
    /// * `query` - A reference to a `UserQuery` that specifies how to search for the user.
    ///
    /// # Returns
    ///
    /// A boolean value that indicates whether a user with the specified query exists.
    ///
    #[inline]
    pub fn exists(&self, query: &UserQuery) -> bool {
        match query {
            UserQuery::UserId(user_id) => {
                self.indexed_by_user_id.contains_key(user_id)
            },
            UserQuery::Username(username) => {
                self.indexed_by_username.contains_key(username)
            },
            UserQuery::UsernameUnicode(username_unicode) => {
                self.indexed_by_username_unicode.contains_key(username_unicode)
            },
            UserQuery::SessionId(session_id) => {
                self.indexed_by_session_id.contains_key(session_id)
            },
        }
    }

    /// Clears all user records from the [`UserSessions`].
    #[inline]
    pub fn clear(&mut self) {
        self.indexed_by_session_id.clear();
        self.indexed_by_username.clear();
        self.indexed_by_username_unicode.clear();
        self.indexed_by_session_id.clear();

        self.len = 0;
    }

    /// Returns the number of user records in the [`UserSessions`].
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}
