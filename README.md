# Rust/Axum Framework

A web framework built upon Axum using the Rust language.
Based on the Rust/Actix sample repository https://github.com/ddimaria/rust-actix-framework

## Features

- Actix 3.x HTTP Server
- ~~Multi-Database Support (CockroachDB, Postgres, MySQL, Sqlite)~~
- ~~JWT Support~~
- ~~Async Caching Layer with a Simple API~~
- ~~Public and Secure Static File Service~~
- Diesel Database Operations are Non-Blocking
- Filesystem Organized for Scale
- .env for Local Development
- Integrated Application State with a Simple API
- Lazy Static Config struct
- Built-in Healthcheck (includes cargo version info)
- Listeners configured for TDD
- Custom Errors and HTTP Payload/Json Validation
- Secure Argon2i Password Hashing
- CORS Support
- Paginated Results
- ~~Unit and Integration Tests~~
- ~~Test Coverage Reports~~
- Dockerfile for Running the Server in a Container
- TravisCI Integration

## Featured Packages

- `Argon2i`: Argon2i Password Hashing
- `axum`: Axum Web Server
- `axum`: Axum Web Server
- `axum-sessions`: User Authentication
- `derive_more`: Error Formatting
- `diesel`: ORM that Operates on Several Databases
- `dotenv`: Configuration Loader (.env)
- `envy`: Deserializes Environment Variables into a Config Struct
- `kcov`: Coverage Analysis
- `rayon`: Parallelize
- `r2d2`: Database Connection Pooling
- `validator`: Validates incoming Json

# Table of Contents

- [Installation](#installation)
- [Running the Server](#running-the-server)
- [Autoreloading](#autoreloading)
- [Tests](#tests)
  - [Running Tests](#running-tests)
- [Docker](#docker)
  - [Docker Compose](#docker-compose)
- [Generating documentation](#generating-documentation)
- [The #[timestamps] proc macro](#the-timestamps-proc-macro)
- [The paginate! declaritive macro](#the-paginate-declaritive-macro)
- [Endpoints](#endpoints)
  - [Healthcheck](#healthcheck)
  - [Login](#login)
  - [Get All Users](#get-all-users)
  - [Get a User](#get-a-user)
  - [Create a User](#create-a-user)
- [License](#license)

## Installation

Create an .env file at the root of your project:

```shell
touch .env
```

Now add environment values for local development:

```ini
AUTH_SALT=CHANGEME
DATABASE=mysql
DATABASE_URL=mysql://root:root@0.0.0.0:13306/rust-actix-framework
JWT_EXPIRATION=24
JWT_KEY=4125442A472D4B614E645267556B58703273357638792F423F4528482B4D6251
REDIS_URL=127.0.0.1:6379
RUST_BACKTRACE=0
RUST_LOG="rust_axum_framework=info,axum=info"
SERVER=127.0.0.1:3000
SESSION_KEY=4125442A472D4B614E645267556B58703273357638792F423F4528482B4D6251
SESSION_NAME=auth
SESSION_SECURE=false
SESSION_TIMEOUT=20
```

**IMPORTANT:** Change .env values for your setup, paying special attention to the salt and various keys.

Next, you'll need to install the Diesel CLI:

```shell
cargo install diesel_cli
```

If you run into errors, see http://diesel.rs/guides/getting-started/

After you've created a blank database, run the migrations via the Diesel CLI:

```shell
diesel migration run
```

## Running the Server

To startup the server:

```shell
cargo run
```

## Autoreloading

To startup the server and autoreload on code changes:

```shell
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

## Tests

Integration tests are in the `/src/tests` folder. There are helper functions
to make testing the API straightforward. For example, if we want to test the
`GET /api/v1/user` route:

```rust
  use crate::tests::helpers::tests::assert_get;

  #[test]
  async fn test_get_users() {
      assert_get("/api/v1/user").await;
  }
```

Using the Actix test server, the request is sent and the response is asserted
for a successful response:

`assert!(response.status().is_success());`

Similarly, to test a POST route:

```rust
use crate::handlers::user::CreateUserRequest;
use crate::tests::helpers::tests::assert_post;

#[test]
async fn test_create_user() {
    let params = CreateUserRequest {
        first_name: "Satoshi".into(),
        last_name: "Nakamoto".into(),
        email: "satoshi@nakamotoinstitute.org".into(),
    };
    assert_post("/api/v1/user", params).await;
}
```

### Running Tests

To run all of the tests:

```shell
cargo test
```

## Docker

To build a Docker image of the application:

```shell
docker build -t actix_framework .
```

Once the image is built, you can run the container in port 3000:

```shell
docker run -it --rm --env-file=.env.docker -p 3000:3000 --name actix_framework actix_framework
```

### Docker Compose

To run dependencies for this application, simply invoke docker-compose:

```shell
docker-compose up
```

## Generating documentation

```shell
cargo doc --no-deps --open
```

## The #[timestamps] proc macro

The `#[timestamps]` macro will automatically append the following fields to a model struct:

```rust
pub created_by: String,
pub created_at: NaiveDateTime,
pub updated_by: String,
pub updated_at: NaiveDateTime,
```

Example:

```rust
use chrono::NaiveDateTime;
use proc_macro::timestamps;

#[timestamps]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Identifiable, Insertable)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}
```

This will expand to:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Identifiable, Insertable)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub created_by: String,
    pub created_at: NaiveDateTime,
    pub updated_by: String,
    pub updated_at: NaiveDateTime,
}
```

## The paginate! declaritive macro

The `paginate!` macro removes boilerplate for paginating a model.

```rust
macro_rules! paginate {
    ($pool:expr, $model:ident, $model_type:ident, $params:ident, $response_type:ident, $base:ident) => {{
        let conn = $pool.get()?;
        let total = $model.select(count_star()).first(&conn)?;
        let pagination = get_pagination($params.page, $params.per_page, total);
        let paginated: $response_type = $model
            .limit(pagination.per_page)
            .offset(pagination.offset)
            .load::<$model_type>(&conn)?
            .into();

        Ok(paginate::<$response_type>(pagination, paginated, $base)?)
    }};
}
```

Below is an example of using the macro in the user model:

```rust
pub fn get_all(
    pool: &PoolType,
    params: PaginationRequest,
    base: String,
) -> Result<PaginationResponse<UsersResponse>, ApiError> {
    use crate::schema::users::dsl::users;

    crate::paginate!(pool, users, User, params, UsersResponse, base)
}
```

## Endpoints

### Healthcheck

Determine if the system is healthy.

`GET /health`

#### Response

`200 OK`

Example:

```shell
curl -X GET http://127.0.0.1:3000/health
```

### Login

`POST /api/v1/auth/login`

#### Request

| Param    | Type   | Description              | Required | Validations           |
| -------- | ------ | ------------------------ | :------: | --------------------- |
| email    | String | The user's email address |   yes    | valid email address   |
| password | String | The user's password      |   yes    | at least 6 characters |

```json
{
  "email": "torvalds@transmeta.com",
  "password": "123456"
}
```

#### Response

Header

```json
HTTP/1.1 200 OK
content-length: 118
content-type: application/json
set-cookie: auth=COOKIE_VALUE_HERE; HttpOnly; Path=/; Max-Age=1200
date: Tue, 15 Oct 2019 02:04:54 GMT
```

Json Body

```json
{
  "id": "0c419802-d1ef-47d6-b8fa-c886a23d61a7",
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

**When sending subsequent requests, create a header variable `cookie` with the value `auth=COOKIE_VALUE_HERE`**

### Logout

`GET /api/v1/auth/logout`

#### Response

`200 OK`

Example:

```shell
curl -X GET http://127.0.0.1:3000/api/v1/auth/logout
```

### Get All Users

Retrieve a paginated listing of all users in the system.

`GET /api/v1/user`

#### Query Parameters

| Param    | Type | Description                                     |
| -------- | ---- | ----------------------------------------------- |
| page     | i64  | The page to start on. Defaults to 1.            |
| per_page | i64  | The number of results per page. Defaults to 10. |

#### Response

```json
{
  "links": {
    "base": "http://127.0.0.1:3000/api/v1/user",
    "first": "http://127.0.0.1:3000/api/v1/user?page=1&per_page=10",
    "last": "http://127.0.0.1:3000/api/v1/user?page=13&per_page=10",
    "prev": null,
    "next": "http://127.0.0.1:3000/api/v1/user?page=2&per_page=10"
  },
  "pagination": {
    "offset": 0,
    "page": 1,
    "per_page": 10,
    "total": 129,
    "total_pages": 13
  },
  "data": [
    {
      "id": "00000000-0000-0000-0000-000000000000",
      "first_name": "admin",
      "last_name": "user",
      "email": "admin@admin.com"
    },
    {
      "id": "035efb82-cfdf-42de-adef-c75d7ac6d3ff",
      "first_name": "ModelUpdateaaa",
      "last_name": "TestUpdatezzz",
      "email": "model-update-test@nothing.org"
    }
  ]
}
```

Example:

```shell
curl -X GET http://127.0.0.1:3000/api/v1/user
```

### Get a User

`GET /api/v1/user/{id}`

#### Request

| Param | Type | Description   |
| ----- | ---- | ------------- |
| id    | Uuid | The user's id |

#### Response

```json
{
  "id": "a421a56e-8652-4da6-90ee-59dfebb9d1b4",
  "first_name": "Satoshi",
  "last_name": "Nakamoto",
  "email": "satoshi@nakamotoinstitute.org"
}
```

Example:

```shell
curl -X GET http://127.0.0.1:3000/api/v1/user/a421a56e-8652-4da6-90ee-59dfebb9d1b4
```

#### Response - Not Found

`404 Not Found`

```json
{
  "errors": ["User c63d285b-7794-4419-bfb7-86d7bb3ff17a not found"]
}
```

### Create a User

`POST /api/v1/user`

#### Request

| Param      | Type   | Description              | Required | Validations           |
| ---------- | ------ | ------------------------ | :------: | --------------------- |
| first_name | String | The user's first name    |   yes    | at least 3 characters |
| last_name  | String | The user's last name     |   yes    | at least 3 characters |
| email      | String | The user's email address |   yes    | valid email address   |

```json
{
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

#### Response

```json
{
  "id": "0c419802-d1ef-47d6-b8fa-c886a23d61a7",
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

Example:

```shell
curl -X POST \
  http://127.0.0.1:3000/api/v1/user \
  -H 'Content-Type: application/json' \
  -d '{
    "first_name": "Linus",
    "last_name": "Torvalds",
    "email": "torvalds@transmeta.com"
}'
```

#### Response - Validation Errors

`422 Unprocessable Entity`

```json
{
  "errors": [
    "first_name is required and must be at least 3 characters",
    "last_name is required and must be at least 3 characters",
    "email must be a valid email"
  ]
}
```

### Update a User

`PUT /api/v1/{id}`

#### Request

Path

| Param | Type | Description   |
| ----- | ---- | ------------- |
| id    | Uuid | The user's id |

Body

| Param      | Type   | Description              | Required | Validations           |
| ---------- | ------ | ------------------------ | :------: | --------------------- |
| first_name | String | The user's first name    |   yes    | at least 3 characters |
| last_name  | String | The user's last name     |   yes    | at least 3 characters |
| email      | String | The user's email address |   yes    | valid email address   |

```json
{
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

#### Response

```json
{
  "id": "0c419802-d1ef-47d6-b8fa-c886a23d61a7",
  "first_name": "Linus",
  "last_name": "Torvalds",
  "email": "torvalds@transmeta.com"
}
```

Example:

```shell
curl -X PUT \
  http://127.0.0.1:3000/api/v1/user/0c419802-d1ef-47d6-b8fa-c886a23d61a7 \
  -H 'Content-Type: application/json' \
  -d '{
    "first_name": "Linus",
    "last_name": "Torvalds",
    "email": "torvalds@transmeta.com"
}'
```

#### Response - Validation Errors

`422 Unprocessable Entity`

```json
{
  "errors": [
    "first_name is required and must be at least 3 characters",
    "last_name is required and must be at least 3 characters",
    "email must be a valid email"
  ]
}
```

#### Response - Not Found

`404 Not Found`

```json
{
  "errors": ["User 0c419802-d1ef-47d6-b8fa-c886a23d61a7 not found"]
}
```

### Delete a User

`DELETE /api/v1/user/{id}`

#### Request

| Param | Type | Description   |
| ----- | ---- | ------------- |
| id    | Uuid | The user's id |

#### Response

```json
{
  "id": "a421a56e-8652-4da6-90ee-59dfebb9d1b4",
  "first_name": "Satoshi",
  "last_name": "Nakamoto",
  "email": "satoshi@nakamotoinstitute.org"
}
```

#### Response

`200 OK`

Example:

```shell
curl -X DELETE http://127.0.0.1:3000/api/v1/user/a421a56e-8652-4da6-90ee-59dfebb9d1b4
```

#### Response - Not Found

`404 Not Found`

```json
{
  "errors": ["User c63d285b-7794-4419-bfb7-86d7bb3ff17a not found"]
}
```

## License

This project is licensed under:

- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
