// Copyright 2024 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::service::Services;
use poem::endpoint::PrometheusExporter;
use poem::Route;
use poem::{get, EndpointExt};
use poem_openapi::OpenApiService;
use poem_openapi::Tags;
use prometheus::Registry;
use std::ops::Deref;
use std::sync::Arc;

mod healthcheck;
mod template;
mod worker;
mod worker_connect;

#[derive(Tags)]
enum ApiTags {
    Template,
    Worker,
    HealthCheck,
}

pub fn combined_routes(prometheus_registry: Arc<Registry>, services: &Services) -> Route {
    let api_service = make_open_api_service(services);

    let ui = api_service.swagger_ui();
    let spec = api_service.spec_endpoint_yaml();
    let metrics = PrometheusExporter::new(prometheus_registry.deref().clone());

    let connect_services = worker_connect::ConnectService::new(services.worker_service.clone());

    Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .nest("/specs", spec)
        .nest("/metrics", metrics)
        .at(
            "/v2/templates/:template_id/workers/:worker_name/connect",
            get(worker_connect::ws.data(connect_services)),
        )
}

type ApiServices = (
    template::TemplateApi,
    worker::WorkerApi,
    healthcheck::HealthcheckApi,
);

pub fn make_open_api_service(services: &Services) -> OpenApiService<ApiServices, ()> {
    OpenApiService::new(
        (
            template::TemplateApi {
                template_service: services.template_service.clone(),
            },
            worker::WorkerApi {
                template_service: services.template_service.clone(),
                worker_service: services.worker_service.clone(),
            },
            healthcheck::HealthcheckApi,
        ),
        "Golem API",
        "2.0",
    )
}
