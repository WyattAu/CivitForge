# CivitForge Sync Benchmark Methodology

## Desktop-to-Server Sync Benchmarks

### File Upload Sync
- Upload test files of sizes: 1KB, 10KB, 100KB, 1MB, 10MB
- Measure: time from start of upload to server acknowledgment
- 10 iterations per size, discard warmup run
- Report: min, mean, median, P95, P99, throughput (MB/s)

### Directory Tree Sync
- Scan local directory tree at depth 1-5
- Compare with server-side directory listing via API
- Measure: time from scan start to diff result
- 10 iterations per depth

### Git Status Sync
- Run `git status` equivalent via gix on local repos
- Measure: time from status check start to result
- 20 iterations

## Server API Response Benchmarks
(Handled by tests/e2e/benchmark.mjs)
- Page load times for all routes
- API response times for all endpoints
- Form interaction latency

## Memory Benchmark
- Navigate 30 pages sequentially
- Measure JS heap before and after
- Report: delta MB (memory leak indicator)
