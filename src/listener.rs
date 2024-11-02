use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::{Body, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::ortb::BidRequest;
use http_body_util::BodyExt;

async fn handle_rtb_request<B>(req: Request<B>) -> Result<Response<Full<Bytes>>, B::Error>
where
    B: Body,
{
    let bytes = req.collect().await?.to_bytes().to_vec();

    let resp = match serde_json::from_slice::<BidRequest>(bytes.as_slice()) {
        Ok(_bid_request) => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Full::new(Bytes::new()))
            .expect("no args could fail"),
        Err(err) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from(format!(
                "Error deserializing input as BidRequest: {}",
                err
            ))))
            .unwrap(),
    };

    Ok(resp)
}

pub async fn start(
    socket_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(socket_addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our simple  service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(handle_rtb_request))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::handle_rtb_request;
    use crate::ortb::BidRequest;
    use http_body_util::BodyExt;
    use hyper::{Method, Request, StatusCode};

    #[tokio::test]
    async fn test_listener_request() {
        let socket = "0.0.0.0:4006".parse().unwrap();
        let listener = super::start(socket);

        let resp = async {
            let br = BidRequest::default();
            reqwest::Client::new()
                .post("http://0.0.0.0:4006")
                .json(&br)
                .send()
                .await
                .unwrap()
        };

        tokio::select! {
           _ = listener => {},
           resp = resp => {
            assert!(resp.status() == StatusCode::NO_CONTENT)
           }
        }
    }

    #[tokio::test]
    async fn test_handle_rtb_request() {
        let req_body = BidRequest::default();
        let req_body = serde_json::to_string(&req_body).unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .body(req_body)
            .unwrap();

        let resp = handle_rtb_request(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let msg = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(msg, "");
    }

    #[tokio::test]
    async fn test_handle_rtb_bad_request() {
        let req = Request::builder()
            .method(Method::POST)
            .body("".to_string())
            .unwrap();

        let resp = handle_rtb_request(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let msg = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(
            msg,
            "Error deserializing input as BidRequest: EOF while parsing a value at line 1 column 0"
        );
    }
}
