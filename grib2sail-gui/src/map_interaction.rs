use std::collections::BTreeMap;

const MIN_ZOOM: f64 = 0.35;
const MAX_ZOOM: f64 = 10.0;

#[derive(Clone, Default)]
pub(crate) struct MapViewport {
    zoom: f64,
    origin: Option<(f64, f64)>,
    touches: BTreeMap<i32, (f64, f64)>,
    pinch: Option<PinchState>,
}

#[derive(Clone, Copy)]
struct PinchState {
    base_zoom: f64,
    base_distance: f64,
}

impl MapViewport {
    pub(crate) fn zoom_wheel(&mut self, origin: (f64, f64), delta_y: f64) {
        let factor = (1.0 - delta_y * 0.0015).clamp(0.75, 1.25);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.origin = Some(origin);
    }

    pub(crate) fn pointer_down(&mut self, pointer_id: i32, pointer_type: &str, point: (f64, f64)) {
        if pointer_type != "touch" {
            return;
        }

        self.touches.insert(pointer_id, point);
        self.ensure_pinch();
    }

    pub(crate) fn pointer_move(&mut self, pointer_id: i32, pointer_type: &str, point: (f64, f64)) {
        if pointer_type != "touch" {
            return;
        }

        if let Some(existing) = self.touches.get_mut(&pointer_id) {
            *existing = point;
        }

        self.ensure_pinch();
        self.apply_pinch();
    }

    pub(crate) fn pointer_up(&mut self, pointer_id: i32, pointer_type: &str) {
        if pointer_type != "touch" {
            return;
        }

        self.touches.remove(&pointer_id);

        if self.touches.len() < 2 {
            self.pinch = None;
        }
    }

    pub(crate) fn transform_style(&self) -> String {
        match self.origin {
            Some((x, y)) => format!(
                "transform: scale({}); transform-origin: {}px {}px; will-change: transform;",
                self.zoom, x, y
            ),
            None => format!(
                "transform: scale({}); transform-origin: 50% 50%; will-change: transform;",
                self.zoom
            ),
        }
    }

    fn ensure_pinch(&mut self) {
        if self.pinch.is_some() || self.touches.len() < 2 {
            return;
        }

        let Some((origin, distance)) = self.pinch_geometry() else {
            return;
        };

        self.origin = Some(origin);
        self.pinch = Some(PinchState {
            base_zoom: self.zoom,
            base_distance: distance.max(1.0),
        });
    }

    fn apply_pinch(&mut self) {
        let Some(pinch) = self.pinch else {
            return;
        };

        let Some((origin, distance)) = self.pinch_geometry() else {
            return;
        };

        self.zoom = (pinch.base_zoom * (distance / pinch.base_distance)).clamp(MIN_ZOOM, MAX_ZOOM);
        self.origin = Some(origin);
    }

    fn pinch_geometry(&self) -> Option<((f64, f64), f64)> {
        let mut points = self.touches.values();
        let &(x0, y0) = points.next()?;
        let &(x1, y1) = points.next()?;

        let origin = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        let distance = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        Some((origin, distance))
    }
}
