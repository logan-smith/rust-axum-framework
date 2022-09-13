#[cfg(test)]
pub mod tests {
    use crate::config::CONFIG;
    use crate::database::{add_pool, init_pool, Pool};
    use crate::handlers::auth::LoginRouteParams;
    use crate::pagination::PaginationRequest;
    use crate::routes::routes;
    use crate::state::{new_state, AppState};
    use actix_web::{
        dev::ServiceResponse,
        test,
        web::{Data, HttpRequest, Query},
        App,
    };
    use diesel::mysql::MysqlConnection;
    use serde::Serialize;

    /// Helper for HTTP GET integration tests
    pub async fn test_get(route: &str) -> ServiceResponse {
        let login_request = LoginRequest {
            email: "satoshi@nakamotoinstitute.org".into(),
            password: "123456".into(),
        };

        let mut app = test::init_service(
            App::new()
                .configure(add_cache)
                .app_data(app_state())
                .wrap(get_identity_service())
                .configure(add_pool)
                .configure(routes),
        )
        .await;

        let response = test::call_service(
            &mut app,
            test::TestRequest::post()
                .set_json(&login_request)
                .uri("/api/v1/auth/login")
                .to_request(),
        )
        .await;

        let cookie = response.response().cookies().next().unwrap().to_owned();
        test::call_service(
            &mut app,
            test::TestRequest::get()
                .cookie(cookie.clone())
                .uri(route)
                .to_request(),
        )
        .await
    }

    /// Helper for HTTP POST integration tests
    pub async fn test_post<T: Serialize>(route: &str, params: T) -> ServiceResponse {
        let mut app = test::init_service(
            App::new()
                .configure(add_cache)
                .app_data(app_state())
                .wrap(get_identity_service())
                .configure(add_pool)
                .configure(routes),
        )
        .await;
        let login = login().await;
        let cookie = login.response().cookies().next().unwrap().to_owned();
        test::call_service(
            &mut app,
            test::TestRequest::post()
                .set_json(&params)
                .cookie(cookie.clone())
                .uri(route)
                .to_request(),
        )
        .await
    }

    /// Mock a HttpRequest for testing handlers
    pub fn mock_get_request(url: &str) -> HttpRequest {
        test::TestRequest::get().uri(url).to_http_request()
    }

    /// Helper to login for tests
    // pub fn login_request() -> Request {
    //     let login_request = LoginRequest {
    //         email: "satoshi@nakamotoinstitute.org".into(),
    //         password: "123456".into(),
    //     };
    //     test::TestRequest::post()
    //         .set_json(&login_request)
    //         .uri("/api/v1/auth/login")
    //         .to_request()
    // }

    /// Assert that a route is successful for HTTP GET requests
    pub async fn assert_get(route: &str) -> ServiceResponse {
        let response = test_get(route).await;
        assert!(response.status().is_success());
        response
    }

    /// Assert that a route is successful for HTTP POST requests
    pub async fn assert_post<T: Serialize>(route: &str, params: T) -> ServiceResponse {
        let response = test_post(route, params).await;
        assert!(response.status().is_success());
        response
    }

    /// Returns a r2d2 Pooled Connection to be used in tests
    pub fn get_pool() -> Pool<MysqlConnection> {
        println!("CONFIG: {:#?}", CONFIG.database_url);
        init_pool::<MysqlConnection>(CONFIG.clone()).unwrap()
    }

    /// Returns a r2d2 Pooled Connection wrappedn in Actix Application Data
    pub fn get_data_pool() -> Data<Pool<MysqlConnection>> {
        Data::new(get_pool())
    }

    /// Utility to get pagination params
    pub fn get_pagination_params() -> PaginationRequest {
        PaginationRequest {
            page: Some(1),
            per_page: Some(10),
        }
    }

    /// Wrap pagination params in a Query
    pub fn get_query_pagination_params() -> Query<PaginationRequest> {
        Query(get_pagination_params())
    }

    /// Login to routes  
    pub async fn login() -> ServiceResponse {
        let login_request = LoginRequest {
            email: "satoshi@nakamotoinstitute.org".into(),
            password: "123456".into(),
        };
        let mut app = test::init_service(
            App::new()
                .wrap(get_identity_service())
                .configure(add_pool)
                .configure(routes),
        )
        .await;
        test::call_service(
            &mut app,
            test::TestRequest::post()
                .set_json(&login_request)
                .uri("/api/v1/auth/login")
                .to_request(),
        )
        .await
    }

    // Mock applicate state
    pub fn app_state() -> AppState<'static, String> {
        new_state::<String>()
    }
}
