use geojson::{GeoJson, Geometry, Value as GeoValue};
use std::sync::OnceLock;

static WORLD_OUTLINES: OnceLock<Vec<CountryPath>> = OnceLock::new();

pub(crate) fn world_outlines() -> &'static [CountryPath] {
    WORLD_OUTLINES.get_or_init(load_world_outlines).as_slice()
}

#[derive(Clone)]
pub(crate) struct CountryPath {
    pub(crate) name: String,
    pub(crate) d: String,
}

fn load_world_outlines() -> Vec<CountryPath> {
    let geojson = include_str!("../assets/world.geojson");
    let parsed = geojson
        .parse::<GeoJson>()
        .expect("failed to parse bundled world GeoJSON");

    let mut outlines = Vec::new();

    let GeoJson::FeatureCollection(collection) = parsed else {
        panic!("expected a GeoJSON FeatureCollection");
    };

    for (index, feature) in collection.features.iter().enumerate() {
        let name = feature
            .properties
            .as_ref()
            .and_then(|props| props.get("ADMIN"))
            .and_then(|value| value.as_str())
            .or_else(|| {
                feature
                    .properties
                    .as_ref()
                    .and_then(|props| props.get("name"))
                    .and_then(|value| value.as_str())
            })
            .or_else(|| {
                feature
                    .properties
                    .as_ref()
                    .and_then(|props| props.get("NAME"))
                    .and_then(|value| value.as_str())
            })
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("country-{index}"));

        let Some(geometry) = feature.geometry.as_ref() else {
            continue;
        };

        let d = geometry_to_svg_path(geometry);
        if !d.is_empty() {
            outlines.push(CountryPath { name, d });
        }
    }

    outlines
}

fn geometry_to_svg_path(geometry: &Geometry) -> String {
    match &geometry.value {
        GeoValue::Polygon(rings) => rings_to_path(rings),
        GeoValue::MultiPolygon(polygons) => polygons.iter().fold(String::new(), |mut acc, rings| {
            acc.push_str(&rings_to_path(rings));
            acc
        }),
        _ => String::new(),
    }
}

fn rings_to_path(rings: &[Vec<Vec<f64>>]) -> String {
    let mut path = String::new();

    for ring in rings {
        if ring.len() < 2 {
            continue;
        }

        let mut iter = ring.iter().filter_map(|point| project_point(point));
        let Some((x0, y0)) = iter.next() else {
            continue;
        };

        path.push_str(&format!("M{x0:.2},{y0:.2}"));

        for (x, y) in iter {
            path.push_str(&format!(" L{x:.2},{y:.2}"));
        }

        path.push_str(" Z ");
    }

    path
}

fn project_point(point: &[f64]) -> Option<(f64, f64)> {
    let [longitude, latitude, ..] = point else {
        return None;
    };

    let x = (longitude + 180.0) / 360.0 * 1000.0;
    let y = (90.0 - latitude) / 180.0 * 500.0;
    Some((x, y))
}
