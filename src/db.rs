use rusqlite::{Connection, Result};

pub fn init() -> Result<Connection>{
    let conn = Connection::open("./data.db3")?;
    
    Ok(conn)
}
