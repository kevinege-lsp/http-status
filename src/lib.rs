use spin_sdk::http::{IntoResponse, Request, Response, StatusCode};
use spin_sdk::http_service;

/// Returns the HTTP status named by the `status` query string parameter.
///
/// - `?status=404` -> responds with `404 Not Found`
/// - `?status=302` -> responds with `302 Found`
/// - `?status=799`, `?status=abc`, or no parameter -> `200 OK`
///   with a plain-text explanation of why no other status was returned.
#[http_service]
async fn handle_http_status(req: Request) -> anyhow::Result<impl IntoResponse> {
    let (status, body) = resolve(req.uri().query());
    Ok(Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(body))
}

/// Maps the request's query string to the status code and body to return.
fn resolve(query: Option<&str>) -> (StatusCode, String) {
    let Some(raw) = status_param(query) else {
        return (
            StatusCode::OK,
            "No 'status' query parameter supplied. Try ?status=404 to receive that HTTP status in the response.\n"
                .to_string(),
        );
    };

    match parse_status_code(raw) {
        Some(code) if code.is_informational() => (
            StatusCode::OK,
            format!(
                "{} {} is an informational (1xx) status. It can't be sent as a final response, \
                 so it can't be returned here.\n",
                code.as_u16(),
                code.canonical_reason().unwrap_or_default()
            ),
        ),
        Some(code) => {
            let reason = code.canonical_reason().unwrap_or_default();
            (code, format!("{} {}\n", code.as_u16(), reason))
        }
        None => (
            StatusCode::OK,
            format!(
                "'{raw}' is not a valid HTTP status code. Valid codes are the registered \
                 2xx-5xx status codes, e.g. 200, 301, 404, 500.\n"
            ),
        ),
    }
}

/// Finds the first `status` parameter in the query string.
fn status_param(query: Option<&str>) -> Option<&str> {
    query?
        .split('&')
        .filter_map(|pair| pair.split_once('=').or(Some((pair, ""))))
        .find(|(key, _)| *key == "status")
        .map(|(_, value)| value)
}

/// Accepts any registered HTTP status code (1xx-5xx).
fn parse_status_code(raw: &str) -> Option<StatusCode> {
    let code: u16 = raw.trim().parse().ok()?;
    let status = StatusCode::from_u16(code).ok()?;
    status.canonical_reason().is_some().then_some(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_status_codes_are_returned() {
        for code in [200, 201, 202, 301, 302, 307, 308, 400, 401, 403, 404, 418, 429, 500, 502, 503, 504] {
            let (status, body) = resolve(Some(&format!("status={code}")));
            assert_eq!(status.as_u16(), code);
            assert!(body.starts_with(&code.to_string()));
        }
    }

    #[test]
    fn finds_param_among_others() {
        let (status, _) = resolve(Some("foo=bar&status=502&baz=1"));
        assert_eq!(status, StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn invalid_codes_return_ok_with_explanation() {
        for value in ["299", "399", "499", "599", "600", "999", "abc", "", "-1", "4044"] {
            let (status, body) = resolve(Some(&format!("status={value}")));
            assert_eq!(status, StatusCode::OK, "value {value:?}");
            assert!(body.contains("not a valid HTTP status code"), "value {value:?}");
        }
    }

    #[test]
    fn missing_param_returns_ok() {
        assert_eq!(resolve(None).0, StatusCode::OK);
        assert_eq!(resolve(Some("other=1")).0, StatusCode::OK);
    }

    #[test]
    fn informational_codes_return_ok_with_explanation() {
        for code in [100, 101, 102, 103] {
            let (status, body) = resolve(Some(&format!("status={code}")));
            assert_eq!(status, StatusCode::OK);
            assert!(body.contains("informational"));
        }
    }
}
