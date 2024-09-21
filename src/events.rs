use r2d2::{Error, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Result};
use std::iter::Iterator;
use std::path::Path;
use std::sync::Arc;

pub type SqlitePool = Pool<SqliteConnectionManager>;
pub type PooledSqliteConnection = PooledConnection<SqliteConnectionManager>;

#[derive(Clone)]
pub struct DatabasePool(Arc<SqlitePool>);
impl DatabasePool {
    pub fn new(database_url: &str) -> Result<Self, r2d2::Error> {
        let manager = SqliteConnectionManager::file(database_url);
        let pool = Pool::new(manager)?;
        Ok(DatabasePool(Arc::new(pool)))
    }
    pub fn get(&self) -> std::result::Result<PooledConnection<SqliteConnectionManager>, Error> {
        self.0.get()
    }
}

/// An event, as seen by the database
pub struct EventData {
    pub(crate) name: String,
    pub(crate) short_description: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) thumbnail: Option<String>, //url
    pub(crate) picture: Option<String>,   //url
    pub(crate) max_participants: Option<usize>,

    pub(crate) server_id: u64,
    pub(crate) manager_role_id: u64,
    pub(crate) participant_role_id: u64,
    pub(crate) manifest_id: u64,
    pub(crate) manifest_channel_id: u64,
    pub(crate) category_id: u64
}

/// A channel or channel category
struct Channel {
    event: String, //Cross-references EventID
    channel_id: u64,
}

pub fn create_connection<P: AsRef<Path>>(path: P) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    Ok(conn)
}

/// Creates the necessary tables
pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS EVENTS(
                    ID INTEGER PRIMARY KEY AUTOINCREMENT,
                    NAME TEXT NOT NULL,
                    SHORT_DESCRIPTION TEXT,
                    DESCRIPTION TEXT,
                    THUMBNAIL TEXT,
                    PICTURE TEXT,
                    MAX_PARTICIPANTS INTEGER,
                    SERVER_ID INTEGER NOT NULL,
                    MANAGER_ROLE_ID INTEGER NOT NULL,
                    PARTICIPANT_ROLE_ID INTEGER NOT NULL,
                    MANIFEST_ID INTEGER NOT NULL,
                    MANIFEST_CHANNEL_ID INTEGER NOT NULL,
                    CATEGORY_ID INTEGER NOT NULL
          )"#,
        (),
    )?;

    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS CHANNELS (
            EVENT_ID INTEGER,                       -- Define the EVENT_ID column first
            CHANNEL_ID INTEGER NOT NULL,               -- Define the CHANNEL_ID column
            FOREIGN KEY(EVENT_ID) REFERENCES EVENTS(ID) ON DELETE CASCADE
        );"#,
        (),
    )?;

    Ok(())
}

/// Inserts a new event into the table.
/// Returns the UID of the inserted event
pub fn insert_event(conn: &Connection, data: EventData) -> Result<i64> {
    conn.execute(
        r#"INSERT INTO EVENTS(
        NAME,
        SHORT_DESCRIPTION,
        DESCRIPTION,
        THUMBNAIL,
        PICTURE,
        MAX_PARTICIPANTS,
        SERVER_ID,
        MANAGER_ROLE_ID,
        PARTICIPANT_ROLE_ID,
        MANIFEST_ID,
        MANIFEST_CHANNEL_ID,
        CATEGORY_ID
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
        params![
            data.name,
            data.short_description,
            data.description,
            data.thumbnail,
            data.picture,
            data.max_participants,
            data.server_id,
            data.manager_role_id,
            data.participant_role_id,
            data.manifest_id,
            data.manifest_channel_id,
            data.category_id
        ],
    )?;

    let mut stmt = conn.prepare("SELECT ID FROM EVENTS WHERE ROWID=?1")?;
    stmt.query_row(params![conn.last_insert_rowid()], |row| {
        Ok(row.get::<_, i64>(0)?)
    })
}

/// Inserts channels into the table.
pub fn insert_channels(conn: &Connection, event_id: i64, channels: Vec<u64>) -> Result<()> {
    let mut stmt = conn.prepare(
        r#"INSERT INTO CHANNELS(
        EVENT_ID,
        CHANNEL_ID
    ) VALUES (?1, ?2)"#,
    )?;

    for channel in channels {
        stmt.execute(params![event_id, channel])?;
    }

    Ok(())
}

pub fn get_channels_by_event_id(conn: &Connection, event_id: i64) -> Result<Vec<u64>> {
    let mut statement = conn.prepare(r#"SELECT CHANNEL_ID FROM CHANNELS WHERE EVENT_ID=?1"#)?;
    let rows = statement.query_map(params![event_id], |row| row.get::<_, u64>(0))?;

    Ok(rows.filter(|x| x.is_ok()).map(|x| x.unwrap()).collect())
}

/// Returns Ok(number of affected rows) if all went well
pub fn delete_event(conn: &Connection, event_id: i64) -> Result<usize> {
    conn.execute(r#"DELETE FROM CHANNELS WHERE EVENT_ID=?1"#, params![event_id])
}

/// Returns Ok((Event_ID, Event_Data)) if an event owns channel [channel_id]
pub fn get_event_by_channel(conn: &Connection, channel_id: u64) -> Result<(i64, EventData)> {
    let id = conn.query_row(
        r#"SELECT EVENT_ID FROM CHANNELS WHERE CHANNEL_ID=?1"#,
        params![channel_id],
        |row| row.get::<_, i64>(0),
    )?;

    conn.query_row(r#"SELECT * FROM EVENTS WHERE ID=?1"#, params![id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            EventData {
                name: row.get(1)?,
                short_description: row.get(2)?,
                description: row.get(3)?,
                thumbnail: row.get(4)?,
                picture: row.get(5)?,
                max_participants: row.get(6)?,
                server_id: row.get(7)?,
                manager_role_id: row.get(8)?,
                participant_role_id: row.get(9)?,
                manifest_id: row.get(10)?,
                manifest_channel_id: row.get(11)?,
                category_id: row.get(12)?
            },
        ))
    })
}

pub fn get_event_by_manifest(conn: &Connection, manifest_id: u64) -> Result<(i64, EventData)> {
    conn.query_row(r#"SELECT * FROM EVENTS WHERE MANIFEST_ID=?1"#, params![manifest_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            EventData {
                name: row.get(1)?,
                short_description: row.get(2)?,
                description: row.get(3)?,
                thumbnail: row.get(4)?,
                picture: row.get(5)?,
                max_participants: row.get(6)?,
                server_id: row.get(7)?,
                manager_role_id: row.get(8)?,
                participant_role_id: row.get(9)?,
                manifest_id: row.get(10)?,
                manifest_channel_id: row.get(11)?,
                category_id: row.get(12)?
            },
        ))
    })
}

pub fn get_all_events(conn: &Connection) -> Result<Vec<(i64, EventData)>> {
    let mut statement = conn.prepare("SELECT * FROM EVENTS")?;
    let event_iter = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                EventData {
                    name: row.get(1)?,
                    short_description: row.get(2)?,
                    description: row.get(3)?,
                    thumbnail: row.get(4)?,
                    picture: row.get(5)?,
                    max_participants: row.get(6)?,
                    server_id: row.get(7)?,
                    manager_role_id: row.get(8)?,
                    participant_role_id: row.get(9)?,
                    manifest_id: row.get(10)?,
                    manifest_channel_id: row.get(11)?,
                    category_id: row.get(12)?,
                },
            ))
        })?;

    Ok(event_iter.filter(|x| x.is_ok()).map(|x| x.unwrap()).collect())
}
