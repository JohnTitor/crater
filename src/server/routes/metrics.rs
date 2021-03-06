use crate::prelude::*;
use crate::server::agents::Agent;
use crate::server::Data;
use http::{Response, StatusCode};
use hyper::Body;
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use warp::{self, Filter, Rejection};

pub fn routes(
    data: Arc<Data>,
) -> impl Filter<Extract = (Response<Body>,), Error = Rejection> + Clone {
    let data_cloned = data.clone();
    let data_filter = warp::any().map(move || data_cloned.clone());

    warp::get2()
        .and(warp::path::end())
        .and(data_filter.clone())
        .map(|data| match endpoint_metrics(data) {
            Ok(resp) => resp,
            Err(err) => {
                error!("error while processing metrics");
                crate::utils::report_failure(&err);

                let mut resp = Response::new(format!("Error: {}\n", err).into());
                *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                resp
            }
        })
}

fn endpoint_metrics(data: Arc<Data>) -> Fallible<Response<Body>> {
    data.metrics.update_agent_status(
        &data.db,
        &data.agents.all()?.iter().collect::<Vec<&Agent>>(),
    )?;
    let mut buffer = Vec::new();
    let families = prometheus::gather();
    TextEncoder::new().encode(&families, &mut buffer)?;
    Ok(Response::new(Body::from(buffer)))
}
