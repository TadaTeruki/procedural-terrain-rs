use std::marker::PhantomData;

use crate::{
    core::{
        attributes::TerrainAttributes,
        traits::{Model, Site},
        units::{Altitude, Length, Step},
    },
    lem::drainage_basin::DrainageBasin,
    lem::stream_tree,
};

/// The default value of the exponent `m` for calculating stream power.
const DEFAULT_M_EXP: f64 = 0.5;

/// Provides methods for generating terrain.
///
/// ### Required parameters
///  - `model` is the vector representation of the terrain.
///  - `attributes` is the attributes of sites. Attributes contains uplift rates, erodibilities, base altitudes and maximum slopes (see [TerrainAttributes] for details).
/// ### Optional parameters
///  - `max_iteration` is the maximum number of iterations. If not set, the iterations will be repeated until the altitudes of all sites are stable.
///  - `m_exp` is the constants for calculating stream power. If not set, the default value is 0.5.
///
pub struct TerrainGenerator<S, M, T>
where
    S: Site,
    M: Model<S, T>,
{
    model: Option<M>,
    attributes: Option<Vec<TerrainAttributes>>,
    max_iteration: Option<Step>,
    m_exp: Option<f64>,
    _phantom: PhantomData<(S, T)>,
}

impl<S, M, T> Default for TerrainGenerator<S, M, T>
where
    S: Site,
    M: Model<S, T>,
{
    fn default() -> Self {
        Self {
            model: None,
            attributes: None,
            max_iteration: None,
            m_exp: None,
            _phantom: PhantomData,
        }
    }
}

impl<S, M, T> TerrainGenerator<S, M, T>
where
    S: Site,
    M: Model<S, T>,
{
    /// Set the model that contains the set of sites.
    pub fn set_model(self, model: M) -> Self {
        Self {
            model: Some(model),
            ..self
        }
    }

    /// Set the attributes of sites.
    /// attributes contains uplift rates, erodibilities, base altitudes and maximum slopes.
    pub fn set_attributes(self, attributes: Vec<TerrainAttributes>) -> Self {
        Self {
            attributes: Some(attributes),
            ..self
        }
    }

    /// Set the maximum number of iterations.
    pub fn set_max_iteration(self, max_iteration: Step) -> Self {
        Self {
            max_iteration: Some(max_iteration),
            ..self
        }
    }

    /// Set the exponent `m` for calculating stream power.
    pub fn set_exponent_m(self, m_exp: f64) -> Self {
        Self {
            m_exp: Some(m_exp),
            ..self
        }
    }

    /// Generate terrain.
    pub fn generate(self) -> Result<T, Box<dyn std::error::Error>> {
        let model = {
            if let Some(model) = &self.model {
                model
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set `TerrainModel` before generating terrain",
                )));
            }
        };

        let (num, sites, areas, graph, outlets) = (
            model.num(),
            model.sites(),
            model.areas(),
            model.graph(),
            model.outlets(),
        );

        let attributes = {
            if let Some(attributes) = &self.attributes {
                if attributes.len() != num {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "The number of attributes must be equal to the number of sites",
                    )));
                }
                attributes
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "You must set attributes before generating terrain",
                )));
            }
        };

        let m_exp = {
            if let Some(m_exp) = &self.m_exp {
                *m_exp
            } else {
                DEFAULT_M_EXP
            }
        };

        let altitudes: Vec<Altitude> = {
            let mut altitudes = attributes
                .iter()
                .map(|a| a.base_altitude)
                .collect::<Vec<_>>();
            let mut step = 0;
            loop {
                let stream_tree =
                    stream_tree::StreamTree::construct(sites, &altitudes, graph, outlets);

                let mut drainage_areas = areas.to_vec();
                let mut response_times = vec![0.0; num];
                let mut changed = false;

                // calculate altitudes for each drainage basin
                outlets.iter().for_each(|&outlet| {
                    // construct drainage basin
                    let drainage_basin = DrainageBasin::construct(outlet, &stream_tree, graph);

                    // calculate drainage areas
                    drainage_basin.for_each_downstream(|i| {
                        let j = stream_tree.next[i];
                        if j != i {
                            drainage_areas[j] += drainage_areas[i];
                        }
                    });

                    // calculate response times
                    drainage_basin.for_each_upstream(|i| {
                        let j = stream_tree.next[i];
                        let distance: Length = {
                            let (ok, edge) = graph.has_edge(i, j);
                            if ok {
                                edge
                            } else {
                                0.0
                            }
                        };
                        let celerity = attributes[i].erodibility * drainage_areas[i].powf(m_exp);
                        response_times[i] += response_times[j] + 1.0 / celerity * distance;
                    });

                    // calculate altitudes
                    drainage_basin.for_each_upstream(|i| {
                        let mut new_altitude = altitudes[outlet]
                            + attributes[i].uplift_rate
                                * (response_times[i] - response_times[outlet]).max(0.0);

                        // check if the slope is too steep
                        // if max_slope_func is not set, the slope is not checked
                        if let Some(max_slope) = attributes[i].max_slope {
                            let j = stream_tree.next[i];
                            let distance: Length = {
                                let (ok, edge) = graph.has_edge(i, j);
                                if ok {
                                    edge
                                } else {
                                    1.0
                                }
                            };
                            let max_slope = max_slope.tan();
                            let slope = (new_altitude - altitudes[j]) / distance;
                            if slope > max_slope {
                                new_altitude = altitudes[j] + max_slope * distance;
                            }
                        }

                        changed |= new_altitude != altitudes[i];
                        altitudes[i] = new_altitude;
                    });
                });

                // if the altitudes of all sites are stable, break
                if !changed {
                    break;
                }
                step += 1;
                if let Some(max_iteration) = &self.max_iteration {
                    if step >= *max_iteration {
                        break;
                    }
                }
            }

            altitudes
        };

        Ok(model.create_terrain_from_result(&altitudes))
    }
}
