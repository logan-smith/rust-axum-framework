use axum::http::StatusCode;

/// Handler to get the liveness of the service
pub async fn get_health_endpoint() -> StatusCode {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_health() {
        let response = get_health_endpoint().await;
        assert_eq!(response, StatusCode::OK);
    }
}
