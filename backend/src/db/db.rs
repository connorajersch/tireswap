use rusqlite::{Connection, Result, params};
use std::path::Path;

/// Struct to represent climate data
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClimateData {
    pub id: i64,
    pub station_id: i64,
    pub year: i64,
    pub switch_to_summer: Option<String>,
    pub switch_to_winter: Option<String>,
}

/// Database struct to manage SQLite connections
pub struct Database {
    conn: Connection,
}

pub struct Station {
    pub id: i64,
    pub name: String,
    pub lon_x: f64,
    pub lat_y: f64,
    pub dly_first_date: Option<String>,
    pub dly_last_date: Option<String>,
}

impl Database {
    /// Initialize a new database connection
    ///
    /// # Arguments
    /// * `db_path` - Path to the database file
    ///
    /// # Returns
    /// * `Result<Self>` - Database instance or error
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Database { conn })
    }

    /// Create a new in-memory database (useful for testing)
    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Database { conn })
    }

    /// Initialize the database schema
    /// Creates tables for weather stations and climate data
    pub fn initialize_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS stations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                lon_x REAL NOT NULL,
                lat_y REAL NOT NULL,
                dly_first_date TEXT,
                dly_last_date TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                station_id INTEGER NOT NULL,
                year INTEGER NOT NULL,
                switch_to_summer TEXT,
                switch_to_winter TEXT,
                FOREIGN KEY (station_id) REFERENCES stations(id)
            )",
            [],
        )?;

        Ok(())
    }

    /// Insert a new station into the database
    ///
    /// # Arguments
    /// * `id` - Station ID
    /// * `name` - Station name
    /// * `lon_x` - Longitude
    /// * `lat_y` - Latitude
    /// * `dly_first_date` - First date of daily weather recordings
    /// * `dly_last_date` - Last date of daily weather recordings
    ///
    /// # Returns
    /// * `Result<usize>` - Number of rows affected
    pub fn insert_station(
        &self,
        id: i64,
        name: &String,
        lon_x: f64,
        lat_y: f64,
        dly_first_date: Option<&str>,
        dly_last_date: Option<&str>,
    ) -> Result<usize> {
        self.conn.execute(
            "INSERT OR REPLACE INTO stations (id, name, lon_x, lat_y, dly_first_date, dly_last_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, lon_x, lat_y, dly_first_date, dly_last_date],
        )
    }

    /// Insert climate data into the database
    ///
    /// # Arguments
    /// * `station_id` - Station ID
    /// * `year` - Year
    /// * `switch_to_summer` - Switch to summer tires date
    /// * `switch_to_winter` - Switch to winter tires date
    ///
    /// # Returns
    /// * `Result<i64>` - ID of the inserted data
    #[allow(dead_code)]
    pub fn insert_data(
        &self,
        station_id: i64,
        year: i64,
        switch_to_summer: Option<&str>,
        switch_to_winter: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO data (station_id, year, switch_to_summer, switch_to_winter)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                station_id,
                year,
                switch_to_summer,
                switch_to_winter
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get a station by ID
    ///
    /// # Arguments
    /// * `station_id` - Station ID
    ///
    /// # Returns
    /// * `Result<Option<(i64, String, f64, f64)>>` - Station data (id, name, lon_x, lat_y) or None
    #[allow(dead_code)]
    pub fn get_station_by_id(&self, station_id: i64) -> Result<Option<(i64, String, f64, f64)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, lon_x, lat_y FROM stations WHERE id = ?1")?;

        let mut rows = stmt.query(params![station_id])?;

        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let lon_x: f64 = row.get(2)?;
            let lat_y: f64 = row.get(3)?;
            Ok(Some((id, name, lon_x, lat_y)))
        } else {
            Ok(None)
        }
    }

    /// Query all stations
    ///
    /// # Returns
    /// * `Result<Vec<Station>>` - Vector of station data
    pub fn get_all_stations(&self) -> Result<Vec<Station>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, lon_x, lat_y, dly_first_date, dly_last_date FROM stations",
        )?;

        let stations = stmt.query_map([], |row| {
            Ok(Station {
                id: row.get(0)?,
                name: row.get(1)?,
                lon_x: row.get(2)?,
                lat_y: row.get(3)?,
                dly_first_date: row.get(4)?,
                dly_last_date: row.get(5)?,
            })
        })?;

        let mut result = Vec::new();
        for station in stations {
            result.push(station?);
        }
        Ok(result)
    }

    /// Get climate data by station ID
    ///
    /// # Arguments
    /// * `station_id` - Station ID
    ///
    /// # Returns
    /// * `Result<Vec<ClimateData>>` - Vector of climate data for the station
    #[allow(dead_code)]
    pub fn get_data_by_station(&self, station_id: i64) -> Result<Vec<ClimateData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, station_id, year, switch_to_summer, switch_to_winter
             FROM data WHERE station_id = ?1",
        )?;

        let data_entries = stmt.query_map(params![station_id], |row| {
            Ok(ClimateData {
                id: row.get(0)?,
                station_id: row.get(1)?,
                year: row.get(2)?,
                switch_to_summer: row.get(3)?,
                switch_to_winter: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for entry in data_entries {
            result.push(entry?);
        }
        Ok(result)
    }

    /// Get climate data by year
    ///
    /// # Arguments
    /// * `year` - Year
    ///
    /// # Returns
    /// * `Result<Vec<ClimateData>>` - Vector of climate data for the year
    #[allow(dead_code)]
    pub fn get_data_by_year(&self, year: i64) -> Result<Vec<ClimateData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, station_id, year, switch_to_summer, switch_to_winter
             FROM data WHERE year = ?1",
        )?;

        let data_entries = stmt.query_map(params![year], |row| {
            Ok(ClimateData {
                id: row.get(0)?,
                station_id: row.get(1)?,
                year: row.get(2)?,
                switch_to_summer: row.get(3)?,
                switch_to_winter: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for entry in data_entries {
            result.push(entry?);
        }
        Ok(result)
    }

    /// Query all climate data
    ///
    /// # Returns
    /// * `Result<Vec<ClimateData>>` - Vector of all climate data entries
    #[allow(dead_code)]
    pub fn get_all_data(&self) -> Result<Vec<ClimateData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, station_id, year, switch_to_summer, switch_to_winter FROM data",
        )?;

        let data_entries = stmt.query_map([], |row| {
            Ok(ClimateData {
                id: row.get(0)?,
                station_id: row.get(1)?,
                year: row.get(2)?,
                switch_to_summer: row.get(3)?,
                switch_to_winter: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for entry in data_entries {
            result.push(entry?);
        }
        Ok(result)
    }

    /// Delete a station and all its associated data
    ///
    /// # Arguments
    /// * `station_id` - Station ID
    ///
    /// # Returns
    /// * `Result<usize>` - Number of rows affected
    #[allow(dead_code)]
    pub fn delete_station(&self, station_id: i64) -> Result<usize> {
        // First delete associated data
        self.conn.execute(
            "DELETE FROM data WHERE station_id = ?1",
            params![station_id],
        )?;
        // Then delete the station
        self.conn
            .execute("DELETE FROM stations WHERE id = ?1", params![station_id])
    }

    /// Execute a custom query
    ///
    /// # Arguments
    /// * `query` - SQL query string
    ///
    /// # Returns
    /// * `Result<usize>` - Number of rows affected
    #[allow(dead_code)]
    pub fn execute_query(&self, query: &str) -> Result<usize> {
        self.conn.execute(query, [])
    }

    /// Begin a transaction
    #[allow(dead_code)]
    pub fn begin_transaction(&mut self) -> Result<()> {
        self.conn.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }

    /// Commit a transaction
    #[allow(dead_code)]
    pub fn commit_transaction(&mut self) -> Result<()> {
        self.conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// Rollback a transaction
    #[allow(dead_code)]
    pub fn rollback_transaction(&mut self) -> Result<()> {
        self.conn.execute("ROLLBACK", [])?;
        Ok(())
    }

    /// Get the underlying connection for advanced operations
    #[allow(dead_code)]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_initialization() {
        let db = Database::new_in_memory().unwrap();
        db.initialize_schema().unwrap();
    }

    #[test]
    fn test_insert_and_query_station() {
        let db = Database::new_in_memory().unwrap();
        db.initialize_schema().unwrap();

        db.insert_station(
            4607,
            &"Test Station".to_string(),
            -79.4,
            43.7,
            Some("2020-01-01"),
            Some("2023-12-31"),
        )
        .unwrap();

        let station = db.get_station_by_id(4607).unwrap();
        assert!(station.is_some());

        let (id, name, lon_x, lat_y) = station.unwrap();
        assert_eq!(id, 4607);
        assert_eq!(name, "Test Station");
        assert_eq!(lon_x, -79.4);
        assert_eq!(lat_y, 43.7);
    }

    #[test]
    fn test_insert_and_query_data() {
        let db = Database::new_in_memory().unwrap();
        db.initialize_schema().unwrap();

        db.insert_station(4607, &"Test Station".to_string(), -79.4, 43.7, None, None)
            .unwrap();
        let data_id = db
            .insert_data(
                4607,
                2023,
                Some("2023-10-20"),
                Some("2023-11-05"),
            )
            .unwrap();
        assert!(data_id > 0);

        let data = db.get_data_by_station(4607).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].year, 2023);
        assert_eq!(data[0].switch_to_summer, Some("2023-10-20".to_string()));
    }

    #[test]
    fn test_get_all_stations() {
        let db = Database::new_in_memory().unwrap();
        db.initialize_schema().unwrap();

        db.insert_station(
            4607,
            &"Station 1".to_string(),
            -79.4,
            43.7,
            Some("2020-01-01"),
            Some("2023-12-31"),
        )
        .unwrap();
        db.insert_station(5678, &"Station 2".to_string(), -80.0, 44.0, None, None)
            .unwrap();

        let stations = db.get_all_stations().unwrap();
        assert_eq!(stations.len(), 2);
    }

    #[test]
    fn test_delete_station() {
        let db = Database::new_in_memory().unwrap();
        db.initialize_schema().unwrap();

        db.insert_station(4607, &"Test Station".to_string(), -79.4, 43.7, None, None)
            .unwrap();
        db.insert_data(4607, 2023, None, None)
            .unwrap();

        db.delete_station(4607).unwrap();

        let station = db.get_station_by_id(4607).unwrap();
        assert!(station.is_none());

        let data = db.get_data_by_station(4607).unwrap();
        assert_eq!(data.len(), 0);
    }
}
