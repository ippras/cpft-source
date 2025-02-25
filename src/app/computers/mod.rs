pub(crate) use self::{
    distance::{
        Computed as DistanceComputed, Key as DistanceKey,
        filtered::{Computed as DistanceFilteredComputed, Key as DistanceFilteredKey},
        plot::{
            Computed as DistancePlotComputed, Key as DistancePlotKey, Value as DistancePlotValue,
        },
    },
    source::{
        Computed as SourceComputed, Key as SourceKey,
        plot::{Computed as SourcePlotComputed, Key as SourcePlotKey, Value as SourcePlotValue},
    },
};

pub(crate) mod plot {
    use egui::emath::Float as _;
    use egui_plot::PlotPoint;
    use std::hash::{Hash, Hasher};

    /// Index key
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct IndexKey(pub PlotPoint);

    impl Eq for IndexKey {}

    impl From<IndexKey> for PlotPoint {
        fn from(value: IndexKey) -> Self {
            PlotPoint {
                x: value.0.x,
                y: value.0.y,
            }
        }
    }

    impl Hash for IndexKey {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.x.ord().hash(state);
            self.0.y.ord().hash(state);
        }
    }

    impl PartialEq for IndexKey {
        fn eq(&self, other: &Self) -> bool {
            self.0.x.ord() == other.0.x.ord() && self.0.y.ord() == other.0.y.ord()
        }
    }
}

pub(crate) mod distance;
pub(crate) mod source;
