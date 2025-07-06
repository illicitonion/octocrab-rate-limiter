use std::{pin::Pin, sync::Arc, time::Duration};

use http::{HeaderValue, header::AUTHORIZATION};
use moka::future::Cache;
use tokio::sync::Semaphore;
use tower::{Layer, Service};

#[derive(Clone, Debug)]
pub struct AccessTokenRateLimitLayer {
    per_token_semaphores: Cache<HeaderValue, Arc<tokio::sync::Semaphore>>,
}

impl AccessTokenRateLimitLayer {
    pub fn new(idle_ttl: Duration) -> AccessTokenRateLimitLayer {
        let per_token_semaphores = Cache::builder().time_to_idle(idle_ttl).build();
        AccessTokenRateLimitLayer {
            per_token_semaphores,
        }
    }
}

impl<S> Layer<S> for AccessTokenRateLimitLayer {
    type Service = AccessTokenRateLimit<S>;

    fn layer(&self, service: S) -> Self::Service {
        AccessTokenRateLimit {
            inner: service,
            per_token_semaphores: self.per_token_semaphores.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AccessTokenRateLimit<T> {
    inner: T,
    per_token_semaphores: Cache<HeaderValue, Arc<tokio::sync::Semaphore>>,
}

impl<Request, S> Service<http::Request<Request>> for AccessTokenRateLimit<S>
where
    S: Service<http::Request<Request>> + Clone + Send + 'static,
    S::Response: Send,
    S::Error: Send,
    S::Future: Send,
    Request: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    type Future = Pin<Box<dyn Future<Output = <<S as tower::Service<http::Request<Request>>>::Future as std::future::IntoFuture>::Output> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: http::Request<Request>) -> Self::Future {
        let header_value = request.headers().get(AUTHORIZATION).map(|v| v.to_owned());

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let per_token_semaphores = self.per_token_semaphores.clone();
        Box::pin(async move {
            let semaphore;
            let mut permit = None;
            if let Some(header_value) = header_value {
                semaphore = per_token_semaphores
                    .get_with(header_value, async {
                        // GitHub's secondary rate limits start kicking in at 100 parallel requests.
                        Arc::new(Semaphore::new(99))
                    })
                    .await;
                // UNWRAP: We never close these semaphores, and never leak them outside of the struct so no one else can, so acquire can't fail.
                permit = Some(semaphore.acquire().await.unwrap());
            }
            let result = inner.call(request).await;
            drop(permit);
            result
        })
    }
}
