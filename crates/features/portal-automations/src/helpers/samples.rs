pub fn sample_of(field: &str) -> &'static str {
    match field {
        "schedule.cron" => "0 3 * * *",
        "schedule.at" => "2026-01-01T03:00:00Z",
        "portal.address" => "0.0.0.0:8080",
        "portal.version" => env!("CARGO_PKG_VERSION"),
        "service.id" => "nas",
        "service.name" => "NAS",
        "service.previous_id" => "storage",
        "status.from" => "up",
        "status.to" => "down",
        "status.error" => "connection refused",
        "status.diagnosis" => "refused",
        "user.name" => "admin",
        "client.address" => "192.168.1.20",
        "client.environment" => "home",
        "sign_in.reason" => "credentials",
        "webhook.id" => "7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d",
        "webhook.title" => "Deploy from CI",
        "configuration.revision" => "6f1ed002ab5595859014ebf0951522d9",
        "configuration.previous_revision" => "0cc175b9c0f1b6a831c399e269772661",
        _ => "",
    }
}
