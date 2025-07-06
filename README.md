# octocrab-rate-limiter

[![crates.io](https://img.shields.io/crates/d/octocrab-rate-limiter.svg)](https://crates.io/crates/octocrab-rate-limiter)
[![Documentation](https://docs.rs/octocrab-rate-limiter/badge.svg)](https://docs.rs/octocrab-rate-limiter/)

A [Tower](https://crates.io/crates/tower) [Layer](https://docs.rs/tower/latest/tower/trait.Layer.html) to help avoid hitting GitHub rate limits when using [octocrab](https://crates.io/crates/octocrab).

There are [several kinds of rate limit when using GitHub](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api), this crate doesn't stick to all of them, but it helps with some (and may help with more over time).

## Standard rate limits

### Primary

- [ ] 1000 requests per hour per repository.

### Secondary

- [x] 100 concurrent requests.
- [ ] 900 points per minute REST, 2000 points per minute GraphQL.
- [ ] 90 seconds of CPU time per 60 seconds of real time.
- [ ] 80 content-generating requests per minute, 500 content-generating requests per hour.
