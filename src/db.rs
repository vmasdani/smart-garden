use rusqlite::{params, Connection, Result};

pub fn init() -> Result<Connection>{
    let conn = Connection::open("./data.db3")?;
 
    conn.execute("
        create table if not exists schedule (
            id integer primary key autoincrement,
            hour integer,
            minute integer
        )
    ", params![])?;

    conn.execute("
        create table if not exists watering_time (
            id integer primary key autoincrement,
            minute integer,
            second integer
        )
    ", params![])?;

    conn.execute("
        create table if not exists sensor (
            id integer primary key autoincrement,
            serial_number  integer
        )
    ", params![])?;

    conn.execute("
        create table if not exists sensor_data (
            id integer primary key autoincrement,
            moisture integer
        );
    ", params![])?;
            
    Ok(conn)
}
