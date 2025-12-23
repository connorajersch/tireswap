use crate::db::Database;
use kiddo::{KdTree, SquaredEuclidean};
use rusqlite::Result;

const EARTH_RADIUS_KM: f64 = 6371.0;

/// Structure to hold station information with spatial data
#[derive(Debug, Clone)]
pub struct StationWithDistance {
    pub id: i64,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub lon_x: f64,
    #[allow(dead_code)]
    pub lat_y: f64,
    pub distance_km: f64,
    #[allow(dead_code)]
    pub dly_first_date: Option<String>,
    #[allow(dead_code)]
    pub dly_last_date: Option<String>,
}

/// NearestStationFinder uses a k-d tree to efficiently find the closest weather station
/// to a given latitude and longitude using haversine distance.
pub struct NearestStationFinder {
    kdtree: KdTree<f64, 2>,
    stations: Vec<(i64, String, f64, f64, Option<String>, Option<String>)>, // (id, name, lon, lat, dly_first_date, dly_last_date)
}

impl NearestStationFinder {
    /// Create a new NearestStationFinder by loading all stations from the database
    ///
    /// # Arguments
    /// * `db` - Reference to the database connection
    ///
    /// # Returns
    /// * `Result<Self>` - A new NearestStationFinder instance or error
    pub fn new(db: &Database) -> Result<Self> {
        let stations = db.get_all_stations()?;
        let mut kdtree = KdTree::new();

        let mut station_vec = Vec::new();

        for (idx, station) in stations.iter().enumerate() {
            // Store station data
            station_vec.push((
                station.id,
                station.name.clone(),
                station.lon_x,
                station.lat_y,
                station.dly_first_date.clone(),
                station.dly_last_date.clone(),
            ));

            // Insert into k-d tree using [longitude, latitude] as coordinates
            // We use raw coordinates here; haversine will be calculated during search
            // kiddo uses the index as the item value
            kdtree.add(&[station.lon_x, station.lat_y], idx as u64);
        }

        Ok(NearestStationFinder {
            kdtree,
            stations: station_vec,
        })
    }

    /// Calculate haversine distance between two points on Earth
    ///
    /// # Arguments
    /// * `lat1` - Latitude of first point in degrees
    /// * `lon1` - Longitude of first point in degrees
    /// * `lat2` - Latitude of second point in degrees
    /// * `lon2` - Longitude of second point in degrees
    ///
    /// # Returns
    /// * `f64` - Distance in kilometers
    fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }

    /// Find the nearest station to the given coordinates
    ///
    /// # Arguments
    /// * `lat` - Target latitude in degrees
    /// * `lon` - Target longitude in degrees
    ///
    /// # Returns
    /// * `Option<StationWithDistance>` - The nearest station with its distance, or None if no stations exist
    #[allow(dead_code)]
    pub fn find_nearest(&self, lat: f64, lon: f64) -> Option<StationWithDistance> {
        if self.stations.is_empty() {
            return None;
        }

        // Use k-d tree to find nearest neighbors (we'll check more than 1 because
        // Euclidean distance in lat/lon space != haversine distance)
        let k = std::cmp::min(10, self.stations.len());
        let nearest = self.kdtree.nearest_n::<SquaredEuclidean>(&[lon, lat], k);

        // Calculate actual haversine distances for the candidates
        let mut best: Option<StationWithDistance> = None;
        let mut best_distance = f64::INFINITY;

        for neighbour in nearest {
            let idx = neighbour.item as usize;
            // Get station by index
            if let Some((id, name, s_lon, s_lat, dly_first, dly_last)) = self.stations.get(idx) {
                let distance = Self::haversine_distance(lat, lon, *s_lat, *s_lon);

                if distance < best_distance {
                    best_distance = distance;
                    best = Some(StationWithDistance {
                        id: *id,
                        name: name.clone(),
                        lon_x: *s_lon,
                        lat_y: *s_lat,
                        distance_km: distance,
                        dly_first_date: dly_first.clone(),
                        dly_last_date: dly_last.clone(),
                    });
                }
            }
        }

        best
    }

    /// Find the k nearest stations to the given coordinates
    ///
    /// # Arguments
    /// * `lat` - Target latitude in degrees
    /// * `lon` - Target longitude in degrees
    /// * `k` - Number of nearest stations to return
    ///
    /// # Returns
    /// * `Vec<StationWithDistance>` - Vector of k nearest stations sorted by distance
    pub fn find_k_nearest(&self, lat: f64, lon: f64, k: usize) -> Vec<StationWithDistance> {
        if self.stations.is_empty() {
            return vec![];
        }

        // Query more candidates from k-d tree than we need
        let candidates = std::cmp::min(k * 3, self.stations.len());
        let nearest = self
            .kdtree
            .nearest_n::<SquaredEuclidean>(&[lon, lat], candidates);

        // Calculate haversine distances for all candidates
        let mut stations_with_dist: Vec<StationWithDistance> = nearest
            .iter()
            .filter_map(|neighbour| {
                let idx = neighbour.item as usize;
                self.stations
                    .get(idx)
                    .map(|(id, name, s_lon, s_lat, dly_first, dly_last)| {
                        let distance = Self::haversine_distance(lat, lon, *s_lat, *s_lon);
                        StationWithDistance {
                            id: *id,
                            name: name.clone(),
                            lon_x: *s_lon,
                            lat_y: *s_lat,
                            distance_km: distance,
                            dly_first_date: dly_first.clone(),
                            dly_last_date: dly_last.clone(),
                        }
                    })
            })
            .collect();

        // Sort by distance and take k
        stations_with_dist.sort_by(|a, b| a.distance_km.partial_cmp(&b.distance_km).unwrap());
        stations_with_dist.truncate(k);

        stations_with_dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        // Distance between New York and London (approximately 5570 km)
        let ny_lat = 40.7128;
        let ny_lon = -74.0060;
        let london_lat = 51.5074;
        let london_lon = -0.1278;

        let distance =
            NearestStationFinder::haversine_distance(ny_lat, ny_lon, london_lat, london_lon);

        // Allow for some tolerance in the calculation
        assert!(
            (distance - 5570.0).abs() < 100.0,
            "Distance was {}",
            distance
        );
    }

    #[test]
    fn test_haversine_same_point() {
        let lat = 45.0;
        let lon = -75.0;

        let distance = NearestStationFinder::haversine_distance(lat, lon, lat, lon);

        assert!(
            distance < 0.01,
            "Distance should be nearly zero but was {}",
            distance
        );
    }
}
